// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use ir::{
    GreedyRewriteConfig, Module, OperationBuilder, OperationId, PatternBenefit, PatternResult,
    PatternRewriter, RewriteError, RewritePattern, RewritePatternSet, Type, ValueProducer,
    apply_patterns_greedily,
};

struct ReplaceOldPattern;

impl RewritePattern for ReplaceOldPattern {
    fn name(&self) -> &'static str {
        "ReplaceOldPattern"
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<PatternResult, RewriteError> {
        let operand = rewriter.operation(operation).unwrap().operand(0).unwrap();

        let replacement = rewriter.create_operation(
            OperationBuilder::new("test.new").result(Type::f64()),
            [operand],
        )?;

        let result = rewriter.operation(replacement).unwrap().result(0).unwrap();

        rewriter.replace_operation(operation, &[result])?;

        Ok(PatternResult::Rewritten)
    }
}

#[test]
fn creates_and_replaces_operations() {
    let mut module = Module::new();

    let input = module
        .append_operation(OperationBuilder::new("test.input").result(Type::f64()), [])
        .unwrap();

    let input_value = module.operation(input).unwrap().result(0).unwrap();

    let old = module
        .append_operation(
            OperationBuilder::new("test.old").result(Type::f64()),
            [input_value],
        )
        .unwrap();

    let old_result = module.operation(old).unwrap().result(0).unwrap();

    let consumer = module
        .append_operation(OperationBuilder::new("test.consumer"), [old_result])
        .unwrap();

    let mut patterns = RewritePatternSet::new();

    patterns.add("test.old", PatternBenefit::DEFAULT, ReplaceOldPattern);

    apply_patterns_greedily(&mut module, &patterns, &GreedyRewriteConfig::default()).unwrap();

    assert!(module.operation(old).is_none());

    let replacement_value = module.operation(consumer).unwrap().operand(0).unwrap();

    let ValueProducer::OperationResult {
        operation: replacement,
        ..
    } = module.value(replacement_value).unwrap().producer()
    else {
        panic!("expected operation result");
    };

    assert_eq!(
        module.operation(replacement).unwrap().name().as_str(),
        "test.new",
    );

    module.verify().unwrap();
}
