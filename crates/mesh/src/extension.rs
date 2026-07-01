// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use compiler::{
    CompilerExtension, CompilerRegistry, ConversionEdgeDescriptor, PipelineDescriptor,
    RegistryError,
};
use domain::DOMAIN_STAGE;

use crate::{
    LOWER_DOMAIN_TO_MESH_PASS, LOWER_DOMAIN_TO_MESH_PIPELINE, LowerDomainToMeshPass, MESH_STAGE,
    register_mesh_dialect,
};

pub const MESH_TO_METHOD_PIPELINE: &str = "mesh.ready-for-method";

#[derive(Debug, Default, Clone, Copy)]
pub struct MeshCompilerExtension;

impl MeshCompilerExtension {
    pub const fn new() -> Self {
        Self
    }
}

impl CompilerExtension for MeshCompilerExtension {
    fn name(&self) -> &'static str {
        "mesh"
    }

    fn register(&self, registry: &mut CompilerRegistry) -> Result<(), RegistryError> {
        register_mesh_dialect(registry.dialects_mut())?;
        registry
            .passes_mut()
            .register(LOWER_DOMAIN_TO_MESH_PASS, LowerDomainToMeshPass::new)?;
        registry.pipelines_mut().register(
            PipelineDescriptor::new(LOWER_DOMAIN_TO_MESH_PIPELINE).pass(LOWER_DOMAIN_TO_MESH_PASS),
        )?;
        // This is intentionally empty: it marks that a method-neutral mesh is
        // available and the planner may now choose a numerical method.
        registry
            .pipelines_mut()
            .register(PipelineDescriptor::new(MESH_TO_METHOD_PIPELINE))?;
        registry.conversions_mut().register(
            ConversionEdgeDescriptor::new(
                "domain-to-mesh",
                DOMAIN_STAGE,
                MESH_STAGE,
                LOWER_DOMAIN_TO_MESH_PIPELINE,
            )
            .property("spatial.representation", "mesh"),
        )?;
        registry.conversions_mut().register(
            ConversionEdgeDescriptor::new(
                "mesh-to-method",
                MESH_STAGE,
                "method",
                MESH_TO_METHOD_PIPELINE,
            )
            .property("spatial.mesh", "available"),
        )?;
        Ok(())
    }
}
