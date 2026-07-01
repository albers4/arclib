// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use ir::{
    OperationId,
    OperationName,
    OperationRef,
    PatternBenefit,
    PatternResult,
    PatternRewriter,
    RewriteError,
    Type,
    ValueId,
};

use kernel::{
    KernelDescriptor,
};

use crate::KernelCallOp;

pub trait CustomKernelPattern:
    Send + Sync
{
    fn name(&self) -> &'static str;

    fn root_operation(
        &self,
    ) -> OperationName;

    fn benefit(
        &self,
    ) -> PatternBenefit {
        PatternBenefit::DEFAULT
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter:
            &mut KernelPatternRewriter<'_, '_>,
    ) -> Result<
        PatternResult,
        RewriteError,
    >;
}

pub struct KernelPatternRewriter<
    'r,
    'm,
> {
    base:
        &'r mut PatternRewriter<'m>,

    descriptor:
        &'r KernelDescriptor,
}

impl<'r, 'm>
    KernelPatternRewriter<'r, 'm>
{
    pub(crate) fn new(
        base:
            &'r mut PatternRewriter<'m>,

        descriptor:
            &'r KernelDescriptor,
    ) -> Self {
        Self {
            base,
            descriptor,
        }
    }

    pub fn descriptor(
        &self,
    ) -> &KernelDescriptor {
        self.descriptor
    }

    pub fn operation(
        &self,
        operation: OperationId,
    ) -> Option<
        OperationRef<'_>,
    > {
        self.base
            .operation(operation)
    }

    pub fn base(
        &self,
    ) -> &PatternRewriter<'m> {
        self.base
    }

    pub fn base_mut(
        &mut self,
    ) -> &mut PatternRewriter<'m> {
        self.base
    }

    pub fn replace_with_kernel_call(
        &mut self,

        operation: OperationId,

        operands:
            impl IntoIterator<
                Item = ValueId,
            >,

        result_types:
            impl IntoIterator<
                Item = Type,
            >,
    ) -> Result<
        OperationId,
        RewriteError,
    > {
        let call =
            self.base
                .create_operation(
                    KernelCallOp::builder(
                        self.descriptor,
                        result_types,
                    ),
                    operands,
                )?;

        let results =
            self.base
                .operation(call)
                .expect(
                    "newly created \
                     kernel.call must exist",
                )
                .results()
                .to_vec();

        self.base
            .replace_operation(
                operation,
                &results,
            )?;

        Ok(call)
    }
}

pub struct KernelPatternAdapter {
    descriptor:
        Arc<KernelDescriptor>,

    pattern:
        Arc<
            dyn CustomKernelPattern,
        >,
}

impl KernelPatternAdapter {
    pub fn new(
        descriptor:
            Arc<KernelDescriptor>,

        pattern:
            Arc<
                dyn CustomKernelPattern,
            >,
    ) -> Self {
        Self {
            descriptor,
            pattern,
        }
    }
}

impl ir::RewritePattern
    for KernelPatternAdapter
{
    fn name(&self) -> &'static str {
        self.pattern.name()
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter:
            &mut PatternRewriter<'_>,
    ) -> Result<
        PatternResult,
        RewriteError,
    > {
        let mut kernel_rewriter =
            KernelPatternRewriter::new(
                rewriter,
                &self.descriptor,
            );

        self.pattern
            .match_and_rewrite(
                operation,
                &mut kernel_rewriter,
            )
    }
}