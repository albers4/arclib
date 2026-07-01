// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use ir::{
    Attribute, DialectDescriptor, DialectRegistry, DialectRegistryError, OperationBuilder,
    OperationDescriptor, OperationRef, Pure,
};

use super::{
    DifferentialOperatorInterface, ExpressionType, LinearOperatorInterface,
    OperatorExpressionInterface,
};

pub const OPERATOR_DIALECT: &str = "operator";
pub const OPERATOR_STAGE: &str = "operator";

pub const FIELD_OPERATION: &str = "operator.field";
pub const TIME_DERIVATIVE_OPERATION: &str = "operator.time_derivative";
pub const GRADIENT_OPERATION: &str = "operator.gradient";
pub const DIVERGENCE_OPERATION: &str = "operator.divergence";
pub const LAPLACIAN_OPERATION: &str = "operator.laplacian";
pub const SCALE_OPERATION: &str = "operator.scale";
pub const ADD_OPERATION: &str = "operator.add";
pub const SUBTRACT_OPERATION: &str = "operator.subtract";

pub const ELEMENT_ATTRIBUTE: &str = "operator.element";
pub const RANK_ATTRIBUTE: &str = "operator.rank";
pub const DIMENSION_ATTRIBUTE_PREFIX: &str = "operator.dimension.";
pub const FACTOR_ATTRIBUTE: &str = "operator.factor";

pub struct FieldOp;
impl FieldOp {
    pub fn builder(result: &ExpressionType) -> OperationBuilder {
        expression_builder(OperationBuilder::new(FIELD_OPERATION), result)
    }
}

pub struct TimeDerivativeOp;
impl TimeDerivativeOp {
    pub fn builder(result: &ExpressionType) -> OperationBuilder {
        expression_builder(OperationBuilder::new(TIME_DERIVATIVE_OPERATION), result)
    }
}

pub struct GradientOp;
impl GradientOp {
    pub fn builder(result: &ExpressionType) -> OperationBuilder {
        expression_builder(OperationBuilder::new(GRADIENT_OPERATION), result)
    }
}

pub struct DivergenceOp;
impl DivergenceOp {
    pub fn builder(result: &ExpressionType) -> OperationBuilder {
        expression_builder(OperationBuilder::new(DIVERGENCE_OPERATION), result)
    }
}

pub struct LaplacianOp;
impl LaplacianOp {
    pub fn builder(result: &ExpressionType) -> OperationBuilder {
        expression_builder(OperationBuilder::new(LAPLACIAN_OPERATION), result)
    }
}

pub struct ScaleOp;
impl ScaleOp {
    pub fn builder(result: &ExpressionType, factor: f64) -> OperationBuilder {
        expression_builder(
            OperationBuilder::new(SCALE_OPERATION)
                .attribute(FACTOR_ATTRIBUTE, Attribute::Float(factor)),
            result,
        )
    }
}

pub struct AddOp;
impl AddOp {
    pub fn builder(result: &ExpressionType) -> OperationBuilder {
        expression_builder(OperationBuilder::new(ADD_OPERATION), result)
    }
}

pub struct SubtractOp;
impl SubtractOp {
    pub fn builder(result: &ExpressionType) -> OperationBuilder {
        expression_builder(OperationBuilder::new(SUBTRACT_OPERATION), result)
    }
}

pub fn register_operator_dialect(
    registry: &mut DialectRegistry,
) -> Result<(), DialectRegistryError> {
    let mut dialect = DialectDescriptor::new(OPERATOR_DIALECT);

    dialect.register_operation(
        OperationDescriptor::new(FIELD_OPERATION)
            .with_trait::<Pure>()
            .with_interface(OperatorExpressionInterface)
            .with_verifier(|operation: OperationRef<'_>| verify_expression(operation, 1)),
    )?;
    dialect.register_operation(
        OperationDescriptor::new(TIME_DERIVATIVE_OPERATION)
            .with_trait::<Pure>()
            .with_interface(OperatorExpressionInterface)
            .with_interface(DifferentialOperatorInterface)
            .with_interface(LinearOperatorInterface)
            .with_verifier(|operation: OperationRef<'_>| verify_expression(operation, 1)),
    )?;
    for name in [
        GRADIENT_OPERATION,
        DIVERGENCE_OPERATION,
        LAPLACIAN_OPERATION,
    ] {
        dialect.register_operation(
            OperationDescriptor::new(name)
                .with_trait::<Pure>()
                .with_interface(OperatorExpressionInterface)
                .with_interface(DifferentialOperatorInterface)
                .with_interface(LinearOperatorInterface)
                .with_verifier(|operation: OperationRef<'_>| verify_expression(operation, 1)),
        )?;
    }
    dialect.register_operation(
        OperationDescriptor::new(SCALE_OPERATION)
            .with_trait::<Pure>()
            .with_interface(OperatorExpressionInterface)
            .with_interface(LinearOperatorInterface)
            .with_verifier(verify_scale),
    )?;
    for name in [ADD_OPERATION, SUBTRACT_OPERATION] {
        dialect.register_operation(
            OperationDescriptor::new(name)
                .with_trait::<Pure>()
                .with_interface(OperatorExpressionInterface)
                .with_interface(LinearOperatorInterface)
                .with_verifier(|operation: OperationRef<'_>| verify_expression(operation, 2)),
        )?;
    }

    registry.register_dialect(dialect)
}

fn expression_builder(mut builder: OperationBuilder, result: &ExpressionType) -> OperationBuilder {
    builder = builder
        .attribute(ELEMENT_ATTRIBUTE, Attribute::string(result.element()))
        .attribute(
            RANK_ATTRIBUTE,
            Attribute::Integer(i64::try_from(result.rank()).expect("rank exceeds i64")),
        )
        .result(result.ir_type());
    for (index, dimension) in result.dimensions().iter().copied().enumerate() {
        builder = builder.attribute(
            format!("{DIMENSION_ATTRIBUTE_PREFIX}{index}"),
            Attribute::Integer(i64::from(dimension)),
        );
    }
    builder
}

fn verify_scale(operation: OperationRef<'_>) -> Result<(), String> {
    verify_expression(operation, 1)?;
    match operation.attribute(FACTOR_ATTRIBUTE) {
        Some(Attribute::Float(value)) if value.is_finite() => Ok(()),
        Some(Attribute::Float(_)) => Err("operator.factor must be finite".into()),
        Some(_) => Err("operator.factor must be a float".into()),
        None => Err("operator.scale requires operator.factor".into()),
    }
}

fn verify_expression(operation: OperationRef<'_>, operand_count: usize) -> Result<(), String> {
    if operation.operands().len() != operand_count {
        return Err(format!(
            "{} requires {operand_count} operands",
            operation.name()
        ));
    }
    if operation.results().len() != 1 {
        return Err(format!("{} requires one result", operation.name()));
    }
    if !operation.regions().is_empty() || !operation.successors().is_empty() {
        return Err(format!("{} must be a leaf operation", operation.name()));
    }
    let expected = expression_type_from_attributes(operation)?;
    if operation.result_type(0) != Some(&expected.ir_type()) {
        return Err(format!(
            "{} has an invalid expression result type",
            operation.name()
        ));
    }
    Ok(())
}

fn expression_type_from_attributes(operation: OperationRef<'_>) -> Result<ExpressionType, String> {
    let element = operation
        .attribute(ELEMENT_ATTRIBUTE)
        .and_then(Attribute::as_str)
        .ok_or_else(|| "operator.element must be a string".to_owned())?;
    let rank = match operation.attribute(RANK_ATTRIBUTE) {
        Some(Attribute::Integer(value)) if *value >= 0 => {
            usize::try_from(*value).map_err(|_| "operator.rank is too large".to_owned())?
        }
        _ => return Err("operator.rank must be a non-negative integer".into()),
    };
    let mut dimensions = Vec::with_capacity(rank);
    for index in 0..rank {
        let name = format!("{DIMENSION_ATTRIBUTE_PREFIX}{index}");
        match operation.attribute(&name) {
            Some(Attribute::Integer(value)) if *value > 0 => dimensions.push(
                u32::try_from(*value).map_err(|_| format!("attribute '{name}' is too large"))?,
            ),
            _ => return Err(format!("attribute '{name}' must be a positive integer")),
        }
    }
    Ok(ExpressionType::new(element, dimensions))
}
