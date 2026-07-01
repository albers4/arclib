// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use compiler::{CompilerExtension, RegistryError};

use crate::register_method_dialect;

#[derive(Debug, Default, Clone, Copy)]
pub struct MethodCompilerExtension;

impl MethodCompilerExtension {
    pub fn new() -> Self {
        Self {}
    }
}

impl CompilerExtension for MethodCompilerExtension {
    fn name(&self) -> &'static str {
        "method"
    }

    fn register(&self, registry: &mut compiler::CompilerRegistry) -> Result<(), RegistryError> {
        register_method_dialect(registry.dialects_mut())?;

        Ok(())
    }
}
