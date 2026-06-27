// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

use super::{ConstraintScope, Property};

#[derive(Debug)]
pub enum PolicyError {
    InvalidProperty(String),

    InvalidMetric(String),

    InvalidWeight {
        owner: String,
        weight: f64,
    },

    InvalidValue,

    EmptyOneOf {
        property: Property,
    },

    NonNumericBound {
        property: Property,
    },

    ConflictingRequiredConstraints {
        scope: ConstraintScope,
        property: Property,
    },
}

impl fmt::Display for PolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProperty(name) => {
                write!(formatter, "invalid policy property '{name}'")
            }

            Self::InvalidMetric(name) => {
                write!(formatter, "invalid optimization metric '{name}'")
            }

            Self::InvalidWeight { owner, weight } => {
                write!(
                    formatter,
                    "{owner} has invalid weight {weight}; \
                     weights must be finite and greater than zero"
                )
            }

            Self::InvalidValue => formatter.write_str("policy value contains a non-finite number"),

            Self::EmptyOneOf { property } => {
                write!(
                    formatter,
                    "constraint '{property}' has an empty candidate set"
                )
            }

            Self::NonNumericBound { property } => {
                write!(
                    formatter,
                    "constraint '{property}' uses a non-numeric bound"
                )
            }

            Self::ConflictingRequiredConstraints { scope, property } => {
                write!(
                    formatter,
                    "required constraints conflict for \
                     property '{property}' in scope {scope:?}"
                )
            }
        }
    }
}

impl Error for PolicyError {}
