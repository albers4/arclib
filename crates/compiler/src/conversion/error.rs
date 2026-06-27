// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

use ir::{OperationId, OperationName, RewriteError};

use crate::conversion::TypeConversionError;

use super::Legality;

#[derive(Debug)]
pub enum ConversionError {
    Rewrite(RewriteError),

    DynamicLegalityFailed {
        operation: OperationId,
        name: OperationName,
        message: String,
    },

    UnlegalizableOperation {
        operation: OperationId,
        name: OperationName,
        legality: Legality,
    },

    RewriteLimitExceeded {
        limit: usize,
    },

    Type(TypeConversionError),
}

impl ConversionError {
    pub fn operation(&self) -> Option<OperationId> {
        match self {
            Self::Rewrite(_) => None,

            Self::DynamicLegalityFailed { operation, .. }
            | Self::UnlegalizableOperation { operation, .. } => Some(*operation),

            Self::RewriteLimitExceeded { .. } | Self::Type(_) => None,
        }
    }
}

impl From<RewriteError> for ConversionError {
    fn from(error: RewriteError) -> Self {
        Self::Rewrite(error)
    }
}

impl fmt::Display for ConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rewrite(error) => {
                write!(formatter, "{error}")
            }

            Self::DynamicLegalityFailed {
                operation,
                name,
                message,
            } => {
                write!(
                    formatter,
                    "dynamic legality check failed for \
                     operation {operation:?} ('{name}'): \
                     {message}"
                )
            }

            Self::UnlegalizableOperation {
                operation,
                name,
                legality,
            } => {
                write!(
                    formatter,
                    "operation {operation:?} ('{name}') \
                     could not be legalized; classification: \
                     {legality:?}"
                )
            }

            Self::RewriteLimitExceeded { limit } => {
                write!(
                    formatter,
                    "conversion exceeded the rewrite limit \
                     of {limit}"
                )
            }

            Self::Type(error) => {
                write!(formatter, "{error}")
            }
        }
    }
}

impl Error for ConversionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Rewrite(error) => Some(error),
            Self::Type(error) => Some(error),
            _ => None,
        }
    }
}

impl From<TypeConversionError> for ConversionError {
    fn from(error: TypeConversionError) -> Self {
        Self::Type(error)
    }
}
