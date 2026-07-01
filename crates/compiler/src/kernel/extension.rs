// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use crate::{
    CompilerExtension,
    CompilerRegistry,
    PipelineDescriptor,
    RegistryError,
    KernelRegistry,
    SELECT_CUSTOM_KERNELS_PASS,
    SelectCustomKernelsPass,
    register_kernel_dialect,
};

pub const SELECT_CUSTOM_KERNELS_PIPELINE:
    &str =
    "kernel.select-custom";

pub struct KernelCompilerExtension {
    registry:
        Arc<KernelRegistry>,
}

impl KernelCompilerExtension {
    pub fn new(
        registry:
            KernelRegistry,
    ) -> Self {
        Self {
            registry:
                Arc::new(
                    registry,
                ),
        }
    }

    pub fn registry(
        &self,
    ) -> &Arc<KernelRegistry> {
        &self.registry
    }
}

impl CompilerExtension
    for KernelCompilerExtension
{
    fn name(&self) -> &'static str {
        "kernel"
    }

    fn register(
        &self,
        registry:
            &mut CompilerRegistry,
    ) -> Result<(), RegistryError> {
        register_kernel_dialect(
            registry.dialects_mut(),
        )?;

        let kernels =
            self.registry.clone();

        registry
            .passes_mut()
            .register(
                SELECT_CUSTOM_KERNELS_PASS,

                move || {
                    SelectCustomKernelsPass::new(
                        kernels.clone(),
                    )
                },
            )?;

        registry
            .pipelines_mut()
            .register(
                PipelineDescriptor::new(
                    SELECT_CUSTOM_KERNELS_PIPELINE,
                )
                .pass(
                    SELECT_CUSTOM_KERNELS_PASS,
                ),
            )?;

        Ok(())
    }
}