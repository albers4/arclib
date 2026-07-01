// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

#[derive(Debug)]
pub enum FvmError {
    MeshRequiresAtLeastTwoCells,
    InitialStateLength { expected: usize, actual: usize },
    InvalidDiffusivity(f64),
    InvalidTimeStep(f64),
    UnstableExplicitStep { maximum: f64, actual: f64 },
    Execution(execution::ExecutionError),
    ExecutionPlan(execution::ExecutionPlanError),
    Runtime(String),
}

impl From<execution::ExecutionError> for FvmError {
    fn from(error: execution::ExecutionError) -> Self {
        Self::Execution(error)
    }
}

impl From<execution::ExecutionPlanError> for FvmError {
    fn from(error: execution::ExecutionPlanError) -> Self {
        Self::ExecutionPlan(error)
    }
}

impl fmt::Display for FvmError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MeshRequiresAtLeastTwoCells => {
                write!(formatter, "the 1D FVM mesh requires at least two cells")
            }
            Self::InitialStateLength { expected, actual } => write!(
                formatter,
                "initial state contains {actual} values; expected {expected}",
            ),
            Self::InvalidDiffusivity(value) => write!(
                formatter,
                "diffusivity must be finite and non-negative, got {value}",
            ),
            Self::InvalidTimeStep(value) => {
                write!(
                    formatter,
                    "time step must be finite and positive, got {value}"
                )
            }
            Self::UnstableExplicitStep { maximum, actual } => write!(
                formatter,
                "explicit diffusion step {actual} exceeds the 1D stability limit {maximum}",
            ),
            Self::Execution(error) => write!(formatter, "{error}"),
            Self::ExecutionPlan(error) => write!(formatter, "{error}"),
            Self::Runtime(message) => write!(formatter, "{message}"),
        }
    }
}

impl Error for FvmError {}
