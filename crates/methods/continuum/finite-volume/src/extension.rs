// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use compiler::{
    CapabilityRequirement, CompilerExtension, CompilerRegistry, ConversionEdgeDescriptor,
    EdgeApplicability, PipelineDescriptor, RegistryError,
};
use mesh::{GENERATE_OPERATION, SHAPE_ATTRIBUTE, STRUCTURED_LINE_OPERATION};
use method::{DISCRETIZE_OPERATION, METHOD_FAMILY_ATTRIBUTE, METHOD_KIND_ATTRIBUTE, METHOD_STAGE};

use crate::{
    FVM_STAGE, FvmKernelSet, LOWER_FVM_TO_KERNEL_PASS, LOWER_FVM_TO_KERNEL_PIPELINE,
    LOWER_METHOD_TO_FVM_PASS, LOWER_METHOD_TO_FVM_PIPELINE, LowerFvmToKernelPass,
    LowerMethodToFvmPass, register_fvm_dialect,
};

#[derive(Debug, Clone)]
pub struct FiniteVolumeCompilerExtension {
    kernels: FvmKernelSet,
}

impl FiniteVolumeCompilerExtension {
    pub fn new() -> Self {
        Self {
            kernels: FvmKernelSet::default(),
        }
    }

    pub fn with_kernels(mut self, kernels: FvmKernelSet) -> Self {
        self.kernels = kernels;
        self
    }
}

impl Default for FiniteVolumeCompilerExtension {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerExtension for FiniteVolumeCompilerExtension {
    fn name(&self) -> &'static str {
        "finite-volume-method"
    }

    fn register(&self, registry: &mut CompilerRegistry) -> Result<(), RegistryError> {
        register_fvm_dialect(registry.dialects_mut())?;

        registry
            .passes_mut()
            .register(LOWER_METHOD_TO_FVM_PASS, LowerMethodToFvmPass::new)?;
        let kernels = self.kernels.clone();
        registry
            .passes_mut()
            .register(LOWER_FVM_TO_KERNEL_PASS, move || {
                LowerFvmToKernelPass::new(kernels.clone())
            })?;

        registry.pipelines_mut().register(
            PipelineDescriptor::new(LOWER_METHOD_TO_FVM_PIPELINE).pass(LOWER_METHOD_TO_FVM_PASS),
        )?;
        registry.pipelines_mut().register(
            PipelineDescriptor::new(LOWER_FVM_TO_KERNEL_PIPELINE).pass(LOWER_FVM_TO_KERNEL_PASS),
        )?;

        registry.conversions_mut().register(
            ConversionEdgeDescriptor::new(
                "method-to-fvm",
                METHOD_STAGE,
                FVM_STAGE,
                LOWER_METHOD_TO_FVM_PIPELINE,
            )
            .property("method.family", "continuum")
            .property("method.name", "finite-volume")
            .property("mesh.requirement", "cell-face-topology")
            .property("spatial.dimension", 1_i64)
            .applicable_when(|module, _request| {
                let mut has_line_mesh = false;

                for id in module.operations() {
                    let Some(operation) = module.operation(id) else {
                        continue;
                    };

                    if operation.name().as_str() == STRUCTURED_LINE_OPERATION {
                        has_line_mesh = true;
                    }

                    if operation.name().as_str() == GENERATE_OPERATION
                        && operation
                            .attribute(SHAPE_ATTRIBUTE)
                            .and_then(ir::Attribute::as_str)
                            == Some("line")
                    {
                        has_line_mesh = true;
                    }

                    if operation.name().as_str() == DISCRETIZE_OPERATION {
                        let family = operation
                            .attribute(METHOD_FAMILY_ATTRIBUTE)
                            .and_then(ir::Attribute::as_str);
                        let kind = operation
                            .attribute(METHOD_KIND_ATTRIBUTE)
                            .and_then(ir::Attribute::as_str);

                        if family.is_some_and(|value| value != "continuum") {
                            return EdgeApplicability::unavailable(
                                "the discretization request does not select the continuum family",
                            );
                        }

                        if kind.is_some_and(|value| {
                            !matches!(value, "finite-volume" | "fvm")
                        }) {
                            return EdgeApplicability::unavailable(
                                "the discretization request selects another numerical method",
                            );
                        }
                    }
                }

                if has_line_mesh {
                    EdgeApplicability::Applicable
                } else {
                    EdgeApplicability::unavailable(
                        "the first FVM subset requires a structured 1D line mesh or line-mesh request",
                    )
                }
            }),
        )?;
        registry.conversions_mut().register(
            ConversionEdgeDescriptor::new(
                "fvm-to-kernel",
                FVM_STAGE,
                "kernel",
                LOWER_FVM_TO_KERNEL_PIPELINE,
            )
            .property("numeric.scalar", "f64")
            .requires_target(CapabilityRequirement::equals("backend.cpu_openmp", true)),
        )?;
        Ok(())
    }
}
