// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{collections::HashMap, ffi::c_void};

use crate::KernelRuntimeError;

pub type PackedKernelFunction =
    unsafe extern "C" fn(arguments: *const *mut c_void, argument_count: usize) -> i32;

#[derive(Default)]
pub struct LinkedKernelRuntime {
    functions: HashMap<String, PackedKernelFunction>,
}

impl LinkedKernelRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        kernel: impl AsRef<str>,
        function: PackedKernelFunction,
    ) -> Result<(), KernelRuntimeError> {
        let kernel = kernel.as_ref();
        if self.functions.contains_key(kernel) {
            return Err(KernelRuntimeError::DuplicateRuntimeFunction(
                kernel.to_owned(),
            ));
        }
        self.functions.insert(kernel.to_owned(), function);
        Ok(())
    }

    pub fn contains(&self, kernel: &str) -> bool {
        self.functions.contains_key(kernel)
    }

    /// # Safety
    ///
    /// Every pointer must satisfy the ABI declared by the corresponding
    /// `KernelDescriptor` and remain valid for the complete call.
    pub unsafe fn invoke(
        &self,
        kernel: &str,
        arguments: &mut [*mut c_void],
    ) -> Result<(), KernelRuntimeError> {
        let function = self.functions.get(kernel).copied().ok_or_else(|| {
            KernelRuntimeError::MissingRuntimeFunction(kernel.to_owned())
        })?;

        let status = unsafe { function(arguments.as_ptr(), arguments.len()) };
        if status != 0 {
            return Err(KernelRuntimeError::RuntimeFailure {
                kernel: kernel.to_owned(),
                status,
            });
        }
        Ok(())
    }
}
