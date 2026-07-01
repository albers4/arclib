// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod extension;

pub mod dialect;

pub use extension::MethodCompilerExtension;

pub use dialect::{
    DISCRETIZE_OPERATION, DiscreteMethodInterface, DiscretizationRequestInterface, DiscretizeOp,
    METHOD_DIALECT, METHOD_FAMILY_ATTRIBUTE, METHOD_KIND_ATTRIBUTE, METHOD_STAGE, MethodFamily,
    MethodOpInterface, register_method_dialect,
};

#[cfg(test)]
mod tests {
    use compiler::CompilerSession;

    use ir::Module;

    use crate::{DiscretizeOp, MethodCompilerExtension, MethodFamily};

    #[test]
    fn registers_and_verifies_method_dialect() {
        let session = CompilerSession::builder()
            .register_extension(MethodCompilerExtension::new())
            .unwrap()
            .build();

        assert!(session.registry().has_extension("method",));

        let mut module = Module::new();

        module
            .append_operation(
                DiscretizeOp::for_method(MethodFamily::Continuum, "finite-volume"),
                [],
            )
            .unwrap();

        session.verify(&module).unwrap();
    }
}
