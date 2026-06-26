// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::{OperationRef, Pure};

use super::{
    DialectDescriptor, DialectRegistry, DialectRegistryError, IsolatedFromAbove,
    OperationDescriptor,
};

pub fn register_builtin_dialect(
    registry: &mut DialectRegistry,
) -> Result<(), DialectRegistryError> {
    let mut builtin = DialectDescriptor::new("builtin");

    builtin.register_operation(
        OperationDescriptor::new("builtin.module")
            .with_trait::<IsolatedFromAbove>()
            .with_verifier(|operation: OperationRef| {
                if operation.regions().len() != 1 {
                    return Err("builtin.module must contain \
                     exactly one region"
                        .into());
                }

                if operation.parent_block().is_some() {
                    return Err("builtin.module must not have \
                     a parent block"
                        .into());
                }

                Ok(())
            }),
    )?;

    builtin.register_operation(
        OperationDescriptor::new("builtin.namespace")
            .with_trait::<IsolatedFromAbove>()
            .with_verifier(|operation: OperationRef| {
                if operation.regions().len() != 1 {
                    return Err("builtin.namespace must contain \
                     exactly one region"
                        .into());
                }

                Ok(())
            }),
    )?;

    builtin.register_operation(
        OperationDescriptor::new("builtin.unrealized_conversion_cast").with_trait::<Pure>(),
    )?;

    registry.register_dialect(builtin)
}

impl DialectRegistry {
    pub fn with_builtin() -> Self {
        let mut registry = Self::new();

        register_builtin_dialect(&mut registry).expect(
            "builtin dialect registration \
             must always succeed",
        );

        registry
    }
}
