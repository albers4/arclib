// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::ffi::c_void;

use runtime::LocalExecutor;

use crate::{EXPLICIT_EULER_KERNEL, FvmError, LAPLACIAN_KERNEL, SCALE_KERNEL};

unsafe extern "C" {
    fn fvm_laplacian_orthogonal_f64_packed(
        arguments: *const *mut c_void,
        argument_count: usize,
    ) -> i32;
    fn fvm_scale_f64_packed(arguments: *const *mut c_void, argument_count: usize) -> i32;
    fn fvm_explicit_euler_f64_packed(arguments: *const *mut c_void, argument_count: usize) -> i32;
}

pub fn register_native_openmp_kernels(executor: &mut LocalExecutor) -> Result<(), FvmError> {
    executor
        .kernels_mut()
        .register(LAPLACIAN_KERNEL, fvm_laplacian_orthogonal_f64_packed)
        .map_err(|error| FvmError::Runtime(error.to_string()))?;
    executor
        .kernels_mut()
        .register(SCALE_KERNEL, fvm_scale_f64_packed)
        .map_err(|error| FvmError::Runtime(error.to_string()))?;
    executor
        .kernels_mut()
        .register(EXPLICIT_EULER_KERNEL, fvm_explicit_euler_f64_packed)
        .map_err(|error| FvmError::Runtime(error.to_string()))?;
    Ok(())
}
