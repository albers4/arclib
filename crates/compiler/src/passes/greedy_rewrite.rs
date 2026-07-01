// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

#[cfg(test)]
mod tests {
    use ir::{
        Attribute, GreedyRewriteConfig, Module, OperationBuilder, OperationId, PatternBenefit,
        PatternResult, PatternRewriter, RewriteError, RewritePattern, RewritePatternSet,
        apply_patterns_greedily,
    };

    use crate::{AnalysisManager, Pass, PassContext, PassError};

    pub struct GreedyRewritePass {
        name: &'static str,
        patterns: RewritePatternSet,
        config: GreedyRewriteConfig,
    }

    impl GreedyRewritePass {
        pub fn new(name: &'static str, patterns: RewritePatternSet) -> Self {
            Self {
                name,
                patterns,
                config: GreedyRewriteConfig::default(),
            }
        }

        pub fn with_config(mut self, config: GreedyRewriteConfig) -> Self {
            self.config = config;
            self
        }
    }

    impl Pass for GreedyRewritePass {
        fn name(&self) -> &'static str {
            self.name
        }

        fn run(
            &self,
            module: &mut Module,
            context: &mut PassContext,
            _analyses: &mut AnalysisManager,
        ) -> Result<(), PassError> {
            let report = apply_patterns_greedily(module, &self.patterns, &self.config)
                .map_err(|error| PassError::failed(self.name(), error.to_string()))?;

            if report.rewrites > 0 {
                context.mark_changed();
            }

            Ok(())
        }
    }

    struct AddFirstMarker;

    impl RewritePattern for AddFirstMarker {
        fn name(&self) -> &'static str {
            "AddFirstMarker"
        }

        fn match_and_rewrite(
            &self,
            operation: OperationId,
            rewriter: &mut PatternRewriter<'_>,
        ) -> Result<PatternResult, RewriteError> {
            let already_marked = matches!(
                rewriter.operation(operation).unwrap().attribute("first"),
                Some(Attribute::Bool(true)),
            );

            if already_marked {
                return Ok(PatternResult::NoMatch);
            }

            rewriter.set_attribute(operation, "first", Attribute::Bool(true))?;

            Ok(PatternResult::Rewritten)
        }
    }

    struct AddSecondMarker;

    impl RewritePattern for AddSecondMarker {
        fn name(&self) -> &'static str {
            "AddSecondMarker"
        }

        fn match_and_rewrite(
            &self,
            operation: OperationId,
            rewriter: &mut PatternRewriter<'_>,
        ) -> Result<PatternResult, RewriteError> {
            let operation_ref = rewriter.operation(operation).unwrap();

            let has_first = matches!(
                operation_ref.attribute("first"),
                Some(Attribute::Bool(true)),
            );

            let has_second = matches!(
                operation_ref.attribute("second"),
                Some(Attribute::Bool(true)),
            );

            if !has_first || has_second {
                return Ok(PatternResult::NoMatch);
            }

            rewriter.set_attribute(operation, "second", Attribute::Bool(true))?;

            Ok(PatternResult::Rewritten)
        }
    }

    #[test]
    fn greedy_rewrite_runs_until_fixpoint() {
        let mut module = Module::new();

        let operation = module
            .append_operation(OperationBuilder::new("test.operation"), [])
            .unwrap();

        let mut patterns = RewritePatternSet::new();

        // Tried first, but cannot match until
        // AddFirstMarker has run.
        patterns.add("test.operation", PatternBenefit::new(100), AddSecondMarker);

        patterns.add("test.operation", PatternBenefit::new(10), AddFirstMarker);

        let pass = GreedyRewritePass::new("test.greedy-rewrite", patterns).with_config(
            GreedyRewriteConfig {
                max_iterations: 64,
                max_rewrites: 1_000_000,
            },
        );

        let mut context = PassContext::default();

        let mut analyses = AnalysisManager::default();

        pass.run(&mut module, &mut context, &mut analyses).unwrap();

        let operation = module.operation(operation).unwrap();

        assert_eq!(operation.attribute("first"), Some(&Attribute::Bool(true)),);

        assert_eq!(operation.attribute("second"), Some(&Attribute::Bool(true)),);

        assert!(context.changed());

        module.verify().unwrap();
    }
}
