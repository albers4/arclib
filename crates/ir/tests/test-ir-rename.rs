// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use ir::{
    GreedyRewriteConfig, Module, OperationBuilder, OperationId, PatternBenefit, PatternResult,
    PatternRewriter, RewriteError, RewritePattern, RewritePatternSet, apply_patterns_greedily,
};

struct RenamePattern {
    from: &'static str,
    to: &'static str,
}

impl RewritePattern for RenamePattern {
    fn name(&self) -> &'static str {
        "RenamePattern"
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<PatternResult, RewriteError> {
        if rewriter.operation(operation).unwrap().name().as_str() != self.from {
            return Ok(PatternResult::NoMatch);
        }

        let replacement = rewriter.create_operation(OperationBuilder::new(self.to), [])?;

        rewriter.replace_operation(operation, &[])?;

        assert!(rewriter.operation(replacement).is_some());

        Ok(PatternResult::Rewritten)
    }
}

#[test]
fn repeats_until_convergence() {
    let mut module = Module::new();

    module
        .append_operation(OperationBuilder::new("test.a"), [])
        .unwrap();

    let mut patterns = RewritePatternSet::new();

    patterns.add(
        "test.a",
        PatternBenefit::DEFAULT,
        RenamePattern {
            from: "test.a",
            to: "test.b",
        },
    );

    patterns.add(
        "test.b",
        PatternBenefit::DEFAULT,
        RenamePattern {
            from: "test.b",
            to: "test.c",
        },
    );

    let report =
        apply_patterns_greedily(&mut module, &patterns, &GreedyRewriteConfig::default()).unwrap();

    assert_eq!(report.rewrites, 2);

    let names: Vec<_> = module
        .storage()
        .operation_ids()
        .filter_map(|operation| {
            module
                .operation(operation)
                .map(|operation| operation.name().as_str().to_owned())
        })
        .collect();

    assert!(names.iter().any(|name| { name == "test.c" }));

    assert!(
        !names
            .iter()
            .any(|name| { name == "test.a" || name == "test.b" })
    );

    module.verify().unwrap();
}
