// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::ffi::c_void;

use runtime::LocalExecutor;

use crate::{EXPLICIT_EULER_KERNEL, FvmError, LAPLACIAN_KERNEL, SCALE_KERNEL};

pub fn register_portable_kernels(executor: &mut LocalExecutor) -> Result<(), FvmError> {
    executor
        .kernels_mut()
        .register(LAPLACIAN_KERNEL, portable_laplacian)
        .map_err(|error| FvmError::Runtime(error.to_string()))?;
    executor
        .kernels_mut()
        .register(SCALE_KERNEL, portable_scale)
        .map_err(|error| FvmError::Runtime(error.to_string()))?;
    executor
        .kernels_mut()
        .register(EXPLICIT_EULER_KERNEL, portable_explicit_euler)
        .map_err(|error| FvmError::Runtime(error.to_string()))?;
    Ok(())
}

unsafe extern "C" fn portable_laplacian(
    arguments: *const *mut c_void,
    argument_count: usize,
) -> i32 {
    if argument_count != 8 {
        return 1;
    }
    let field = unsafe { argument::<f64>(arguments, 0) };
    let owner = unsafe { argument::<i32>(arguments, 1) };
    let neighbour = unsafe { argument::<i32>(arguments, 2) };
    let coefficients = unsafe { argument::<f64>(arguments, 3) };
    let volumes = unsafe { argument::<f64>(arguments, 4) };
    let destination = unsafe { argument::<f64>(arguments, 5) };
    let cell_count = unsafe { *argument::<i64>(arguments, 6) };
    let face_count = unsafe { *argument::<i64>(arguments, 7) };
    if cell_count <= 0 || face_count < 0 {
        return 2;
    }
    let cells = cell_count as usize;
    let faces = face_count as usize;
    for cell in 0..cells {
        unsafe { *destination.add(cell) = 0.0 };
    }
    for face in 0..faces {
        let owner_cell = unsafe { *owner.add(face) } as usize;
        let neighbour_cell = unsafe { *neighbour.add(face) } as usize;
        if owner_cell >= cells || neighbour_cell >= cells {
            return 3;
        }
        let flux = unsafe {
            *coefficients.add(face) * (*field.add(neighbour_cell) - *field.add(owner_cell))
        };
        unsafe {
            *destination.add(owner_cell) += flux / *volumes.add(owner_cell);
            *destination.add(neighbour_cell) -= flux / *volumes.add(neighbour_cell);
        }
    }
    0
}

unsafe extern "C" fn portable_scale(arguments: *const *mut c_void, argument_count: usize) -> i32 {
    if argument_count != 4 {
        return 1;
    }
    let field = unsafe { argument::<f64>(arguments, 0) };
    let factor = unsafe { *argument::<f64>(arguments, 1) };
    let destination = unsafe { argument::<f64>(arguments, 2) };
    let cell_count = unsafe { *argument::<i64>(arguments, 3) };
    if cell_count <= 0 || !factor.is_finite() {
        return 2;
    }
    for cell in 0..cell_count as usize {
        unsafe { *destination.add(cell) = *field.add(cell) * factor };
    }
    0
}

unsafe extern "C" fn portable_explicit_euler(
    arguments: *const *mut c_void,
    argument_count: usize,
) -> i32 {
    if argument_count != 5 {
        return 1;
    }
    let state = unsafe { argument::<f64>(arguments, 0) };
    let rhs = unsafe { argument::<f64>(arguments, 1) };
    let time_step = unsafe { *argument::<f64>(arguments, 2) };
    let destination = unsafe { argument::<f64>(arguments, 3) };
    let cell_count = unsafe { *argument::<i64>(arguments, 4) };
    if cell_count <= 0 || !time_step.is_finite() {
        return 2;
    }
    for cell in 0..cell_count as usize {
        unsafe { *destination.add(cell) = *state.add(cell) + time_step * *rhs.add(cell) };
    }
    0
}

unsafe fn argument<T>(arguments: *const *mut c_void, index: usize) -> *mut T {
    unsafe { *arguments.add(index) }.cast::<T>()
}
