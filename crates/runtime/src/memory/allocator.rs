// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{
    alloc::{Layout, alloc, dealloc},
    ffi::c_void,
    fmt,
    ptr::NonNull,
    sync::Arc,
};

use execution::{BufferSpec, ExecutionPlan, MemoryPlan, MemorySpace};

use crate::memory::store::validate_binding;
use crate::{BufferAllocation, BufferBinding, ResourceStore, RuntimeError};

pub trait BufferAllocator: fmt::Debug + Send + Sync {
    fn supports(&self, memory_space: MemorySpace) -> bool;
    fn allocate(&self, spec: &BufferSpec) -> Result<BufferBinding, RuntimeError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HostAllocator;

#[derive(Debug)]
struct HostAllocation {
    pointer: NonNull<c_void>,
    layout: Layout,
}

// Safety: this allocation is uniquely owned and deallocated only after the last
// binding is dropped. Access synchronization is the executor's responsibility.
unsafe impl Send for HostAllocation {}
unsafe impl Sync for HostAllocation {}

impl BufferAllocation for HostAllocation {
    fn pointer(&self) -> NonNull<c_void> {
        self.pointer
    }

    fn bytes(&self) -> usize {
        self.layout.size()
    }

    fn memory_space(&self) -> MemorySpace {
        MemorySpace::Host
    }
}

impl Drop for HostAllocation {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.pointer.as_ptr().cast(), self.layout);
        }
    }
}

impl BufferAllocator for HostAllocator {
    fn supports(&self, memory_space: MemorySpace) -> bool {
        memory_space == MemorySpace::Host
    }

    fn allocate(&self, spec: &BufferSpec) -> Result<BufferBinding, RuntimeError> {
        let layout = Layout::from_size_align(spec.bytes(), spec.alignment()).map_err(|error| {
            RuntimeError::AllocationFailed {
                memory_space: spec.memory_space(),
                bytes: spec.bytes(),
                message: error.to_string(),
            }
        })?;
        let pointer = NonNull::new(unsafe { alloc(layout) }).ok_or_else(|| {
            RuntimeError::AllocationFailed {
                memory_space: spec.memory_space(),
                bytes: spec.bytes(),
                message: "allocator returned a null pointer".into(),
            }
        })?;

        Ok(BufferBinding::from_allocation(Arc::new(HostAllocation {
            pointer: pointer.cast(),
            layout,
        })))
    }
}

type AllocateCallback = Arc<
    dyn Fn(&BufferSpec) -> Result<NonNull<c_void>, String> + Send + Sync,
>;
type FreeCallback = Arc<dyn Fn(NonNull<c_void>, &BufferSpec) + Send + Sync>;

pub struct CallbackAllocator {
    memory_space: MemorySpace,
    allocate: AllocateCallback,
    free: FreeCallback,
}

impl CallbackAllocator {
    pub fn new<A, F>(memory_space: MemorySpace, allocate: A, free: F) -> Self
    where
        A: Fn(&BufferSpec) -> Result<NonNull<c_void>, String> + Send + Sync + 'static,
        F: Fn(NonNull<c_void>, &BufferSpec) + Send + Sync + 'static,
    {
        Self {
            memory_space,
            allocate: Arc::new(allocate),
            free: Arc::new(free),
        }
    }
}

impl fmt::Debug for CallbackAllocator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CallbackAllocator")
            .field("memory_space", &self.memory_space)
            .finish_non_exhaustive()
    }
}

struct CallbackAllocation {
    pointer: NonNull<c_void>,
    spec: BufferSpec,
    free: FreeCallback,
}

unsafe impl Send for CallbackAllocation {}
unsafe impl Sync for CallbackAllocation {}

impl fmt::Debug for CallbackAllocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CallbackAllocation")
            .field("pointer", &self.pointer)
            .field("spec", &self.spec)
            .finish_non_exhaustive()
    }
}

impl BufferAllocation for CallbackAllocation {
    fn pointer(&self) -> NonNull<c_void> {
        self.pointer
    }

    fn bytes(&self) -> usize {
        self.spec.bytes()
    }

    fn memory_space(&self) -> MemorySpace {
        self.spec.memory_space()
    }
}

impl Drop for CallbackAllocation {
    fn drop(&mut self) {
        (self.free)(self.pointer, &self.spec);
    }
}

impl BufferAllocator for CallbackAllocator {
    fn supports(&self, memory_space: MemorySpace) -> bool {
        memory_space == self.memory_space
    }

    fn allocate(&self, spec: &BufferSpec) -> Result<BufferBinding, RuntimeError> {
        let pointer = (self.allocate)(spec).map_err(|message| {
            RuntimeError::AllocationFailed {
                memory_space: spec.memory_space(),
                bytes: spec.bytes(),
                message,
            }
        })?;

        Ok(BufferBinding::from_allocation(Arc::new(CallbackAllocation {
            pointer,
            spec: spec.clone(),
            free: self.free.clone(),
        })))
    }
}

#[derive(Default)]
pub struct AllocatorRegistry {
    allocators: Vec<Arc<dyn BufferAllocator>>,
}

impl AllocatorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_host_allocator() -> Self {
        let mut registry = Self::new();
        registry.register(HostAllocator);
        registry
    }

    pub fn register<A>(&mut self, allocator: A)
    where
        A: BufferAllocator + 'static,
    {
        self.allocators.push(Arc::new(allocator));
    }

    pub fn allocate(&self, spec: &BufferSpec) -> Result<BufferBinding, RuntimeError> {
        let allocator = self
            .allocators
            .iter()
            .find(|allocator| allocator.supports(spec.memory_space()))
            .ok_or(RuntimeError::MissingAllocator(spec.memory_space()))?;
        allocator.allocate(spec)
    }
}

pub fn materialize_execution_plan(
    plan: &ExecutionPlan,
    resources: &mut ResourceStore,
    allocators: &AllocatorRegistry,
) -> Result<(), RuntimeError> {
    if resources.declarations() != plan.resources() {
        return Err(RuntimeError::ResourceTableMismatch);
    }
    materialize_memory_plan(plan.memory(), resources, allocators)
}

/// Allocates every slot before mutating `resources`. Allocation or validation
/// failure therefore leaves the resource store unchanged.
pub fn materialize_memory_plan(
    plan: &MemoryPlan,
    resources: &mut ResourceStore,
    allocators: &AllocatorRegistry,
) -> Result<(), RuntimeError> {
    for slot in plan.slots() {
        for resource in slot.resources() {
            resources.ensure_unbound(*resource)?;
        }
    }

    let mut allocated = Vec::with_capacity(plan.slots().len());
    for slot in plan.slots() {
        let binding = allocators.allocate(slot.spec())?;
        for resource in slot.resources() {
            let spec = resources.buffer_spec(*resource)?;
            validate_binding(*resource, spec, &binding)?;
        }
        allocated.push((slot.resources().to_vec(), binding));
    }

    for (resource_ids, binding) in allocated {
        for resource in resource_ids {
            resources.commit_buffer(resource, binding.clone());
        }
    }

    Ok(())
}
