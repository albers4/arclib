// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use ir::{
    Attribute, DialectDescriptor, DialectRegistry, DialectRegistryError, OperationBuilder,
    OperationDescriptor, OperationRef, Signedness, Type,
};
use method::DiscreteMethodInterface;

use super::{
    FvmBufferElement, FvmBufferType, FvmMeshViewInterface, FvmProgramInterface,
    FvmStencilInterface, FvmTimeIntegratorInterface,
};

pub const FVM_DIALECT: &str = "fvm";
pub const FVM_STAGE: &str = "method.fvm";

pub const PROGRAM_OPERATION: &str = "fvm.program";
pub const PREPARE_MESH_OPERATION: &str = "fvm.prepare_mesh";
pub const CONSTANT_OPERATION: &str = "fvm.constant";
pub const LAPLACIAN_OPERATION: &str = "fvm.laplacian";
pub const SCALE_OPERATION: &str = "fvm.scale";
pub const EXPLICIT_EULER_OPERATION: &str = "fvm.explicit_euler";

pub const DIMENSION_ATTRIBUTE: &str = "fvm.dimension";
pub const SCHEME_ATTRIBUTE: &str = "fvm.scheme";
pub const VALUE_ATTRIBUTE: &str = "fvm.value";

pub const PREPARE_OWNER_RESULT: usize = 0;
pub const PREPARE_NEIGHBOUR_RESULT: usize = 1;
pub const PREPARE_FACE_COEFFICIENT_RESULT: usize = 2;
pub const PREPARE_CELL_VOLUME_RESULT: usize = 3;
pub const PREPARE_CELL_COUNT_RESULT: usize = 4;
pub const PREPARE_FACE_COUNT_RESULT: usize = 5;

pub const LAPLACIAN_FIELD_OPERAND: usize = 0;
pub const LAPLACIAN_OWNER_OPERAND: usize = 1;
pub const LAPLACIAN_NEIGHBOUR_OPERAND: usize = 2;
pub const LAPLACIAN_FACE_COEFFICIENT_OPERAND: usize = 3;
pub const LAPLACIAN_CELL_VOLUME_OPERAND: usize = 4;
pub const LAPLACIAN_DESTINATION_OPERAND: usize = 5;
pub const LAPLACIAN_CELL_COUNT_OPERAND: usize = 6;
pub const LAPLACIAN_FACE_COUNT_OPERAND: usize = 7;

pub const SCALE_FIELD_OPERAND: usize = 0;
pub const SCALE_FACTOR_OPERAND: usize = 1;
pub const SCALE_DESTINATION_OPERAND: usize = 2;
pub const SCALE_CELL_COUNT_OPERAND: usize = 3;

pub const EULER_STATE_OPERAND: usize = 0;
pub const EULER_RHS_OPERAND: usize = 1;
pub const EULER_TIME_STEP_OPERAND: usize = 2;
pub const EULER_DESTINATION_OPERAND: usize = 3;
pub const EULER_CELL_COUNT_OPERAND: usize = 4;

pub struct FvmProgramOp;
impl FvmProgramOp {
    pub fn builder(dimension: u32) -> OperationBuilder {
        OperationBuilder::new(PROGRAM_OPERATION)
            .symbol_table()
            .region()
            .attribute(
                DIMENSION_ATTRIBUTE,
                Attribute::Integer(i64::from(dimension)),
            )
    }
}

pub struct PrepareMeshOp;
impl PrepareMeshOp {
    pub fn builder(dimension: u32) -> OperationBuilder {
        OperationBuilder::new(PREPARE_MESH_OPERATION)
            .attribute(
                DIMENSION_ATTRIBUTE,
                Attribute::Integer(i64::from(dimension)),
            )
            .results([
                FvmBufferType::new(FvmBufferElement::I32).ir_type(),
                FvmBufferType::new(FvmBufferElement::I32).ir_type(),
                FvmBufferType::new(FvmBufferElement::F64).ir_type(),
                FvmBufferType::new(FvmBufferElement::F64).ir_type(),
                Type::integer(64, Signedness::Signless),
                Type::integer(64, Signedness::Signless),
            ])
    }
}

pub struct ConstantOp;
impl ConstantOp {
    pub fn builder(value: f64) -> OperationBuilder {
        OperationBuilder::new(CONSTANT_OPERATION)
            .attribute(VALUE_ATTRIBUTE, Attribute::Float(value))
            .result(Type::f64())
    }
}

pub struct LaplacianOp;
impl LaplacianOp {
    pub fn builder(result_type: Type) -> OperationBuilder {
        OperationBuilder::new(LAPLACIAN_OPERATION)
            .attribute(SCHEME_ATTRIBUTE, Attribute::string("gauss-orthogonal"))
            .result(result_type)
    }
}

pub struct ScaleOp;
impl ScaleOp {
    pub fn builder(result_type: Type) -> OperationBuilder {
        OperationBuilder::new(SCALE_OPERATION)
            .attribute(SCHEME_ATTRIBUTE, Attribute::string("pointwise"))
            .result(result_type)
    }
}

pub struct ExplicitEulerOp;
impl ExplicitEulerOp {
    pub fn builder(result_type: Type) -> OperationBuilder {
        OperationBuilder::new(EXPLICIT_EULER_OPERATION)
            .attribute(SCHEME_ATTRIBUTE, Attribute::string("explicit-euler"))
            .result(result_type)
    }
}

pub fn register_fvm_dialect(registry: &mut DialectRegistry) -> Result<(), DialectRegistryError> {
    let mut dialect = DialectDescriptor::new(FVM_DIALECT);
    dialect.register_operation(
        OperationDescriptor::new(PROGRAM_OPERATION)
            .with_interface(FvmProgramInterface)
            .with_interface(DiscreteMethodInterface)
            .with_verifier(verify_program),
    )?;
    dialect.register_operation(
        OperationDescriptor::new(PREPARE_MESH_OPERATION)
            .with_interface(FvmMeshViewInterface)
            .with_verifier(verify_prepare_mesh),
    )?;
    dialect.register_operation(
        OperationDescriptor::new(CONSTANT_OPERATION).with_verifier(verify_constant),
    )?;
    dialect.register_operation(
        OperationDescriptor::new(LAPLACIAN_OPERATION)
            .with_interface(FvmStencilInterface)
            .with_verifier(|operation: OperationRef<'_>| {
                verify_leaf(operation, 8, 1)?;
                verify_scheme(operation, "gauss-orthogonal")
            }),
    )?;
    dialect.register_operation(OperationDescriptor::new(SCALE_OPERATION).with_verifier(
        |operation: OperationRef<'_>| {
            verify_leaf(operation, 4, 1)?;
            verify_scheme(operation, "pointwise")
        },
    ))?;
    dialect.register_operation(
        OperationDescriptor::new(EXPLICIT_EULER_OPERATION)
            .with_interface(FvmTimeIntegratorInterface)
            .with_verifier(|operation: OperationRef<'_>| {
                verify_leaf(operation, 5, 1)?;
                verify_scheme(operation, "explicit-euler")
            }),
    )?;
    registry.register_dialect(dialect)
}

fn verify_program(operation: OperationRef<'_>) -> Result<(), String> {
    if operation.regions().len() != 1
        || !operation.operands().is_empty()
        || !operation.results().is_empty()
        || !operation.is_symbol_table()
    {
        return Err(
            "fvm.program must be a zero-operand, zero-result symbol table with one region".into(),
        );
    }
    positive_integer(operation, DIMENSION_ATTRIBUTE)?;
    Ok(())
}

fn verify_prepare_mesh(operation: OperationRef<'_>) -> Result<(), String> {
    verify_leaf(operation, 1, 6)?;
    positive_integer(operation, DIMENSION_ATTRIBUTE)?;
    Ok(())
}

fn verify_constant(operation: OperationRef<'_>) -> Result<(), String> {
    verify_leaf(operation, 0, 1)?;
    match operation.attribute(VALUE_ATTRIBUTE) {
        Some(Attribute::Float(value)) if value.is_finite() => Ok(()),
        Some(Attribute::Float(_)) => Err("fvm.value must be finite".into()),
        Some(_) => Err("fvm.value must be a float".into()),
        None => Err("fvm.constant requires fvm.value".into()),
    }
}

fn verify_leaf(operation: OperationRef<'_>, operands: usize, results: usize) -> Result<(), String> {
    if operation.operands().len() != operands {
        return Err(format!("{} requires {operands} operands", operation.name()));
    }
    if operation.results().len() != results {
        return Err(format!("{} requires {results} results", operation.name()));
    }
    if !operation.regions().is_empty() || !operation.successors().is_empty() {
        return Err(format!("{} must be a leaf operation", operation.name()));
    }
    Ok(())
}

fn verify_scheme(operation: OperationRef<'_>, expected: &str) -> Result<(), String> {
    match operation
        .attribute(SCHEME_ATTRIBUTE)
        .and_then(Attribute::as_str)
    {
        Some(actual) if actual == expected => Ok(()),
        Some(actual) => Err(format!(
            "{} uses scheme '{actual}', expected '{expected}'",
            operation.name(),
        )),
        None => Err(format!("{} requires fvm.scheme", operation.name())),
    }
}

fn positive_integer(operation: OperationRef<'_>, name: &str) -> Result<i64, String> {
    match operation.attribute(name) {
        Some(Attribute::Integer(value)) if *value > 0 => Ok(*value),
        _ => Err(format!("attribute '{name}' must be a positive integer")),
    }
}
