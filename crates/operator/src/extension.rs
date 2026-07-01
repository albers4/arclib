// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use compiler::{CompilerExtension, CompilerRegistry, RegistryError};

use crate::register_operator_dialect;

#[derive(Debug, Default, Clone, Copy)]
pub struct OperatorCompilerExtension;

impl OperatorCompilerExtension {
    pub const fn new() -> Self {
        Self
    }
}

impl CompilerExtension for OperatorCompilerExtension {
    fn name(&self) -> &'static str {
        "operator"
    }

    fn register(&self, registry: &mut CompilerRegistry) -> Result<(), RegistryError> {
        register_operator_dialect(registry.dialects_mut())?;
        Ok(())
    }
}
