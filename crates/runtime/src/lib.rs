// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod error;
mod executor;
pub mod kernel;
pub mod memory;

pub use error::{KernelRuntimeError, RuntimeError};
pub use executor::LocalExecutor;
pub use kernel::{LinkedKernelRuntime, PackedKernelFunction};
pub use memory::{
    AllocatorRegistry, BufferAllocation, BufferAllocator, BufferBinding, CallbackAllocator,
    HostAllocator, ResourceStore, RuntimeResource, materialize_execution_plan,
    materialize_memory_plan,
};
