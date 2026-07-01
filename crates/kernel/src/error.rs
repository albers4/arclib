// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

#[derive(Debug)]
pub enum KernelError {
    DuplicateKernel(String),

    InvalidDescriptor { kernel: String, message: String },
}

impl fmt::Display for KernelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateKernel(kernel) => {
                write!(
                    formatter,
                    "kernel '{kernel}' is \
                     already registered"
                )
            }

            Self::InvalidDescriptor { kernel, message } => {
                write!(
                    formatter,
                    "invalid kernel \
                     '{kernel}': {message}"
                )
            }
        }
    }
}

impl Error for KernelError {}
