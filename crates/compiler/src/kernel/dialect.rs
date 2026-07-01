// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use ir::{
    Attribute, DialectDescriptor, DialectRegistry, DialectRegistryError, OperationBuilder,
    OperationDescriptor, OperationRef, Type,
};

use kernel::KernelDescriptor;

pub const KERNEL_DIALECT: &str = "kernel";

pub const KERNEL_CALL_OPERATION: &str = "kernel.call";

pub const KERNEL_NAME_ATTRIBUTE: &str = "kernel.name";

pub const KERNEL_SYMBOL_ATTRIBUTE: &str = "kernel.symbol";

pub const KERNEL_BACKEND_ATTRIBUTE: &str = "kernel.backend";

pub const KERNEL_PARAMETER_COUNT_ATTRIBUTE: &str = "kernel.parameter_count";

pub const KERNEL_RESULT_COUNT_ATTRIBUTE: &str = "kernel.result_count";

pub const KERNEL_RESULT_ALIAS_PREFIX: &str = "kernel.result_alias.";

#[derive(Debug, Default, Clone, Copy)]
pub struct KernelCallInterface;

pub struct KernelCallOp;

impl KernelCallOp {
    pub fn builder(
        descriptor: &KernelDescriptor,

        result_types: impl IntoIterator<Item = Type>,
    ) -> OperationBuilder {
        let result_types: Vec<Type> = result_types.into_iter().collect();

        assert_eq!(
            result_types.len(),
            descriptor.abi().result_aliases().len(),
            "kernel.call result types must match \
             the descriptor ABI result aliases",
        );

        let mut builder = OperationBuilder::new(KERNEL_CALL_OPERATION)
            .attribute(KERNEL_NAME_ATTRIBUTE, Attribute::string(descriptor.name()))
            .attribute(
                KERNEL_SYMBOL_ATTRIBUTE,
                Attribute::string(descriptor.symbol()),
            )
            .attribute(
                KERNEL_BACKEND_ATTRIBUTE,
                Attribute::string(descriptor.backend().as_str()),
            )
            .attribute(
                KERNEL_PARAMETER_COUNT_ATTRIBUTE,
                Attribute::Integer(i64::try_from(descriptor.abi().parameters().len()).expect(
                    "kernel parameter count \
                         exceeds i64",
                )),
            )
            .attribute(
                KERNEL_RESULT_COUNT_ATTRIBUTE,
                Attribute::Integer(i64::try_from(result_types.len()).expect(
                    "kernel result count \
                         exceeds i64",
                )),
            )
            .results(result_types);

        for (result_index, parameter_index) in descriptor
            .abi()
            .result_aliases()
            .iter()
            .copied()
            .enumerate()
        {
            builder = builder.attribute(
                format!(
                    "{KERNEL_RESULT_ALIAS_PREFIX}\
                         {result_index}"
                ),
                Attribute::Integer(i64::try_from(parameter_index).expect(
                    "kernel parameter index \
                             exceeds i64",
                )),
            );
        }

        builder
    }
}

pub fn register_kernel_dialect(registry: &mut DialectRegistry) -> Result<(), DialectRegistryError> {
    let mut dialect = DialectDescriptor::new(KERNEL_DIALECT);

    dialect.register_operation(
        OperationDescriptor::new(KERNEL_CALL_OPERATION)
            .with_interface(KernelCallInterface)
            .with_verifier(verify_kernel_call),
    )?;

    registry.register_dialect(dialect)
}

fn verify_kernel_call(operation: OperationRef<'_>) -> Result<(), String> {
    if !operation.regions().is_empty() {
        return Err("kernel.call must not own regions".into());
    }

    if !operation.successors().is_empty() {
        return Err("kernel.call must not have successors".into());
    }

    for attribute in [
        KERNEL_NAME_ATTRIBUTE,
        KERNEL_SYMBOL_ATTRIBUTE,
        KERNEL_BACKEND_ATTRIBUTE,
    ] {
        if operation
            .attribute(attribute)
            .and_then(Attribute::as_str)
            .is_none()
        {
            return Err(format!(
                "kernel.call requires string \
                     attribute '{attribute}'"
            ));
        }
    }

    let expected = match operation.attribute(KERNEL_PARAMETER_COUNT_ATTRIBUTE) {
        Some(Attribute::Integer(value)) if *value >= 0 => {
            usize::try_from(*value).map_err(|_| {
                "kernel parameter count \
                         is too large"
                    .to_owned()
            })?
        }

        _ => {
            return Err("kernel.call requires a \
                     non-negative integer \
                     kernel.parameter_count"
                .into());
        }
    };

    let result_count = match operation.attribute(KERNEL_RESULT_COUNT_ATTRIBUTE) {
        Some(Attribute::Integer(value)) if *value >= 0 => {
            usize::try_from(*value).map_err(|_| "kernel result count is too large".to_owned())?
        }

        _ => {
            return Err("kernel.call requires a \
                    non-negative integer \
                    kernel.result_count"
                .into());
        }
    };

    if operation.results().len() != result_count {
        return Err(format!(
            "kernel.call has {} results but \
                declares {result_count}",
            operation.results().len(),
        ));
    }

    for result_index in 0..result_count {
        let attribute = format!(
            "{KERNEL_RESULT_ALIAS_PREFIX}\
                {result_index}"
        );

        let parameter_index = match operation.attribute(&attribute) {
            Some(Attribute::Integer(value)) if *value >= 0 => {
                usize::try_from(*value).map_err(|_| format!("'{attribute}' is too large"))?
            }

            _ => {
                return Err(format!(
                    "kernel.call requires \
                            non-negative integer \
                            attribute '{attribute}'"
                ));
            }
        };

        if parameter_index >= expected {
            return Err(format!(
                "kernel result {result_index} \
                    aliases ABI parameter \
                    {parameter_index}, but only \
                    {expected} parameters exist"
            ));
        }
    }

    if operation.operands().len() != expected {
        return Err(format!(
            "kernel.call has {} operands \
                 but its ABI expects {expected}",
            operation.operands().len(),
        ));
    }

    Ok(())
}
