// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use ir::{
    GreedyRewriteConfig, Module, OperationBuilder, OperationId, PatternBenefit, PatternResult,
    PatternRewriter, RewriteError, RewritePattern, RewritePatternSet, apply_patterns_greedily,
};

struct EraseMarkerPattern;

impl RewritePattern for EraseMarkerPattern {
    fn name(&self) -> &'static str {
        "EraseMarkerPattern"
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<PatternResult, RewriteError> {
        rewriter.erase_operation(operation)?;

        Ok(PatternResult::Rewritten)
    }
}

#[test]
fn erases_dead_operation() {
    let mut module = Module::new();

    let marker = module
        .append_operation(OperationBuilder::new("test.marker"), [])
        .unwrap();

    let mut patterns = RewritePatternSet::new();

    patterns.add("test.marker", PatternBenefit::DEFAULT, EraseMarkerPattern);

    apply_patterns_greedily(&mut module, &patterns, &GreedyRewriteConfig::default()).unwrap();

    assert!(module.operation(marker).is_none());

    module.verify().unwrap();
}
