// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

#[derive(Debug)]
pub enum RegistryError {
    DuplicateExtension(String),

    DuplicatePass(String),

    MissingPass(String),

    DuplicatePipeline(String),

    MissingPipeline(String),

    InvalidName { kind: &'static str, name: String },

    Dialect(ir::DialectRegistryError),

    DuplicateConversion(String),

    InvalidConversion { name: String, message: String },
}

impl From<ir::DialectRegistryError> for RegistryError {
    fn from(error: ir::DialectRegistryError) -> Self {
        Self::Dialect(error)
    }
}

impl fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateExtension(name) => {
                write!(
                    formatter,
                    "compiler extension '{name}' is already registered"
                )
            }

            Self::DuplicatePass(name) => {
                write!(formatter, "compiler pass '{name}' is already registered")
            }

            Self::MissingPass(name) => {
                write!(formatter, "compiler pass '{name}' is not registered")
            }

            Self::DuplicatePipeline(name) => {
                write!(
                    formatter,
                    "compiler pipeline '{name}' is already registered"
                )
            }

            Self::MissingPipeline(name) => {
                write!(formatter, "compiler pipeline '{name}' is not registered")
            }

            Self::InvalidName { kind, name } => {
                write!(formatter, "invalid {kind} name '{name}'")
            }

            Self::Dialect(error) => {
                write!(formatter, "{error}")
            }

            Self::DuplicateConversion(name) => {
                write!(
                    formatter,
                    "conversion edge '{name}' \
                    is already registered"
                )
            }

            Self::InvalidConversion { name, message } => {
                write!(
                    formatter,
                    "invalid conversion edge \
                    '{name}': {message}"
                )
            }
        }
    }
}

impl Error for RegistryError {}
