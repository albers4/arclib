// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use ir::{BlockId, InsertionPoint, OperationId, PatternRewriter, RegionId, Type, ValueId};

use super::{ConversionValueMapping, TypeConversionError, TypeConverter};

#[derive(Debug, Clone, Copy)]
struct IncomingEdge {
    operation: OperationId,
    successor_index: usize,
}

fn incoming_edges(rewriter: &PatternRewriter<'_>, block: BlockId) -> Vec<IncomingEdge> {
    let mut result = Vec::new();

    for operation_id in rewriter.module().operations() {
        let Some(operation) = rewriter.operation(operation_id) else {
            continue;
        };

        for (successor_index, successor) in operation.successors().iter().copied().enumerate() {
            if successor == block {
                result.push(IncomingEdge {
                    operation: operation_id,

                    successor_index,
                });
            }
        }
    }

    result
}

#[derive(Debug, Clone)]
pub struct SignatureConversion {
    original_count: usize,

    replacements: Vec<Option<Vec<Type>>>,
}

impl SignatureConversion {
    pub fn new(original_count: usize) -> Self {
        Self {
            original_count,
            replacements: vec![None; original_count],
        }
    }

    pub fn from_types(
        source_types: &[Type],
        converter: &TypeConverter,
    ) -> Result<Self, TypeConversionError> {
        let mut conversion = Self::new(source_types.len());

        for (index, source_type) in source_types.iter().enumerate() {
            conversion.convert_input(index, converter.convert_type(source_type)?)?;
        }

        Ok(conversion)
    }

    pub fn convert_input(
        &mut self,
        index: usize,
        target_types: impl IntoIterator<Item = Type>,
    ) -> Result<&mut Self, TypeConversionError> {
        let slot = self.replacements.get_mut(index).ok_or(
            TypeConversionError::SignatureArgumentOutOfBounds {
                index,
                argument_count: self.original_count,
            },
        )?;

        *slot = Some(target_types.into_iter().collect());

        Ok(self)
    }

    pub fn drop_input(&mut self, index: usize) -> Result<&mut Self, TypeConversionError> {
        self.convert_input(index, std::iter::empty())
    }

    pub fn original_count(&self) -> usize {
        self.original_count
    }

    pub fn converted_types(&self, index: usize) -> Result<&[Type], TypeConversionError> {
        let replacement = self.replacements.get(index).ok_or(
            TypeConversionError::SignatureArgumentOutOfBounds {
                index,
                argument_count: self.original_count,
            },
        )?;

        replacement
            .as_deref()
            .ok_or(TypeConversionError::IncompleteSignatureConversion { index })
    }

    pub fn flattened_types(&self) -> Result<Vec<Type>, TypeConversionError> {
        let mut result = Vec::new();

        for index in 0..self.original_count {
            result.extend_from_slice(self.converted_types(index)?);
        }

        Ok(result)
    }

    fn validate_for(&self, source_types: &[Type]) -> Result<(), TypeConversionError> {
        if source_types.len() != self.original_count {
            return Err(TypeConversionError::SignatureArgumentCountMismatch {
                expected: self.original_count,
                actual: source_types.len(),
            });
        }

        for index in 0..self.original_count {
            self.converted_types(index)?;
        }

        Ok(())
    }

    fn is_identity(&self, source_types: &[Type]) -> Result<bool, TypeConversionError> {
        self.validate_for(source_types)?;

        Ok(source_types.iter().enumerate().all(|(index, source)| {
            let converted = self.replacements[index]
                .as_ref()
                .expect("validated signature entry");

            converted.len() == 1 && converted[0] == *source
        }))
    }
}

#[derive(Debug, Clone)]
pub struct SignatureConversionReport {
    block: BlockId,

    original_argument_count: usize,

    new_arguments: Vec<ValueId>,

    converted_arguments: usize,
}

impl SignatureConversionReport {
    pub fn block(&self) -> BlockId {
        self.block
    }

    pub fn original_argument_count(&self) -> usize {
        self.original_argument_count
    }

    pub fn new_arguments(&self) -> &[ValueId] {
        &self.new_arguments
    }

    pub fn converted_arguments(&self) -> usize {
        self.converted_arguments
    }

    pub fn changed(&self) -> bool {
        self.converted_arguments > 0
    }
}

pub fn apply_signature_conversion(
    rewriter: &mut PatternRewriter<'_>,

    block: BlockId,

    signature: &SignatureConversion,

    converter: &TypeConverter,

    mapping: &mut ConversionValueMapping,
) -> Result<SignatureConversionReport, TypeConversionError> {
    let original_arguments = rewriter
        .module()
        .block(block)
        .ok_or(TypeConversionError::MissingBlock(block))?
        .arguments()
        .to_vec();

    let source_types = original_arguments
        .iter()
        .map(|argument| {
            rewriter
                .value(*argument)
                .ok_or(TypeConversionError::MissingValue(*argument))
                .map(|value| value.ty().clone())
        })
        .collect::<Result<Vec<_>, _>>()?;

    signature.validate_for(&source_types)?;

    if signature.is_identity(&source_types)? {
        for argument in &original_arguments {
            mapping.map(*argument, vec![*argument]);
        }

        return Ok(SignatureConversionReport {
            block,

            original_argument_count: original_arguments.len(),

            new_arguments: original_arguments,

            converted_arguments: 0,
        });
    }

    let incoming = incoming_edges(rewriter, block);

    let mut converted_edges = Vec::with_capacity(incoming.len());

    for edge in &incoming {
        let original_edge_operands = rewriter
            .operation(edge.operation)
            .and_then(|operation| {
                operation
                    .successor_operands(edge.successor_index)
                    .map(<[_]>::to_vec)
            })
            .ok_or_else(|| TypeConversionError::MaterializationFailed {
                kind: "successor signature",
                message: format!(
                    "missing successor {} \
                                on operation {:?}",
                    edge.successor_index, edge.operation,
                ),
            })?;

        if original_edge_operands.len() != source_types.len() {
            return Err(TypeConversionError::SignatureArgumentCountMismatch {
                expected: source_types.len(),
                actual: original_edge_operands.len(),
            });
        }

        let mut converted = Vec::new();

        rewriter.with_insertion_point(InsertionPoint::Before(edge.operation), |rewriter| {
            for (index, source_value) in original_edge_operands.iter().copied().enumerate() {
                let requested = signature.converted_types(index)?;

                let values =
                    remap_successor_value(source_value, requested, converter, mapping, rewriter)?;

                converted.extend(values);
            }

            Ok::<(), TypeConversionError>(())
        })?;

        converted_edges.push((*edge, converted));
    }

    // Recreate every argument, including
    // identity arguments. This preserves
    // flattened order after 1-to-N expansion.
    let mut new_groups = Vec::with_capacity(original_arguments.len());

    let mut new_arguments = Vec::new();

    let mut converted_arguments = 0;

    for (index, source_type) in source_types.iter().enumerate() {
        let target_types = signature.converted_types(index)?;

        if target_types.len() != 1 || target_types[0] != *source_type {
            converted_arguments += 1;
        }

        let mut group = Vec::with_capacity(target_types.len());

        for target_type in target_types {
            let argument = rewriter
                .append_block_argument(block, target_type.clone())
                .map_err(|error| TypeConversionError::MaterializationFailed {
                    kind: "block signature",
                    message: error.to_string(),
                })?;

            mapping.map(argument, vec![argument]);

            group.push(argument);
            new_arguments.push(argument);
        }

        new_groups.push(group);
    }

    rewriter.with_insertion_point(InsertionPoint::Start(block), |rewriter| {
        for (index, old_argument) in original_arguments.iter().copied().enumerate() {
            let group = &new_groups[index];

            mapping.map(old_argument, group.clone());

            let has_uses = rewriter
                .value(old_argument)
                .ok_or(TypeConversionError::MissingValue(old_argument))?
                .has_uses();

            if !has_uses {
                continue;
            }

            if group.is_empty() {
                return Err(TypeConversionError::CannotDropUsedArgument {
                    block,
                    index,
                    value: old_argument,
                });
            }

            let replacement = if group.len() == 1 {
                let target_type = rewriter
                    .value(group[0])
                    .ok_or(TypeConversionError::MissingValue(group[0]))?
                    .ty()
                    .clone();

                if target_type == source_types[index] {
                    group[0]
                } else {
                    materialize_original_argument(
                        converter,
                        &source_types[index],
                        group,
                        rewriter,
                        mapping,
                    )?
                }
            } else {
                materialize_original_argument(
                    converter,
                    &source_types[index],
                    group,
                    rewriter,
                    mapping,
                )?
            };

            rewriter
                .replace_all_uses(old_argument, replacement)
                .map_err(|error| TypeConversionError::MaterializationFailed {
                    kind: "block signature",
                    message: error.to_string(),
                })?;
        }

        // Reverse order prevents
        // shifting argument indices.
        for index in (0..original_arguments.len()).rev() {
            rewriter
                .erase_block_argument(block, index)
                .map_err(|error| TypeConversionError::MaterializationFailed {
                    kind: "block signature",
                    message: error.to_string(),
                })?;
        }

        Ok(())
    })?;

    mapping.record_block_arguments_converted(converted_arguments);

    for (edge, operands) in converted_edges {
        rewriter
            .set_successor_operands(edge.operation, edge.successor_index, operands)
            .map_err(|error| TypeConversionError::MaterializationFailed {
                kind: "successor signature",
                message: error.to_string(),
            })?;
    }

    Ok(SignatureConversionReport {
        block,

        original_argument_count: original_arguments.len(),

        new_arguments,

        converted_arguments,
    })
}

pub fn convert_region_entry_signature(
    rewriter: &mut PatternRewriter<'_>,

    region: RegionId,

    converter: &TypeConverter,

    mapping: &mut ConversionValueMapping,
) -> Result<SignatureConversionReport, TypeConversionError> {
    let entry = rewriter
        .module()
        .region(region)
        .ok_or(TypeConversionError::MissingRegion(region))?
        .entry_block()
        .ok_or(TypeConversionError::RegionHasNoEntryBlock(region))?;

    let source_types = rewriter
        .module()
        .block(entry)
        .ok_or(TypeConversionError::MissingBlock(entry))?
        .arguments()
        .iter()
        .map(|argument| {
            rewriter
                .value(*argument)
                .ok_or(TypeConversionError::MissingValue(*argument))
                .map(|value| value.ty().clone())
        })
        .collect::<Result<Vec<_>, _>>()?;

    let signature = SignatureConversion::from_types(&source_types, converter)?;

    apply_signature_conversion(rewriter, entry, &signature, converter, mapping)
}

fn materialize_original_argument(
    converter: &TypeConverter,

    source_type: &Type,

    converted_arguments: &[ValueId],

    rewriter: &mut PatternRewriter<'_>,

    mapping: &mut ConversionValueMapping,
) -> Result<ValueId, TypeConversionError> {
    let materialized =
        converter.materialize_source(&[source_type.clone()], converted_arguments, rewriter)?;

    if materialized.len() != 1 {
        return Err(TypeConversionError::MaterializationCountMismatch {
            kind: "block signature source",
            expected: 1,
            actual: materialized.len(),
        });
    }

    mapping.map(materialized[0], converted_arguments.to_vec());

    Ok(materialized[0])
}

//fn predecessor_operations(
//    rewriter:
//        &PatternRewriter<'_>,
//
//    block: BlockId,
//) -> Vec<OperationId> {
//    rewriter
//        .module()
//        .operations()
//        .into_iter()
//        .filter(|operation| {
//            rewriter
//                .operation(*operation)
//                .is_some_and(
//                    |operation| {
//                        operation
//                            .successors()
//                            .contains(
//                                &block,
//                            )
//                    },
//                )
//        })
//        .collect()
//}

fn remap_successor_value(
    source: ValueId,
    requested: &[Type],
    converter: &TypeConverter,
    mapping: &mut ConversionValueMapping,
    rewriter: &mut PatternRewriter<'_>,
) -> Result<Vec<ValueId>, TypeConversionError> {
    if let Some(mapped) = mapping.get(source) {
        let mapped_types = mapped
            .iter()
            .map(|value| {
                rewriter
                    .value(*value)
                    .ok_or(TypeConversionError::MissingValue(*value))
                    .map(|value| value.ty().clone())
            })
            .collect::<Result<Vec<_>, _>>()?;

        if mapped_types == requested {
            return Ok(mapped.to_vec());
        }
    }

    let source_type = rewriter
        .value(source)
        .ok_or(TypeConversionError::MissingValue(source))?
        .ty()
        .clone();

    if requested.len() == 1 && requested[0] == source_type {
        mapping.map(source, vec![source]);

        return Ok(vec![source]);
    }

    let values = converter.materialize_target(requested, &[source], rewriter)?;

    mapping.map(source, values.clone());

    Ok(values)
}
