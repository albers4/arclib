// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod allocator;
mod buffer;
mod store;

pub use allocator::{
    AllocatorRegistry, BufferAllocator, CallbackAllocator, HostAllocator,
    materialize_execution_plan, materialize_memory_plan,
};
pub use buffer::{BufferAllocation, BufferBinding};
pub use store::{ResourceStore, RuntimeResource};
