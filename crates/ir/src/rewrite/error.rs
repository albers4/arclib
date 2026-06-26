// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

use crate::{IrError, OperationId, RegionId};

#[derive(Debug)]
pub enum RewriteError {
    Ir(IrError),

    ResultCountMismatch {
        operation: OperationId,
        expected: usize,
        provided: usize,
    },

    RegionNotEmptyDuringReplacement {
        operation: OperationId,
        region: RegionId,
    },

    PatternChangedOnNoMatch {
        pattern: &'static str,
        operation: OperationId,
    },

    PatternReportedRewriteWithoutChange {
        pattern: &'static str,
        operation: OperationId,
    },

    RewriteLimitExceeded {
        limit: usize,
    },

    DidNotConverge {
        iterations: usize,
    },

    Message(String),
}

impl RewriteError {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

impl From<IrError> for RewriteError {
    fn from(error: IrError) -> Self {
        Self::Ir(error)
    }
}

impl fmt::Display for RewriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ir(error) => {
                write!(formatter, "{error}")
            }

            Self::ResultCountMismatch {
                operation,
                expected,
                provided,
            } => {
                write!(
                    formatter,
                    "operation {operation:?} has \
                     {expected} results but rewrite \
                     provided {provided} replacements"
                )
            }

            Self::RegionNotEmptyDuringReplacement { operation, region } => {
                write!(
                    formatter,
                    "cannot replace operation \
                    {operation:?}; region {region:?} \
                    still owns blocks"
                )
            }

            Self::PatternChangedOnNoMatch { pattern, operation } => {
                write!(
                    formatter,
                    "pattern '{pattern}' mutated operation \
                     {operation:?} but returned NoMatch"
                )
            }

            Self::PatternReportedRewriteWithoutChange { pattern, operation } => {
                write!(
                    formatter,
                    "pattern '{pattern}' reported a rewrite \
                     of operation {operation:?} without \
                     making a change"
                )
            }

            Self::RewriteLimitExceeded { limit } => {
                write!(formatter, "rewrite limit of {limit} was exceeded")
            }

            Self::DidNotConverge { iterations } => {
                write!(
                    formatter,
                    "greedy rewrite did not converge after \
                     {iterations} iterations"
                )
            }

            Self::Message(message) => formatter.write_str(message),
        }
    }
}

impl Error for RewriteError {}
