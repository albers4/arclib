// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod error;
mod extension;
mod kernels;
mod lowering;
mod mesh;
mod plan;

pub mod dialect;

mod native;
mod portable;

pub use dialect::*;
pub use error::FvmError;
pub use extension::FiniteVolumeCompilerExtension;
pub use kernels::{
    EXPLICIT_EULER_KERNEL, FvmKernelSet, LAPLACIAN_KERNEL, SCALE_KERNEL, explicit_euler_descriptor,
    laplacian_descriptor, scale_descriptor,
};
pub use lowering::*;
pub use mesh::FvmLineMeshView;
pub use plan::{Diffusion1dPlan, DiffusionResources};

pub use native::register_native_openmp_kernels;
pub use portable::register_portable_kernels;
