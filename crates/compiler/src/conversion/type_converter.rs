// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use ir::{OperationBuilder, PatternRewriter, Type, ValueId};

use super::TypeConversionError;

#[derive(Debug)]
pub enum TypeConversionRuleResult {
    NotApplicable,

    Converted(Vec<Type>),

    Failed(String),
}

type ConversionRule = Arc<dyn Fn(&Type) -> TypeConversionRuleResult + Send + Sync>;

type Materialization = Arc<
    dyn for<'a> Fn(
            &[Type],
            &[ValueId],
            &mut PatternRewriter<'a>,
        ) -> Result<Option<Vec<ValueId>>, String>
        + Send
        + Sync,
>;

pub struct TypeConverter {
    rules: Vec<ConversionRule>,

    source_materializations: Vec<Materialization>,

    target_materializations: Vec<Materialization>,

    identity_fallback: bool,
}

impl TypeConverter {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),

            source_materializations: Vec::new(),

            target_materializations: Vec::new(),

            identity_fallback: false,
        }
    }

    pub fn enable_identity_fallback(&mut self) -> &mut Self {
        self.identity_fallback = true;
        self
    }

    pub fn add_conversion<F>(&mut self, conversion: F) -> &mut Self
    where
        F: Fn(&Type) -> TypeConversionRuleResult + Send + Sync + 'static,
    {
        self.rules.push(Arc::new(conversion));

        self
    }

    pub fn add_source_materialization<F>(&mut self, materialization: F) -> &mut Self
    where
        F: for<'a> Fn(
                &[Type],
                &[ValueId],
                &mut PatternRewriter<'a>,
            ) -> Result<Option<Vec<ValueId>>, String>
            + Send
            + Sync
            + 'static,
    {
        self.source_materializations.push(Arc::new(materialization));

        self
    }

    pub fn add_target_materialization<F>(&mut self, materialization: F) -> &mut Self
    where
        F: for<'a> Fn(
                &[Type],
                &[ValueId],
                &mut PatternRewriter<'a>,
            ) -> Result<Option<Vec<ValueId>>, String>
            + Send
            + Sync
            + 'static,
    {
        self.target_materializations.push(Arc::new(materialization));

        self
    }

    pub fn add_unrealized_cast_materializations(&mut self) -> &mut Self {
        self.add_source_materialization(unrealized_cast);

        self.add_target_materialization(unrealized_cast);

        self
    }

    pub fn convert_type(&self, source: &Type) -> Result<Vec<Type>, TypeConversionError> {
        // Newer rules have higher priority.
        for rule in self.rules.iter().rev() {
            match rule(source) {
                TypeConversionRuleResult::NotApplicable => {}

                TypeConversionRuleResult::Converted(types) => {
                    return Ok(types);
                }

                TypeConversionRuleResult::Failed(message) => {
                    return Err(TypeConversionError::RuleFailed {
                        source: source.clone(),
                        message,
                    });
                }
            }
        }

        if self.identity_fallback {
            return Ok(vec![source.clone()]);
        }

        Err(TypeConversionError::NoConversion {
            source: source.clone(),
        })
    }

    pub fn is_legal(&self, source: &Type) -> Result<bool, TypeConversionError> {
        let converted = self.convert_type(source)?;

        Ok(converted.len() == 1 && converted[0] == *source)
    }

    pub fn materialize_source(
        &self,
        requested: &[Type],
        inputs: &[ValueId],
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<Vec<ValueId>, TypeConversionError> {
        self.materialize(
            "source",
            &self.source_materializations,
            requested,
            inputs,
            rewriter,
        )
    }

    pub fn materialize_target(
        &self,
        requested: &[Type],
        inputs: &[ValueId],
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<Vec<ValueId>, TypeConversionError> {
        self.materialize(
            "target",
            &self.target_materializations,
            requested,
            inputs,
            rewriter,
        )
    }

    fn materialize(
        &self,
        kind: &'static str,
        materializations: &[Materialization],
        requested: &[Type],
        inputs: &[ValueId],
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<Vec<ValueId>, TypeConversionError> {
        if requested.is_empty() {
            return Ok(Vec::new());
        }

        for materialization in materializations.iter().rev() {
            let Some(values) = materialization(requested, inputs, rewriter)
                .map_err(|message| TypeConversionError::MaterializationFailed { kind, message })?
            else {
                continue;
            };

            if values.len() != requested.len() {
                return Err(TypeConversionError::MaterializationCountMismatch {
                    kind,
                    expected: requested.len(),
                    actual: values.len(),
                });
            }

            for (value, expected_type) in values.iter().zip(requested) {
                let actual_type = {
                    let value_ref = rewriter.value(*value).ok_or_else(|| {
                        TypeConversionError::MaterializationFailed {
                            kind,
                            message: format!(
                                "materialization produced \
                                         missing value {value:?}"
                            ),
                        }
                    })?;

                    value_ref.ty().clone()
                };

                if &actual_type != expected_type {
                    return Err(TypeConversionError::MaterializationTypeMismatch {
                        kind,
                        value: *value,
                        expected: expected_type.clone(),
                        actual: actual_type,
                    });
                }
            }

            return Ok(values);
        }

        Err(TypeConversionError::NoMaterialization {
            kind,
            requested: requested.to_vec(),
        })
    }
}

impl Default for TypeConverter {
    fn default() -> Self {
        Self::new()
    }
}

fn unrealized_cast(
    requested: &[Type],
    inputs: &[ValueId],
    rewriter: &mut PatternRewriter<'_>,
) -> Result<Option<Vec<ValueId>>, String> {
    let input_types: Vec<Type> = inputs
        .iter()
        .map(|value| {
            rewriter
                .value(*value)
                .map(|value| value.ty().clone())
                .ok_or_else(|| format!("missing input value {value:?}"))
        })
        .collect::<Result<_, _>>()?;

    if input_types == requested {
        return Ok(Some(inputs.to_vec()));
    }

    let cast = rewriter
        .create_operation(
            OperationBuilder::new("builtin.unrealized_conversion_cast")
                .results(requested.iter().cloned()),
            inputs.iter().copied(),
        )
        .map_err(|error| error.to_string())?;

    let results = rewriter
        .operation(cast)
        .ok_or_else(|| "created cast operation disappeared".to_owned())?
        .results()
        .to_vec();

    Ok(Some(results))
}
