// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use compiler::{CompilerExtension, CompilerRegistry, RegistryError};

use crate::register_domain_dialect;

#[derive(Debug, Default, Clone, Copy)]
pub struct DomainCompilerExtension;

impl DomainCompilerExtension {
    pub const fn new() -> Self {
        Self
    }
}

impl CompilerExtension for DomainCompilerExtension {
    fn name(&self) -> &'static str {
        "domain"
    }

    fn register(&self, registry: &mut CompilerRegistry) -> Result<(), RegistryError> {
        register_domain_dialect(registry.dialects_mut())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use compiler::CompilerSession;
    use ir::Module;

    use crate::{DomainCompilerExtension, LineDomain, LineOp};

    #[test]
    fn registers_and_verifies_an_embedded_line_domain() {
        let session = CompilerSession::builder()
            .register_extension(DomainCompilerExtension::new())
            .unwrap()
            .build();

        let mut module = Module::new();
        let line = LineDomain::new([0.0, 0.0, 0.0], [1.0, 1.0, 0.0]).unwrap();

        module
            .append_operation(LineOp::builder("diagonal", &line), [])
            .unwrap();

        session.verify(&module).unwrap();
    }
}
