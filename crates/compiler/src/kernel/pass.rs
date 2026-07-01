// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use crate::{
    AnalysisManager, KernelPatternAdapter, KernelRegistry, Pass, PassContext, PassError,
};

use ir::{
    GreedyRewriteConfig,
    Module,
    RewritePatternSet,
    apply_patterns_greedily,
};

pub const SELECT_CUSTOM_KERNELS_PASS:
    &str =
    "kernel.select-custom";

pub struct SelectCustomKernelsPass {
    registry:
        Arc<KernelRegistry>,

    config:
        GreedyRewriteConfig,
}

impl SelectCustomKernelsPass {
    pub fn new(
        registry:
            Arc<KernelRegistry>,
    ) -> Self {
        Self {
            registry,

            config:
                GreedyRewriteConfig::
                    default(),
        }
    }
}

impl Pass
    for SelectCustomKernelsPass
{
    fn name(&self) -> &'static str {
        SELECT_CUSTOM_KERNELS_PASS
    }

    fn run(
        &self,
        module: &mut Module,
        context:
            &mut PassContext,
        _analyses:
            &mut AnalysisManager,
    ) -> Result<(), PassError> {
        let mut patterns =
            RewritePatternSet::new();

        for registration in
            self.registry
                .registrations()
        {
            if !registration
                .supports(
                    context
                        .request()
                        .target(),
                )
            {
                continue;
            }

            let pattern =
                registration.pattern();

            let root_operation = pattern.root_operation();

            patterns.add(
                root_operation.as_str(),
                pattern.benefit(),

                KernelPatternAdapter::new(
                    registration
                        .descriptor()
                        .clone(),

                    registration
                        .pattern()
                        .clone(),
                ),
            );
        }

        if patterns.is_empty() {
            return Ok(());
        }

        let report =
            apply_patterns_greedily(
                module,
                &patterns,
                &self.config,
            )
            .map_err(|error| {
                PassError::failed(
                    self.name(),
                    error.to_string(),
                )
            })?;

        if report.rewrites > 0 {
            context.mark_changed();
        }

        Ok(())
    }
}