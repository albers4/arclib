// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use ir::{
    Attribute, DialectDescriptor, DialectRegistry, DialectRegistryError, OperationBuilder,
    OperationDescriptor, OperationRef,
};

use crate::dialect::{
    family::MethodFamily,
    interfaces::{DiscretizationRequestInterface, MethodOpInterface},
};

pub const METHOD_DIALECT: &str = "method";

pub const METHOD_STAGE: &str = "method";

pub const DISCRETIZE_OPERATION: &str = "method.discretize";

pub const METHOD_FAMILY_ATTRIBUTE: &str = "method.family";

pub const METHOD_KIND_ATTRIBUTE: &str = "method.kind";

pub struct DiscretizeOp;

impl DiscretizeOp {
    pub fn builder() -> OperationBuilder {
        OperationBuilder::new(DISCRETIZE_OPERATION).region()
    }

    pub fn for_family(family: MethodFamily) -> OperationBuilder {
        Self::builder().attribute(METHOD_FAMILY_ATTRIBUTE, Attribute::string(family.as_str()))
    }

    pub fn for_method(family: MethodFamily, kind: impl AsRef<str>) -> OperationBuilder {
        Self::for_family(family).attribute(METHOD_KIND_ATTRIBUTE, Attribute::string(kind))
    }
}

pub fn register_method_dialect(registry: &mut DialectRegistry) -> Result<(), DialectRegistryError> {
    let mut dialect = DialectDescriptor::new(METHOD_DIALECT);

    dialect.register_operation(
        OperationDescriptor::new(DISCRETIZE_OPERATION)
            .with_interface(MethodOpInterface)
            .with_interface(DiscretizationRequestInterface)
            .with_verifier(verify_discretize),
    )?;

    registry.register_dialect(dialect)
}

pub fn verify_discretize(operation: OperationRef<'_>) -> Result<(), String> {
    if operation.regions().len() != 1 {
        return Err("method.discretize must own exactly one region".into());
    }

    if !operation.operands().is_empty() {
        return Err("method.discretize must not have operands".into());
    }

    if !operation.results().is_empty() {
        return Err("method.discretize must not produce results".into());
    }

    verify_family(operation)?;

    verify_kind(operation)?;

    Ok(())
}

fn verify_family(operation: OperationRef<'_>) -> Result<(), String> {
    let Some(attribute) = operation.attribute(METHOD_FAMILY_ATTRIBUTE) else {
        return Ok(());
    };

    let Some(value) = attribute.as_str() else {
        return Err("method.family must be a string".into());
    };

    if MethodFamily::parse(value).is_none() {
        return Err(format!(
            "unsupported method family \
                 '{value}'"
        ));
    }

    Ok(())
}

fn verify_kind(operation: OperationRef<'_>) -> Result<(), String> {
    let Some(attribute) = operation.attribute(METHOD_KIND_ATTRIBUTE) else {
        return Ok(());
    };

    let Some(value) = attribute.as_str() else {
        return Err("method.kind must be a string".into());
    };

    if value.trim().is_empty() {
        return Err("method.kind must not be empty".into());
    }

    Ok(())
}
