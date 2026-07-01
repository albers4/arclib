// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

use kernel::KernelValueKind;

use crate::{ExecutionNodeId, MemoryPlanningError, ResourceId};

#[derive(Debug)]
pub enum ExecutionError {
    ArgumentCountMismatch {
        kernel: String,
        expected: usize,
        actual: usize,
    },
    InvalidByteCount {
        command: &'static str,
        bytes: usize,
    },
    MissingDependency {
        node: ExecutionNodeId,
        dependency: ExecutionNodeId,
    },
    InvalidExplicitDependency {
        dependency: ExecutionNodeId,
        node_count: usize,
    },
    CycleDetected,
    MissingResource(ResourceId),
    ResourceKindMismatch {
        resource: ResourceId,
        expected: KernelValueKind,
        actual: KernelValueKind,
    },
    BufferTooSmall {
        resource: ResourceId,
        required: usize,
        available: usize,
    },
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArgumentCountMismatch {
                kernel,
                expected,
                actual,
            } => write!(
                formatter,
                "kernel '{kernel}' expects {expected} arguments but received {actual}"
            ),
            Self::InvalidByteCount { command, bytes } => {
                write!(formatter, "{command} requires a positive byte count, got {bytes}")
            }
            Self::MissingDependency { node, dependency } => write!(
                formatter,
                "execution node {node:?} depends on missing node {dependency:?}"
            ),
            Self::InvalidExplicitDependency {
                dependency,
                node_count,
            } => write!(
                formatter,
                "explicit dependency {dependency:?} does not refer to one of the {node_count} existing nodes"
            ),
            Self::CycleDetected => write!(formatter, "execution graph contains a cycle"),
            Self::MissingResource(resource) => {
                write!(formatter, "execution resource {resource:?} does not exist")
            }
            Self::ResourceKindMismatch {
                resource,
                expected,
                actual,
            } => write!(
                formatter,
                "resource {resource:?} has kind {actual:?}; expected {expected:?}"
            ),
            Self::BufferTooSmall {
                resource,
                required,
                available,
            } => write!(
                formatter,
                "buffer resource {resource:?} has {available} bytes; {required} bytes are required"
            ),
        }
    }
}

impl Error for ExecutionError {}

#[derive(Debug)]
pub enum ExecutionPlanError {
    Execution(ExecutionError),
    Memory(MemoryPlanningError),
}

impl From<ExecutionError> for ExecutionPlanError {
    fn from(error: ExecutionError) -> Self {
        Self::Execution(error)
    }
}

impl From<MemoryPlanningError> for ExecutionPlanError {
    fn from(error: MemoryPlanningError) -> Self {
        Self::Memory(error)
    }
}

impl fmt::Display for ExecutionPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Execution(error) => write!(formatter, "{error}"),
            Self::Memory(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for ExecutionPlanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Execution(error) => Some(error),
            Self::Memory(error) => Some(error),
        }
    }
}
