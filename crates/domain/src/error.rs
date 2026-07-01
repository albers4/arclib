// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum DomainError {
    EmptyName,
    NonFiniteCoordinate,
    DegenerateLine,
    DegeneratePlane,
    DegenerateAxis,
    InvalidExtent {
        name: &'static str,
        value: f64,
    },
    InvalidBounds {
        axis: &'static str,
        lower: f64,
        upper: f64,
    },
    InvalidRadius {
        name: &'static str,
        value: f64,
    },
    InvalidTorusRadii {
        major: f64,
        minor: f64,
    },
}

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(formatter, "domain name must not be empty"),
            Self::NonFiniteCoordinate => {
                write!(formatter, "domain coordinates must be finite")
            }
            Self::DegenerateLine => {
                write!(formatter, "line endpoints must not coincide")
            }
            Self::DegeneratePlane => write!(
                formatter,
                "plane basis vectors must be nonzero and linearly independent",
            ),
            Self::DegenerateAxis => {
                write!(formatter, "axis vector must be nonzero")
            }
            Self::InvalidExtent { name, value } => {
                write!(
                    formatter,
                    "extent '{name}' must be finite and positive, got {value}"
                )
            }
            Self::InvalidBounds { axis, lower, upper } => write!(
                formatter,
                "box bounds on {axis} require lower < upper, got {lower} >= {upper}",
            ),
            Self::InvalidRadius { name, value } => {
                write!(
                    formatter,
                    "radius '{name}' must be finite and positive, got {value}"
                )
            }
            Self::InvalidTorusRadii { major, minor } => write!(
                formatter,
                "a ring torus requires major radius > minor radius > 0, got {major} and {minor}",
            ),
        }
    }
}

impl Error for DomainError {}
