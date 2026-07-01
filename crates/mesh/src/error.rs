// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum MeshError {
    InvalidResolution { axis: &'static str, cells: usize },
    ResolutionShapeMismatch,
    UnsupportedDomain,
}

impl fmt::Display for MeshError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidResolution { axis, cells } => write!(
                formatter,
                "mesh resolution on axis '{axis}' must be greater than zero, got {cells}",
            ),
            Self::ResolutionShapeMismatch => {
                write!(formatter, "mesh resolution does not match the domain shape")
            }
            Self::UnsupportedDomain => write!(formatter, "unsupported domain geometry"),
        }
    }
}

impl Error for MeshError {}
