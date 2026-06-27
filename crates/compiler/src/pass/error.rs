// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

use crate::{Diagnostic, PolicyDecision};

#[derive(Debug)]
pub enum PassError {
    Failed { pass: &'static str, message: String },
}

impl PassError {
    pub fn failed(pass: &'static str, message: impl Into<String>) -> Self {
        Self::Failed {
            pass,
            message: message.into(),
        }
    }

    pub fn pass(&self) -> &'static str {
        match self {
            Self::Failed { pass, .. } => pass,
        }
    }
}

impl fmt::Display for PassError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failed { pass, message } => {
                write!(formatter, "pass '{pass}' failed: {message}")
            }
        }
    }
}

impl Error for PassError {}

#[derive(Debug)]
pub struct PassFailure {
    error: PassError,
    diagnostics: Vec<Diagnostic>,
    decisions: Vec<PolicyDecision>,
}

impl PassFailure {
    pub(crate) fn new(
        error: PassError,
        diagnostics: Vec<Diagnostic>,
        decisions: Vec<PolicyDecision>,
    ) -> Self {
        Self {
            error,
            diagnostics,
            decisions,
        }
    }

    pub fn error(&self) -> &PassError {
        &self.error
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn decisions(&self) -> &[PolicyDecision] {
        &self.decisions
    }
}

impl fmt::Display for PassFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.error)
    }
}

impl Error for PassFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}
