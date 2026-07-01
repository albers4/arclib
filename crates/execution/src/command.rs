// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use kernel::{KernelAccess, KernelDescriptor};

use crate::{ExecutionError, ResourceId};

#[derive(Debug, Clone)]
pub struct KernelInvocation {
    descriptor: Arc<KernelDescriptor>,
    arguments: Vec<ResourceId>,
}

impl KernelInvocation {
    pub fn new(
        descriptor: Arc<KernelDescriptor>,
        arguments: impl IntoIterator<Item = ResourceId>,
    ) -> Result<Self, ExecutionError> {
        let arguments: Vec<_> = arguments.into_iter().collect();
        let expected = descriptor.abi().parameters().len();

        if arguments.len() != expected {
            return Err(ExecutionError::ArgumentCountMismatch {
                kernel: descriptor.name().to_owned(),
                expected,
                actual: arguments.len(),
            });
        }

        Ok(Self {
            descriptor,
            arguments,
        })
    }

    pub fn descriptor(&self) -> &Arc<KernelDescriptor> {
        &self.descriptor
    }

    pub fn arguments(&self) -> &[ResourceId] {
        &self.arguments
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferCopy {
    source: ResourceId,
    destination: ResourceId,
    bytes: usize,
}

impl BufferCopy {
    pub fn new(
        source: ResourceId,
        destination: ResourceId,
        bytes: usize,
    ) -> Result<Self, ExecutionError> {
        if bytes == 0 {
            return Err(ExecutionError::InvalidByteCount {
                command: "buffer copy",
                bytes,
            });
        }

        Ok(Self {
            source,
            destination,
            bytes,
        })
    }

    pub const fn source(self) -> ResourceId {
        self.source
    }

    pub const fn destination(self) -> ResourceId {
        self.destination
    }

    pub const fn bytes(self) -> usize {
        self.bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferFill {
    destination: ResourceId,
    value: u8,
    bytes: usize,
}

impl BufferFill {
    pub fn new(
        destination: ResourceId,
        value: u8,
        bytes: usize,
    ) -> Result<Self, ExecutionError> {
        if bytes == 0 {
            return Err(ExecutionError::InvalidByteCount {
                command: "buffer fill",
                bytes,
            });
        }

        Ok(Self {
            destination,
            value,
            bytes,
        })
    }

    pub const fn destination(self) -> ResourceId {
        self.destination
    }

    pub const fn value(self) -> u8 {
        self.value
    }

    pub const fn bytes(self) -> usize {
        self.bytes
    }
}

#[derive(Debug, Clone)]
pub enum ExecutionCommand {
    Kernel(KernelInvocation),
    Copy(BufferCopy),
    Fill(BufferFill),
    Barrier,
}

impl ExecutionCommand {
    pub fn kernel(invocation: KernelInvocation) -> Self {
        Self::Kernel(invocation)
    }

    pub fn copy(copy: BufferCopy) -> Self {
        Self::Copy(copy)
    }

    pub fn fill(fill: BufferFill) -> Self {
        Self::Fill(fill)
    }

    pub const fn barrier() -> Self {
        Self::Barrier
    }

    pub fn as_kernel(&self) -> Option<&KernelInvocation> {
        match self {
            Self::Kernel(invocation) => Some(invocation),
            _ => None,
        }
    }

    pub(crate) fn accesses(&self) -> Vec<(ResourceId, KernelAccess)> {
        match self {
            Self::Kernel(invocation) => invocation
                .descriptor()
                .abi()
                .parameters()
                .iter()
                .zip(invocation.arguments().iter().copied())
                .map(|(parameter, resource)| (resource, parameter.access()))
                .collect(),

            Self::Copy(copy) if copy.source == copy.destination => {
                vec![(copy.source, KernelAccess::ReadWrite)]
            }

            Self::Copy(copy) => vec![
                (copy.source, KernelAccess::Read),
                (copy.destination, KernelAccess::Write),
            ],

            Self::Fill(fill) => vec![(fill.destination, KernelAccess::Write)],
            Self::Barrier => Vec::new(),
        }
    }
}
