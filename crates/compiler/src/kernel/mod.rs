// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod dialect;
mod error;
mod extension;
mod pass;
mod pattern;
mod registry;
mod requirement;

pub use error::KernelError;

pub use dialect::{
    KERNEL_BACKEND_ATTRIBUTE, KERNEL_CALL_OPERATION, KERNEL_DIALECT, KERNEL_NAME_ATTRIBUTE,
    KERNEL_PARAMETER_COUNT_ATTRIBUTE, KERNEL_RESULT_ALIAS_PREFIX, KERNEL_RESULT_COUNT_ATTRIBUTE,
    KERNEL_SYMBOL_ATTRIBUTE, KernelCallInterface, KernelCallOp, register_kernel_dialect,
};

pub use extension::{KernelCompilerExtension, SELECT_CUSTOM_KERNELS_PIPELINE};

pub use pass::{SELECT_CUSTOM_KERNELS_PASS, SelectCustomKernelsPass};

pub use pattern::{CustomKernelPattern, KernelPatternAdapter, KernelPatternRewriter};

pub use registry::{KernelRegistration, KernelRegistry};

pub use requirement::KernelBackendRequirement;
