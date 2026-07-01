// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{ffi::c_void, fmt, ptr::NonNull, sync::Arc};

use execution::MemorySpace;

use crate::RuntimeError;

pub trait BufferAllocation: fmt::Debug + Send + Sync {
    fn pointer(&self) -> NonNull<c_void>;
    fn bytes(&self) -> usize;
    fn memory_space(&self) -> MemorySpace;
}

#[derive(Debug)]
struct ExternalBufferAllocation {
    pointer: NonNull<c_void>,
    bytes: usize,
    memory_space: MemorySpace,
}

// Safety: constructing an external allocation is unsafe. The caller guarantees
// that the allocation may be accessed from all execution threads using it.
unsafe impl Send for ExternalBufferAllocation {}
unsafe impl Sync for ExternalBufferAllocation {}

impl BufferAllocation for ExternalBufferAllocation {
    fn pointer(&self) -> NonNull<c_void> {
        self.pointer
    }

    fn bytes(&self) -> usize {
        self.bytes
    }

    fn memory_space(&self) -> MemorySpace {
        self.memory_space
    }
}

#[derive(Clone)]
pub struct BufferBinding {
    allocation: Arc<dyn BufferAllocation>,
}

impl BufferBinding {
    /// # Safety
    ///
    /// `pointer` must remain valid until every clone of the returned binding
    /// has been dropped. The caller remains responsible for deallocation.
    pub unsafe fn external(
        pointer: *mut c_void,
        bytes: usize,
        memory_space: MemorySpace,
    ) -> Result<Self, RuntimeError> {
        let pointer = NonNull::new(pointer).ok_or(RuntimeError::NullBufferPointer)?;
        Ok(Self {
            allocation: Arc::new(ExternalBufferAllocation {
                pointer,
                bytes,
                memory_space,
            }),
        })
    }

    pub fn from_allocation(allocation: Arc<dyn BufferAllocation>) -> Self {
        Self { allocation }
    }

    pub fn pointer(&self) -> NonNull<c_void> {
        self.allocation.pointer()
    }

    pub fn bytes(&self) -> usize {
        self.allocation.bytes()
    }

    pub fn memory_space(&self) -> MemorySpace {
        self.allocation.memory_space()
    }

    pub fn is_aligned_to(&self, alignment: usize) -> bool {
        alignment != 0 && (self.pointer().as_ptr() as usize) % alignment == 0
    }
}

impl fmt::Debug for BufferBinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BufferBinding")
            .field("pointer", &self.pointer())
            .field("bytes", &self.bytes())
            .field("memory_space", &self.memory_space())
            .finish()
    }
}
