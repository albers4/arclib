// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use ir::{Module, OperationId, PatternRewriter, RewriteError, apply_patterns_to_operation};

use super::{
    ConversionError, ConversionPatternRewriter, ConversionPatternSet, ConversionTarget,
    ConversionValueMapping, Legality, TypeConverter,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionMode {
    Partial,
    Full,
}

#[derive(Debug, Clone)]
pub struct ConversionConfig {
    pub max_rewrites: usize,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            max_rewrites: 100_000,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionReport {
    pub rewrites: usize,
    pub operations_examined: usize,
    pub remaining_unknown_operations: usize,
    pub block_arguments_converted: usize,
}

pub fn apply_partial_conversion(
    module: &mut Module,
    target: &ConversionTarget,
    patterns: &ConversionPatternSet,
) -> Result<ConversionReport, ConversionError> {
    apply_conversion(
        module,
        target,
        patterns,
        ConversionMode::Partial,
        &ConversionConfig::default(),
    )
}

pub fn apply_full_conversion(
    module: &mut Module,
    target: &ConversionTarget,
    patterns: &ConversionPatternSet,
) -> Result<ConversionReport, ConversionError> {
    apply_conversion(
        module,
        target,
        patterns,
        ConversionMode::Full,
        &ConversionConfig::default(),
    )
}

pub fn apply_conversion(
    module: &mut Module,
    target: &ConversionTarget,
    patterns: &ConversionPatternSet,
    mode: ConversionMode,
    config: &ConversionConfig,
) -> Result<ConversionReport, ConversionError> {
    apply_conversion_impl(module, target, patterns, None, None, mode, config)
}

pub fn apply_conversion_with_types(
    module: &mut Module,
    target: &ConversionTarget,
    patterns: &ConversionPatternSet,
    converter: &TypeConverter,
    mode: ConversionMode,
    config: &ConversionConfig,
) -> Result<ConversionReport, ConversionError> {
    let mut mapping = ConversionValueMapping::new();

    apply_conversion_impl(
        module,
        target,
        patterns,
        Some(converter),
        Some(&mut mapping),
        mode,
        config,
    )
}

fn apply_conversion_impl(
    module: &mut Module,
    target: &ConversionTarget,
    patterns: &ConversionPatternSet,
    type_converter: Option<&TypeConverter>,
    mut value_mapping: Option<&mut ConversionValueMapping>,
    mode: ConversionMode,
    config: &ConversionConfig,
) -> Result<ConversionReport, ConversionError> {
    let mut rewrites = 0;
    let mut operations_examined = 0;

    loop {
        let mut candidate = None;

        for operation_id in module.operations() {
            let Some(operation) = module.operation(operation_id) else {
                continue;
            };

            operations_examined += 1;

            let name = operation.name().clone();
            let legality = target.classify(operation)?;

            let requires_conversion = matches!(
                (mode, legality),
                (_, Legality::Illegal) | (ConversionMode::Full, Legality::Unknown)
            );

            if requires_conversion {
                candidate = Some((operation_id, name, legality));
                break;
            }
        }

        let Some((operation, name, legality)) = candidate else {
            break;
        };

        if rewrites >= config.max_rewrites {
            return Err(ConversionError::RewriteLimitExceeded {
                limit: config.max_rewrites,
            });
        }

        let mut rewritten = false;

        if let (Some(converter), Some(mapping)) = (type_converter, value_mapping.as_deref_mut()) {
            rewritten =
                apply_typed_patterns_to_operation(module, operation, patterns, converter, mapping)?;
        }

        if !rewritten {
            rewritten =
                apply_patterns_to_operation(module, operation, patterns.rewrite_patterns())?
                    .was_rewritten();
        }

        if !rewritten {
            return Err(ConversionError::UnlegalizableOperation {
                operation,
                name,
                legality,
            });
        }

        rewrites += 1;
    }

    let mut remaining_unknown_operations = 0;

    for operation_id in module.operations() {
        let Some(operation) = module.operation(operation_id) else {
            continue;
        };

        if target.classify(operation)? == Legality::Unknown {
            remaining_unknown_operations += 1;
        }
    }

    let block_arguments_converted = value_mapping
        .as_deref()
        .map(ConversionValueMapping::block_arguments_converted)
        .unwrap_or(0);

    Ok(ConversionReport {
        rewrites,
        operations_examined,
        remaining_unknown_operations,
        block_arguments_converted,
    })
}

fn apply_typed_patterns_to_operation(
    module: &mut Module,
    operation: OperationId,
    patterns: &ConversionPatternSet,
    converter: &TypeConverter,
    mapping: &mut ConversionValueMapping,
) -> Result<bool, ConversionError> {
    let name = module
        .operation(operation)
        .expect("conversion candidate must still exist")
        .name()
        .clone();

    for registered in patterns.typed_patterns().candidates(&name) {
        let pattern = registered.pattern();

        let matches = {
            let operation_ref = module
                .operation(operation)
                .expect("conversion candidate must still exist");

            pattern.matches(operation_ref)
        };

        if !matches {
            continue;
        }

        let base = PatternRewriter::new(module, operation)?;
        let mut rewriter = ConversionPatternRewriter::new(base, converter, mapping);

        let operands = rewriter.prepare_operands(operation)?;

        pattern.rewrite(operation, &operands, &mut rewriter)?;

        if !rewriter.changed() {
            return Err(ConversionError::Rewrite(
                RewriteError::PatternReportedRewriteWithoutChange {
                    pattern: pattern.name(),
                    operation,
                },
            ));
        }

        return Ok(true);
    }

    Ok(false)
}
