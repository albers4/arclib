// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use ir::{
    Attribute, DialectDescriptor, DialectRegistry, DialectRegistryError, OperationBuilder,
    OperationDescriptor, OperationRef, Type,
};

use super::{
    EquationSystemInterface, EvolutionEquationInterface, ResidualInterface, TimeIntegrator,
    UnknownInterface,
};

pub const EQUATION_DIALECT: &str = "equation";
pub const EQUATION_STAGE: &str = "equation";

pub const SYSTEM_OPERATION: &str = "equation.system";
pub const UNKNOWN_OPERATION: &str = "equation.unknown";
pub const RESIDUAL_OPERATION: &str = "equation.residual";
pub const EVOLUTION_OPERATION: &str = "equation.evolution";

pub const NAME_ATTRIBUTE: &str = "equation.name";
pub const INTEGRATOR_ATTRIBUTE: &str = "equation.integrator";

pub const EVOLUTION_STATE_OPERAND: usize = 0;
pub const EVOLUTION_RHS_OPERAND: usize = 1;
pub const EVOLUTION_TIME_STEP_OPERAND: usize = 2;

pub struct EquationSystemOp;
impl EquationSystemOp {
    pub fn builder(name: impl AsRef<str>) -> OperationBuilder {
        OperationBuilder::new(SYSTEM_OPERATION)
            .symbol_table()
            .region()
            .attribute(NAME_ATTRIBUTE, Attribute::string(name))
    }
}

pub struct UnknownOp;
impl UnknownOp {
    pub fn builder(name: impl AsRef<str>, result_type: Type) -> OperationBuilder {
        OperationBuilder::new(UNKNOWN_OPERATION)
            .attribute(NAME_ATTRIBUTE, Attribute::string(name))
            .result(result_type)
    }
}

pub struct ResidualOp;
impl ResidualOp {
    pub fn builder(name: impl AsRef<str>) -> OperationBuilder {
        OperationBuilder::new(RESIDUAL_OPERATION).attribute(NAME_ATTRIBUTE, Attribute::string(name))
    }
}

pub struct EvolutionOp;
impl EvolutionOp {
    pub fn builder(
        name: impl AsRef<str>,
        result_type: Type,
        integrator: TimeIntegrator,
    ) -> OperationBuilder {
        OperationBuilder::new(EVOLUTION_OPERATION)
            .attribute(NAME_ATTRIBUTE, Attribute::string(name))
            .attribute(INTEGRATOR_ATTRIBUTE, Attribute::string(integrator.as_str()))
            .result(result_type)
    }
}

pub fn register_equation_dialect(
    registry: &mut DialectRegistry,
) -> Result<(), DialectRegistryError> {
    let mut dialect = DialectDescriptor::new(EQUATION_DIALECT);
    dialect.register_operation(
        OperationDescriptor::new(SYSTEM_OPERATION)
            .with_interface(EquationSystemInterface)
            .with_verifier(verify_system),
    )?;
    dialect.register_operation(
        OperationDescriptor::new(UNKNOWN_OPERATION)
            .with_interface(UnknownInterface)
            .with_verifier(verify_unknown),
    )?;
    dialect.register_operation(
        OperationDescriptor::new(RESIDUAL_OPERATION)
            .with_interface(ResidualInterface)
            .with_verifier(verify_residual),
    )?;
    dialect.register_operation(
        OperationDescriptor::new(EVOLUTION_OPERATION)
            .with_interface(EvolutionEquationInterface)
            .with_verifier(verify_evolution),
    )?;
    registry.register_dialect(dialect)
}

fn verify_system(operation: OperationRef<'_>) -> Result<(), String> {
    if operation.regions().len() != 1
        || !operation.operands().is_empty()
        || !operation.results().is_empty()
        || !operation.is_symbol_table()
    {
        return Err(
            "equation.system must be a zero-operand, zero-result symbol table with one region"
                .into(),
        );
    }
    require_name(operation)
}

fn verify_unknown(operation: OperationRef<'_>) -> Result<(), String> {
    verify_leaf(operation, 1, 1)?;
    require_name(operation)
}

fn verify_residual(operation: OperationRef<'_>) -> Result<(), String> {
    verify_leaf(operation, 1, 0)?;
    require_name(operation)
}

fn verify_evolution(operation: OperationRef<'_>) -> Result<(), String> {
    verify_leaf(operation, 3, 1)?;
    require_name(operation)?;
    match operation
        .attribute(INTEGRATOR_ATTRIBUTE)
        .and_then(Attribute::as_str)
    {
        Some("explicit-euler") => Ok(()),
        Some(value) => Err(format!("unsupported equation integrator '{value}'")),
        None => Err("equation.evolution requires equation.integrator".into()),
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

fn require_name(operation: OperationRef<'_>) -> Result<(), String> {
    match operation
        .attribute(NAME_ATTRIBUTE)
        .and_then(Attribute::as_str)
    {
        Some(value) if !value.trim().is_empty() => Ok(()),
        Some(_) => Err("equation.name must not be empty".into()),
        None => Err("equation.name must be a string".into()),
    }
}
