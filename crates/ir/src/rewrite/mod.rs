// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod application;
mod error;
mod greedy;
mod pattern;
mod rewriter;

pub use error::RewriteError;

pub use greedy::{GreedyRewriteConfig, GreedyRewriteReport, apply_patterns_greedily};

pub use pattern::{PatternBenefit, PatternResult, RewritePattern, RewritePatternSet};

pub use rewriter::PatternRewriter;

pub use application::{PatternApplication, apply_patterns_to_operation};
