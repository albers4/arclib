// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use compiler::{AnalysisManager, KernelCallOp, Pass, PassContext, PassError};
use ir::{
    GreedyRewriteConfig, Module, OperationId, PatternBenefit, PatternResult, PatternRewriter,
    RewriteError, RewritePattern, RewritePatternSet, apply_patterns_greedily,
};
use kernel::KernelDescriptor;

use crate::{EXPLICIT_EULER_OPERATION, FvmKernelSet, LAPLACIAN_OPERATION, SCALE_OPERATION};

pub const LOWER_FVM_TO_KERNEL_PASS: &str = "fvm.lower-to-kernel";
pub const LOWER_FVM_TO_KERNEL_PIPELINE: &str = "fvm.to-kernel";

pub struct LowerFvmToKernelPass {
    kernels: FvmKernelSet,
    config: GreedyRewriteConfig,
}

impl LowerFvmToKernelPass {
    pub fn new(kernels: FvmKernelSet) -> Self {
        Self {
            kernels,
            config: GreedyRewriteConfig::default(),
        }
    }

    pub fn with_config(mut self, config: GreedyRewriteConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for LowerFvmToKernelPass {
    fn default() -> Self {
        Self::new(FvmKernelSet::default())
    }
}

impl Pass for LowerFvmToKernelPass {
    fn name(&self) -> &'static str {
        LOWER_FVM_TO_KERNEL_PASS
    }

    fn run(
        &self,
        module: &mut Module,
        context: &mut PassContext,
        _analyses: &mut AnalysisManager,
    ) -> Result<(), PassError> {
        let mut patterns = RewritePatternSet::new();
        patterns.add(
            LAPLACIAN_OPERATION,
            PatternBenefit::DEFAULT,
            LowerOperation::new(
                "LowerFvmLaplacianToKernel",
                self.kernels.laplacian().clone(),
            ),
        );
        patterns.add(
            SCALE_OPERATION,
            PatternBenefit::DEFAULT,
            LowerOperation::new("LowerFvmScaleToKernel", self.kernels.scale().clone()),
        );
        patterns.add(
            EXPLICIT_EULER_OPERATION,
            PatternBenefit::DEFAULT,
            LowerOperation::new(
                "LowerFvmExplicitEulerToKernel",
                self.kernels.explicit_euler().clone(),
            ),
        );

        let report = apply_patterns_greedily(module, &patterns, &self.config)
            .map_err(|error| PassError::failed(self.name(), error.to_string()))?;
        if report.rewrites > 0 {
            context.mark_changed();
        }
        Ok(())
    }
}

struct LowerOperation {
    name: &'static str,
    descriptor: Arc<KernelDescriptor>,
}

impl LowerOperation {
    fn new(name: &'static str, descriptor: Arc<KernelDescriptor>) -> Self {
        Self { name, descriptor }
    }
}

impl RewritePattern for LowerOperation {
    fn name(&self) -> &'static str {
        self.name
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<PatternResult, RewriteError> {
        let (operands, result_types) = {
            let source = rewriter
                .operation(operation)
                .expect("FVM rewrite root operation must exist");
            let result_types = (0..source.results().len())
                .map(|index| {
                    source
                        .result_type(index)
                        .expect("FVM result must have a type")
                        .clone()
                })
                .collect::<Vec<_>>();
            (source.operands().to_vec(), result_types)
        };
        let call = rewriter.create_operation(
            KernelCallOp::builder(&self.descriptor, result_types),
            operands,
        )?;
        let results = rewriter
            .operation(call)
            .expect("new kernel.call must exist")
            .results()
            .to_vec();
        rewriter.replace_operation(operation, &results)?;
        Ok(PatternResult::Rewritten)
    }
}
