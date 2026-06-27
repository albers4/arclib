// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use ir::{
    BlockId, BlockSuccessor, IrError, OperationId, OperationName, OperationRef, PatternBenefit,
    PatternRewriter, RegionId, RewriteError, Type, ValueId,
};

use crate::conversion::{
    SignatureConversion, SignatureConversionReport, apply_signature_conversion,
    convert_region_entry_signature,
};

use super::{ConversionValueMapping, TypeConversionError, TypeConverter};

pub struct ConvertedOperands {
    values: Vec<Vec<ValueId>>,
}

impl ConvertedOperands {
    pub fn get(&self, index: usize) -> Option<&[ValueId]> {
        self.values.get(index).map(Vec::as_slice)
    }

    pub fn single(&self, index: usize) -> Result<ValueId, TypeConversionError> {
        let values = self.get(index).unwrap_or(&[]);

        if values.len() != 1 {
            return Err(TypeConversionError::ExpectedSingleValue {
                operand: index,
                actual: values.len(),
            });
        }

        Ok(values[0])
    }

    pub fn flattened(&self) -> impl Iterator<Item = ValueId> + '_ {
        self.values.iter().flatten().copied()
    }
}

pub trait TypeConversionPattern: Send + Sync {
    fn name(&self) -> &'static str;

    fn matches(&self, _operation: OperationRef<'_>) -> bool {
        true
    }

    fn rewrite(
        &self,
        operation: OperationId,
        operands: &ConvertedOperands,
        rewriter: &mut ConversionPatternRewriter<'_, '_>,
    ) -> Result<(), RewriteError>;
}

pub(crate) struct RegisteredTypeConversionPattern {
    root: Option<OperationName>,
    benefit: PatternBenefit,
    pattern: Arc<dyn TypeConversionPattern>,
}

impl RegisteredTypeConversionPattern {
    pub(crate) fn pattern(&self) -> &dyn TypeConversionPattern {
        self.pattern.as_ref()
    }
}

#[derive(Default)]
pub(crate) struct TypeConversionPatternStorage {
    patterns: Vec<RegisteredTypeConversionPattern>,
}

impl TypeConversionPatternStorage {
    pub fn add<P>(&mut self, root: Option<OperationName>, benefit: PatternBenefit, pattern: P)
    where
        P: TypeConversionPattern + 'static,
    {
        self.patterns.push(RegisteredTypeConversionPattern {
            root,
            benefit,
            pattern: Arc::new(pattern),
        });
    }

    pub fn candidates(&self, name: &OperationName) -> Vec<&RegisteredTypeConversionPattern> {
        let mut result: Vec<_> = self
            .patterns
            .iter()
            .filter(|candidate| candidate.root.as_ref().map_or(true, |root| root == name))
            .collect();

        result.sort_by_key(|candidate| std::cmp::Reverse(candidate.benefit));

        result
    }
}

pub struct ConversionPatternRewriter<'m, 'c> {
    base: PatternRewriter<'m>,

    converter: &'c TypeConverter,

    mapping: &'c mut ConversionValueMapping,
}

impl<'m, 'c> ConversionPatternRewriter<'m, 'c> {
    pub(crate) fn new(
        base: PatternRewriter<'m>,
        converter: &'c TypeConverter,
        mapping: &'c mut ConversionValueMapping,
    ) -> Self {
        Self {
            base,
            converter,
            mapping,
        }
    }

    pub fn operation(&self, operation: OperationId) -> Option<OperationRef<'_>> {
        self.base.operation(operation)
    }

    pub fn create_operation<I, S>(
        &mut self,
        builder: ir::OperationBuilder,
        operands: I,
    ) -> Result<OperationId, RewriteError>
    where
        I: IntoIterator<Item = ValueId>,
    {
        self.base.create_operation(builder, operands)
    }

    pub fn create_operation_with_successors<I, S>(
        &mut self,
        builder: ir::OperationBuilder,
        operands: I,
        successors: S,
    ) -> Result<OperationId, RewriteError>
    where
        I: IntoIterator<Item = ValueId>,
        S: IntoIterator<Item = BlockSuccessor>,
    {
        self.base
            .create_operation_with_successors(builder, operands, successors)
    }

    pub fn prepare_operands(
        &mut self,
        operation: OperationId,
    ) -> Result<ConvertedOperands, RewriteError> {
        let operands = self
            .base
            .operation(operation)
            .ok_or(IrError::MissingOperation(operation))?
            .operands()
            .to_vec();

        let mut converted = Vec::with_capacity(operands.len());

        for operand in operands {
            converted.push(self.remap_value(operand)?);
        }

        Ok(ConvertedOperands { values: converted })
    }

    pub fn remap_value(&mut self, source: ValueId) -> Result<Vec<ValueId>, RewriteError> {
        if let Some(values) = self.mapping.get(source) {
            return Ok(values.to_vec());
        }

        let source_type = self
            .base
            .value(source)
            .ok_or(IrError::MissingValue(source))?
            .ty()
            .clone();

        let target_types = self
            .converter
            .convert_type(&source_type)
            .map_err(|error| RewriteError::message(error.to_string()))?;

        if target_types.len() == 1 && target_types[0] == source_type {
            self.mapping.map(source, vec![source]);

            return Ok(vec![source]);
        }

        let values = self
            .converter
            .materialize_target(&target_types, &[source], &mut self.base)
            .map_err(|error| RewriteError::message(error.to_string()))?;

        self.mapping.map(source, values.clone());

        Ok(values)
    }

    pub fn replace_operation(
        &mut self,
        operation: OperationId,
        replacements: Vec<Vec<ValueId>>,
    ) -> Result<(), RewriteError> {
        let (results, result_types, nonempty_region) = {
            let operation_ref = self
                .base
                .operation(operation)
                .ok_or(IrError::MissingOperation(operation))?;

            let results = operation_ref.results().to_vec();

            let result_types = results
                .iter()
                .map(|value| {
                    self.base
                        .value(*value)
                        .expect("operation result must exist")
                        .ty()
                        .clone()
                })
                .collect::<Vec<_>>();

            let nonempty_region = operation_ref.regions().iter().copied().find(|region| {
                self.base
                    .module()
                    .region(*region)
                    .map_or(true, |region| !region.is_empty())
            });

            (results, result_types, nonempty_region)
        };

        if let Some(region) = nonempty_region {
            return Err(RewriteError::message(
                TypeConversionError::RegionNotEmptyDuringReplacement { operation, region }
                    .to_string(),
            ));
        }

        if replacements.len() != results.len() {
            return Err(RewriteError::message(
                TypeConversionError::ResultCountMismatch {
                    expected: results.len(),
                    actual: replacements.len(),
                }
                .to_string(),
            ));
        }

        for (index, (source_result, source_type, replacement)) in results
            .iter()
            .zip(result_types.iter())
            .zip(replacements.iter())
            .map(|((a, b), c)| (a, b, c))
            .enumerate()
        {
            let expected = self
                .converter
                .convert_type(source_type)
                .map_err(|error| RewriteError::message(error.to_string()))?;

            let actual = replacement
                .iter()
                .map(|value| {
                    self.base
                        .value(*value)
                        .ok_or(IrError::MissingValue(*value))
                        .map(|value| value.ty().clone())
                })
                .collect::<Result<Vec<Type>, IrError>>()?;

            if expected != actual {
                return Err(RewriteError::message(
                    TypeConversionError::ResultTypeMismatch {
                        result: index,
                        expected,
                        actual,
                    }
                    .to_string(),
                ));
            }

            self.mapping.map(*source_result, replacement.clone());

            let has_uses = self
                .base
                .value(*source_result)
                .expect("source result must exist")
                .has_uses();

            if has_uses {
                let bridge = self
                    .converter
                    .materialize_source(&[source_type.clone()], replacement, &mut self.base)
                    .map_err(|error| RewriteError::message(error.to_string()))?;

                if bridge.len() != 1 {
                    return Err(RewriteError::message(
                        "source materialization must \
                             produce one bridge value",
                    ));
                }

                self.mapping.map(bridge[0], replacement.clone());

                self.base.replace_all_uses(*source_result, bridge[0])?;
            }
        }

        self.base.erase_operation(operation)
    }

    pub fn changed(&self) -> bool {
        self.base.changed()
    }

    pub fn move_region_contents(
        &mut self,
        source: RegionId,
        target: RegionId,
    ) -> Result<(), RewriteError> {
        self.base.move_region_contents(source, target)
    }

    pub fn apply_signature_conversion(
        &mut self,
        block: BlockId,
        signature: &SignatureConversion,
    ) -> Result<SignatureConversionReport, RewriteError> {
        apply_signature_conversion(
            &mut self.base,
            block,
            signature,
            self.converter,
            self.mapping,
        )
        .map_err(|error| RewriteError::message(error.to_string()))
    }

    pub fn convert_region_entry_signature(
        &mut self,
        region: RegionId,
    ) -> Result<SignatureConversionReport, RewriteError> {
        convert_region_entry_signature(&mut self.base, region, self.converter, self.mapping)
            .map_err(|error| RewriteError::message(error.to_string()))
    }
}
