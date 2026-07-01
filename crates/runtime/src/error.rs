// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

use execution::{MemorySpace, ResourceId};
use kernel::{KernelBackend, KernelValueKind};

#[derive(Debug)]
pub enum KernelRuntimeError {
    DuplicateRuntimeFunction(String),
    MissingRuntimeFunction(String),
    RuntimeFailure { kernel: String, status: i32 },
}

impl fmt::Display for KernelRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateRuntimeFunction(kernel) => write!(
                formatter,
                "runtime function for kernel '{kernel}' is already registered"
            ),
            Self::MissingRuntimeFunction(kernel) => write!(
                formatter,
                "no runtime function is registered for kernel '{kernel}'"
            ),
            Self::RuntimeFailure { kernel, status } => {
                write!(
                    formatter,
                    "kernel '{kernel}' returned error status {status}"
                )
            }
        }
    }
}

impl Error for KernelRuntimeError {}

#[derive(Debug)]
pub enum RuntimeError {
    Kernel(KernelRuntimeError),
    NullBufferPointer,
    MissingResource(ResourceId),
    ResourceKindMismatch {
        resource: ResourceId,
        expected: KernelValueKind,
        actual: KernelValueKind,
    },
    ResourceAlreadyBound(ResourceId),
    UnmaterializedBuffer(ResourceId),
    BufferTooSmall {
        resource: ResourceId,
        required: usize,
        actual: usize,
    },
    BufferSpaceMismatch {
        resource: ResourceId,
        expected: MemorySpace,
        actual: MemorySpace,
    },
    BufferMisaligned {
        resource: ResourceId,
        required: usize,
        address: usize,
    },
    MissingAllocator(MemorySpace),
    AllocationFailed {
        memory_space: MemorySpace,
        bytes: usize,
        message: String,
    },
    ResourceTableMismatch,
    UnsupportedMemorySpace {
        backend: KernelBackend,
        memory_space: MemorySpace,
    },
    MissingCudaDevice,
    CudaDeviceMismatch {
        active: u32,
        buffer: u32,
    },
    UnsupportedBufferOperation {
        operation: &'static str,
        memory_space: MemorySpace,
    },
}

impl From<KernelRuntimeError> for RuntimeError {
    fn from(error: KernelRuntimeError) -> Self {
        Self::Kernel(error)
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kernel(error) => write!(formatter, "{error}"),
            Self::NullBufferPointer => write!(formatter, "buffer pointer must not be null"),
            Self::MissingResource(resource) => {
                write!(formatter, "runtime resource {resource:?} does not exist")
            }
            Self::ResourceKindMismatch {
                resource,
                expected,
                actual,
            } => write!(
                formatter,
                "resource {resource:?} has kind {actual:?}; expected {expected:?}"
            ),
            Self::ResourceAlreadyBound(resource) => {
                write!(formatter, "buffer resource {resource:?} is already bound")
            }
            Self::UnmaterializedBuffer(resource) => write!(
                formatter,
                "buffer resource {resource:?} has not been materialized or externally bound"
            ),
            Self::BufferTooSmall {
                resource,
                required,
                actual,
            } => write!(
                formatter,
                "buffer for resource {resource:?} has {actual} bytes; at least {required} are required"
            ),
            Self::BufferSpaceMismatch {
                resource,
                expected,
                actual,
            } => write!(
                formatter,
                "buffer for resource {resource:?} is in {actual:?}; expected {expected:?}"
            ),
            Self::BufferMisaligned {
                resource,
                required,
                address,
            } => write!(
                formatter,
                "buffer for resource {resource:?} at address {address:#x} is not aligned to {required} bytes"
            ),
            Self::MissingAllocator(memory_space) => {
                write!(
                    formatter,
                    "no allocator supports memory space {memory_space:?}"
                )
            }
            Self::AllocationFailed {
                memory_space,
                bytes,
                message,
            } => write!(
                formatter,
                "failed to allocate {bytes} bytes in {memory_space:?}: {message}"
            ),
            Self::ResourceTableMismatch => write!(
                formatter,
                "runtime resource store was created for a different execution plan"
            ),
            Self::UnsupportedMemorySpace {
                backend,
                memory_space,
            } => write!(
                formatter,
                "memory space {memory_space:?} is not supported by backend {backend:?}"
            ),
            Self::MissingCudaDevice => write!(
                formatter,
                "a CUDA kernel was scheduled without an active CUDA device ordinal"
            ),
            Self::CudaDeviceMismatch { active, buffer } => write!(
                formatter,
                "CUDA device buffer belongs to device {buffer}, but active device is {active}"
            ),
            Self::UnsupportedBufferOperation {
                operation,
                memory_space,
            } => write!(
                formatter,
                "runtime does not implement {operation} for memory space {memory_space:?}"
            ),
        }
    }
}

impl Error for RuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Kernel(error) => Some(error),
            _ => None,
        }
    }
}
