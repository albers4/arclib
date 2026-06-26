// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::{IrError, Module, OperationId};

use super::{PatternResult, PatternRewriter, RewriteError, RewritePatternSet};

#[derive(Debug, Clone)]
pub enum PatternApplication {
    NoMatch,

    Rewritten {
        created: Vec<OperationId>,
        erased: Vec<OperationId>,
    },
}

impl PatternApplication {
    pub fn was_rewritten(&self) -> bool {
        matches!(self, Self::Rewritten { .. })
    }
}

pub fn apply_patterns_to_operation(
    module: &mut Module,
    operation: OperationId,
    patterns: &RewritePatternSet,
) -> Result<PatternApplication, RewriteError> {
    let name = module
        .operation(operation)
        .ok_or(IrError::MissingOperation(operation))?
        .name()
        .clone();

    for registered in patterns.candidates(&name) {
        if module.operation(operation).is_none() {
            break;
        }

        let pattern = registered.pattern();

        let mut rewriter = PatternRewriter::new(module, operation)?;

        let result = pattern.match_and_rewrite(operation, &mut rewriter)?;

        match (result, rewriter.changed()) {
            (PatternResult::NoMatch, false) => {}

            (PatternResult::NoMatch, true) => {
                return Err(RewriteError::PatternChangedOnNoMatch {
                    pattern: pattern.name(),
                    operation,
                });
            }

            (PatternResult::Rewritten, false) => {
                return Err(RewriteError::PatternReportedRewriteWithoutChange {
                    pattern: pattern.name(),
                    operation,
                });
            }

            (PatternResult::Rewritten, true) => {
                return Ok(PatternApplication::Rewritten {
                    created: rewriter.created_operations().to_vec(),

                    erased: rewriter.erased_operations().to_vec(),
                });
            }
        }
    }

    Ok(PatternApplication::NoMatch)
}
