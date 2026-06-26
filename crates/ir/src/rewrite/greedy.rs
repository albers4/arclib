// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::{
    Module, OperationId,
    rewrite::{
        error::RewriteError,
        pattern::{PatternResult, RewritePatternSet},
        rewriter::PatternRewriter,
    },
};

#[derive(Debug, Clone)]
pub struct GreedyRewriteConfig {
    pub max_iterations: usize,
    pub max_rewrites: usize,
}

impl Default for GreedyRewriteConfig {
    fn default() -> Self {
        Self {
            max_iterations: 32,
            max_rewrites: 100_000,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GreedyRewriteReport {
    pub iterations: usize,
    pub rewrites: usize,
}

pub fn apply_patterns_greedily(
    module: &mut Module,
    patterns: &RewritePatternSet,
    config: &GreedyRewriteConfig,
) -> Result<GreedyRewriteReport, RewriteError> {
    let mut rewrites = 0;

    for iteration in 1..=config.max_iterations {
        let operations: Vec<OperationId> = module.storage().operation_ids().collect();

        let mut changed_this_iteration = false;

        for operation in operations {
            let Some(name) = module
                .operation(operation)
                .map(|operation| operation.name().clone())
            else {
                continue;
            };

            let candidates = patterns.candidates(&name);

            for registered in candidates {
                if module.operation(operation).is_none() {
                    break;
                }

                let pattern = registered.pattern();

                let mut rewriter = PatternRewriter::new(module, operation)?;

                let result = pattern.match_and_rewrite(operation, &mut rewriter)?;

                let changed = rewriter.changed();

                match (result, changed) {
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
                        rewrites += 1;

                        if rewrites > config.max_rewrites {
                            return Err(RewriteError::RewriteLimitExceeded {
                                limit: config.max_rewrites,
                            });
                        }

                        changed_this_iteration = true;

                        // Restart pattern selection for
                        // this operation in the next scan.
                        break;
                    }
                }
            }
        }

        if !changed_this_iteration {
            return Ok(GreedyRewriteReport {
                iterations: iteration,
                rewrites,
            });
        }
    }

    Err(RewriteError::DidNotConverge {
        iterations: config.max_iterations,
    })
}
