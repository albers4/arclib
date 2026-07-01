// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use compiler::{CompilerExtension, CompilerRegistry, RegistryError};

use crate::register_equation_dialect;

#[derive(Debug, Default, Clone, Copy)]
pub struct EquationCompilerExtension;

impl EquationCompilerExtension {
    pub const fn new() -> Self {
        Self
    }
}

impl CompilerExtension for EquationCompilerExtension {
    fn name(&self) -> &'static str {
        "equation"
    }

    fn register(&self, registry: &mut CompilerRegistry) -> Result<(), RegistryError> {
        register_equation_dialect(registry.dialects_mut())?;
        Ok(())
    }
}
