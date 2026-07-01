// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use kernel::{
    KernelAbi, KernelAccess, KernelBackend, KernelDescriptor, KernelParameter, KernelValueKind,
};

pub const LAPLACIAN_KERNEL: &str = "fvm.laplacian.orthogonal.f64";
pub const SCALE_KERNEL: &str = "fvm.scale.f64";
pub const EXPLICIT_EULER_KERNEL: &str = "fvm.explicit-euler.f64";

#[derive(Debug, Clone)]
pub struct FvmKernelSet {
    laplacian: Arc<KernelDescriptor>,
    scale: Arc<KernelDescriptor>,
    explicit_euler: Arc<KernelDescriptor>,
}

impl FvmKernelSet {
    pub fn new(
        laplacian: KernelDescriptor,
        scale: KernelDescriptor,
        explicit_euler: KernelDescriptor,
    ) -> Self {
        Self {
            laplacian: Arc::new(laplacian),
            scale: Arc::new(scale),
            explicit_euler: Arc::new(explicit_euler),
        }
    }

    pub fn cpu_openmp() -> Self {
        Self::new(
            laplacian_descriptor(),
            scale_descriptor(),
            explicit_euler_descriptor(),
        )
    }

    pub fn laplacian(&self) -> &Arc<KernelDescriptor> {
        &self.laplacian
    }
    pub fn scale(&self) -> &Arc<KernelDescriptor> {
        &self.scale
    }
    pub fn explicit_euler(&self) -> &Arc<KernelDescriptor> {
        &self.explicit_euler
    }
}

impl Default for FvmKernelSet {
    fn default() -> Self {
        Self::cpu_openmp()
    }
}

pub fn laplacian_descriptor() -> KernelDescriptor {
    KernelDescriptor::new(
        LAPLACIAN_KERNEL,
        "fvm_laplacian_orthogonal_f64_packed",
        KernelBackend::CpuOpenMp,
    )
    .with_abi(
        KernelAbi::new()
            .parameter(buffer("field", KernelAccess::Read))
            .parameter(buffer("owner", KernelAccess::Read))
            .parameter(buffer("neighbour", KernelAccess::Read))
            .parameter(buffer("face_coefficients", KernelAccess::Read))
            .parameter(buffer("cell_volumes", KernelAccess::Read))
            .parameter(buffer("destination", KernelAccess::Write))
            .parameter(scalar("cell_count"))
            .parameter(scalar("face_count"))
            .result_alias(5),
    )
}

pub fn scale_descriptor() -> KernelDescriptor {
    KernelDescriptor::new(
        SCALE_KERNEL,
        "fvm_scale_f64_packed",
        KernelBackend::CpuOpenMp,
    )
    .with_abi(
        KernelAbi::new()
            .parameter(buffer("field", KernelAccess::Read))
            .parameter(scalar("factor"))
            .parameter(buffer("destination", KernelAccess::Write))
            .parameter(scalar("cell_count"))
            .result_alias(2),
    )
}

pub fn explicit_euler_descriptor() -> KernelDescriptor {
    KernelDescriptor::new(
        EXPLICIT_EULER_KERNEL,
        "fvm_explicit_euler_f64_packed",
        KernelBackend::CpuOpenMp,
    )
    .with_abi(
        KernelAbi::new()
            .parameter(buffer("state", KernelAccess::Read))
            .parameter(buffer("rhs", KernelAccess::Read))
            .parameter(scalar("time_step"))
            .parameter(buffer("destination", KernelAccess::Write))
            .parameter(scalar("cell_count"))
            .result_alias(3),
    )
}

fn buffer(name: &'static str, access: KernelAccess) -> KernelParameter {
    KernelParameter::new(name, KernelValueKind::Buffer, access)
}

fn scalar(name: &'static str) -> KernelParameter {
    KernelParameter::new(name, KernelValueKind::Scalar, KernelAccess::Read)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptors_are_valid() {
        for descriptor in [
            laplacian_descriptor(),
            scale_descriptor(),
            explicit_euler_descriptor(),
        ] {
            descriptor.validate().unwrap();
        }
    }
}
