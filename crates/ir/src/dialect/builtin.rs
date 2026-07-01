// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::{OperationRef, Pure};

use super::{
    DialectDescriptor, DialectRegistry, DialectRegistryError, IsolatedFromAbove,
    OperationDescriptor,
};

const BUILTIN_DIALECT: &str = "builtin";

const BUILTIN_MODULE: &str = "builtin.module";

const BUILTIN_NAMESPACE: &str = "builtin.namespace";

pub const UNREALIZED_CAST: &str = "builtin.unrealized_conversion_cast";

pub fn register_builtin_dialect(
    registry: &mut DialectRegistry,
) -> Result<(), DialectRegistryError> {
    let mut builtin = DialectDescriptor::new(BUILTIN_DIALECT);

    builtin.register_operation(
        OperationDescriptor::new(BUILTIN_MODULE)
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
        OperationDescriptor::new(BUILTIN_NAMESPACE)
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

    builtin.register_operation(OperationDescriptor::new(UNREALIZED_CAST).with_trait::<Pure>())?;

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
