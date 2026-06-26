// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

use crate::{OperationId, OperationName};

#[derive(Debug)]
pub enum DialectRegistryError {
    InvalidDialectName(String),

    InvalidOperationName(OperationName),

    DuplicateDialect(String),

    MissingDialect(String),

    DuplicateOperation(OperationName),

    OperationDialectMismatch {
        dialect: String,
        operation: OperationName,
    },

    UnknownDialect {
        operation: OperationId,
        dialect: String,
    },

    UnknownOperation {
        operation: OperationId,
        name: OperationName,
    },

    OperationVerificationFailed {
        operation: OperationId,
        name: OperationName,
        message: String,
    },
}

impl fmt::Display for DialectRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDialectName(name) => {
                write!(formatter, "invalid dialect namespace '{name}'")
            }

            Self::InvalidOperationName(name) => {
                write!(formatter, "invalid operation name '{name}'")
            }

            Self::DuplicateDialect(name) => {
                write!(formatter, "dialect '{name}' is already registered")
            }

            Self::MissingDialect(name) => {
                write!(formatter, "dialect '{name}' is not registered")
            }

            Self::DuplicateOperation(name) => {
                write!(formatter, "operation '{name}' is already registered")
            }

            Self::OperationDialectMismatch { dialect, operation } => {
                write!(
                    formatter,
                    "operation '{operation}' does not belong \
                     to dialect '{dialect}'"
                )
            }

            Self::UnknownDialect { operation, dialect } => {
                write!(
                    formatter,
                    "operation {operation:?} uses unknown \
                     dialect '{dialect}'"
                )
            }

            Self::UnknownOperation { operation, name } => {
                write!(
                    formatter,
                    "operation {operation:?} has unknown \
                     operation name '{name}'"
                )
            }

            Self::OperationVerificationFailed {
                operation,
                name,
                message,
            } => {
                write!(
                    formatter,
                    "verification of operation \
                     {operation:?} ('{name}') failed: {message}"
                )
            }
        }
    }
}

impl Error for DialectRegistryError {}
