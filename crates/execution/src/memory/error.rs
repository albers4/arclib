// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

use crate::{ExecutionError, ResourceId};

#[derive(Debug)]
pub enum MemoryPlanningError {
    InvalidBufferSpec {
        resource: ResourceId,
        message: String,
    },
    InvalidLifetime {
        resource: ResourceId,
        first_use: usize,
        last_use: usize,
    },
    Execution(ExecutionError),
}

impl From<ExecutionError> for MemoryPlanningError {
    fn from(error: ExecutionError) -> Self {
        Self::Execution(error)
    }
}

impl fmt::Display for MemoryPlanningError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBufferSpec { resource, message } => write!(
                formatter,
                "invalid buffer specification for resource {resource:?}: {message}"
            ),
            Self::InvalidLifetime {
                resource,
                first_use,
                last_use,
            } => write!(
                formatter,
                "resource {resource:?} has invalid lifetime {first_use}..={last_use}"
            ),
            Self::Execution(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for MemoryPlanningError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Execution(error) => Some(error),
            _ => None,
        }
    }
}
