// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use ir::{
    GreedyRewriteConfig, GreedyRewriteReport, Module, OperationBuilder, OperationId,
    PatternBenefit, PatternResult, PatternRewriter, RewriteError, RewritePattern,
    RewritePatternSet, Type, ValueId, ValueProducer, apply_patterns_greedily,
};

use crate::{AnalysisManager, Pass, PassContext, PassError};

const CAST_OPERATION: &str = "builtin.unrealized_conversion_cast";

pub fn populate_reconcile_unrealized_cast_patterns(patterns: &mut RewritePatternSet) {
    patterns.add(
        CAST_OPERATION,
        PatternBenefit::new(100),
        CancelRoundTripCast,
    );

    patterns.add(CAST_OPERATION, PatternBenefit::new(90), RemoveIdentityCast);

    patterns.add(CAST_OPERATION, PatternBenefit::new(80), CollapseCastChain);

    patterns.add(CAST_OPERATION, PatternBenefit::new(10), RemoveDeadCast);
}

pub fn reconcile_unrealized_casts(
    module: &mut Module,
    config: &GreedyRewriteConfig,
) -> Result<GreedyRewriteReport, RewriteError> {
    let mut patterns = RewritePatternSet::new();

    populate_reconcile_unrealized_cast_patterns(&mut patterns);

    apply_patterns_greedily(module, &patterns, config)
}

pub struct ReconcileUnrealizedCastsPass {
    config: GreedyRewriteConfig,
}

impl ReconcileUnrealizedCastsPass {
    pub fn new() -> Self {
        Self {
            config: GreedyRewriteConfig::default(),
        }
    }

    pub fn with_config(mut self, config: GreedyRewriteConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for ReconcileUnrealizedCastsPass {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for ReconcileUnrealizedCastsPass {
    fn name(&self) -> &'static str {
        "conversion.reconcile-unrealized-casts"
    }

    fn run(
        &self,
        module: &mut Module,
        context: &mut PassContext,
        _analyses: &mut AnalysisManager,
    ) -> Result<(), PassError> {
        let report = reconcile_unrealized_casts(module, &self.config)
            .map_err(|error| PassError::failed(self.name(), error.to_string()))?;

        if report.rewrites > 0 {
            context.mark_changed();
        }

        Ok(())
    }
}

struct RemoveIdentityCast;

impl RewritePattern for RemoveIdentityCast {
    fn name(&self) -> &'static str {
        "RemoveIdentityCast"
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<PatternResult, RewriteError> {
        let (operands, results) = {
            let operation_ref = rewriter
                .operation(operation)
                .expect("matched cast must exist");

            (
                operation_ref.operands().to_vec(),
                operation_ref.results().to_vec(),
            )
        };

        if operands.len() != results.len() {
            return Ok(PatternResult::NoMatch);
        }

        for (operand, result) in operands.iter().zip(&results) {
            if value_type(rewriter, *operand)? != value_type(rewriter, *result)? {
                return Ok(PatternResult::NoMatch);
            }
        }

        rewriter.replace_operation(operation, &operands)?;

        Ok(PatternResult::Rewritten)
    }
}

struct CancelRoundTripCast;

impl RewritePattern for CancelRoundTripCast {
    fn name(&self) -> &'static str {
        "CancelRoundTripCast"
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<PatternResult, RewriteError> {
        let Some(inner) = unique_cast_producer(rewriter, operation)? else {
            return Ok(PatternResult::NoMatch);
        };

        let (inner_operands, outer_results) = {
            let inner_ref = rewriter.operation(inner).expect("inner cast must exist");

            let outer_ref = rewriter
                .operation(operation)
                .expect("outer cast must exist");

            (inner_ref.operands().to_vec(), outer_ref.results().to_vec())
        };

        if inner_operands.len() != outer_results.len() {
            return Ok(PatternResult::NoMatch);
        }

        for (input, result) in inner_operands.iter().zip(&outer_results) {
            if value_type(rewriter, *input)? != value_type(rewriter, *result)? {
                return Ok(PatternResult::NoMatch);
            }
        }

        rewriter.replace_operation(operation, &inner_operands)?;

        erase_if_dead(rewriter, inner)?;

        Ok(PatternResult::Rewritten)
    }
}

struct CollapseCastChain;

impl RewritePattern for CollapseCastChain {
    fn name(&self) -> &'static str {
        "CollapseCastChain"
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<PatternResult, RewriteError> {
        let Some(inner) = unique_cast_producer(rewriter, operation)? else {
            return Ok(PatternResult::NoMatch);
        };

        let (inner_operands, result_types) = {
            let inner_ref = rewriter.operation(inner).expect("inner cast must exist");

            let outer_ref = rewriter
                .operation(operation)
                .expect("outer cast must exist");

            let result_types = outer_ref
                .results()
                .iter()
                .map(|value| value_type(rewriter, *value))
                .collect::<Result<Vec<_>, _>>()?;

            (inner_ref.operands().to_vec(), result_types)
        };

        let direct = rewriter.create_operation(
            OperationBuilder::new(CAST_OPERATION).results(result_types),
            inner_operands,
        )?;

        let direct_results = rewriter
            .operation(direct)
            .expect("created direct cast must exist")
            .results()
            .to_vec();

        rewriter.replace_operation(operation, &direct_results)?;

        erase_if_dead(rewriter, inner)?;

        Ok(PatternResult::Rewritten)
    }
}

struct RemoveDeadCast;

impl RewritePattern for RemoveDeadCast {
    fn name(&self) -> &'static str {
        "RemoveDeadCast"
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<PatternResult, RewriteError> {
        let is_dead = rewriter
            .operation(operation)
            .expect("matched cast must exist")
            .results()
            .iter()
            .all(|result| {
                !rewriter
                    .value(*result)
                    .expect("cast result must exist")
                    .has_uses()
            });

        if !is_dead {
            return Ok(PatternResult::NoMatch);
        }

        rewriter.erase_operation(operation)?;

        Ok(PatternResult::Rewritten)
    }
}

fn unique_cast_producer(
    rewriter: &PatternRewriter<'_>,
    operation: OperationId,
) -> Result<Option<OperationId>, RewriteError> {
    let operands = rewriter
        .operation(operation)
        .expect("matched cast must exist")
        .operands()
        .to_vec();

    let Some(first_operand) = operands.first() else {
        return Ok(None);
    };

    let ValueProducer::OperationResult {
        operation: producer,
        ..
    } = rewriter
        .value(*first_operand)
        .expect("cast operand must exist")
        .producer()
    else {
        return Ok(None);
    };

    let producer_ref = rewriter
        .operation(producer)
        .expect("cast producer must exist");

    if producer_ref.name().as_str() != CAST_OPERATION
        || producer_ref.results() != operands.as_slice()
    {
        return Ok(None);
    }

    Ok(Some(producer))
}

fn erase_if_dead(
    rewriter: &mut PatternRewriter<'_>,
    operation: OperationId,
) -> Result<(), RewriteError> {
    let is_dead = rewriter
        .operation(operation)
        .map(|operation_ref| {
            operation_ref.results().iter().all(|result| {
                !rewriter
                    .value(*result)
                    .expect("operation result must exist")
                    .has_uses()
            })
        })
        .unwrap_or(false);

    if is_dead {
        rewriter.erase_operation(operation)?;
    }

    Ok(())
}

fn value_type(rewriter: &PatternRewriter<'_>, value: ValueId) -> Result<Type, RewriteError> {
    rewriter
        .value(value)
        .map(|value_ref| value_ref.ty().clone())
        .ok_or_else(|| RewriteError::message(format!("missing value {value:?}",)))
}

#[cfg(test)]
mod tests {
    use ir::{Module, OperationBuilder, Type};

    use crate::{AnalysisManager, Pass, PassContext, ReconcileUnrealizedCastsPass};

    use super::{CAST_OPERATION, reconcile_unrealized_casts};

    #[test]
    fn removes_identity_cast_1() {
        let mut module = Module::new();

        let producer = module
            .append_operation(
                OperationBuilder::new("test.producer").result(Type::f64()),
                [],
            )
            .unwrap();

        let input = module.operation(producer).unwrap().result(0).unwrap();

        let cast = module
            .append_operation(
                OperationBuilder::new(CAST_OPERATION).result(Type::f64()),
                [input],
            )
            .unwrap();

        let cast_result = module.operation(cast).unwrap().result(0).unwrap();

        let consumer = module
            .append_operation(OperationBuilder::new("test.consumer"), [cast_result])
            .unwrap();

        reconcile_unrealized_casts(&mut module, &Default::default()).unwrap();

        assert!(module.operation(cast).is_none());
        assert_eq!(module.operation(consumer).unwrap().operand(0), Some(input),);

        module.verify().unwrap();
    }

    #[test]
    fn removes_identity_cast_2() {
        let mut module = Module::new();

        let producer = module
            .append_operation(
                OperationBuilder::new("test.producer").result(Type::f64()),
                [],
            )
            .unwrap();

        let input = module.operation(producer).unwrap().result(0).unwrap();

        let cast = module
            .append_operation(
                OperationBuilder::new(CAST_OPERATION).result(Type::f64()),
                [input],
            )
            .unwrap();

        let cast_result = module.operation(cast).unwrap().result(0).unwrap();

        let consumer = module
            .append_operation(OperationBuilder::new("test.consumer"), [cast_result])
            .unwrap();

        let mut context = PassContext::default();

        let mut analyses = AnalysisManager::default();

        ReconcileUnrealizedCastsPass::new()
            .run(&mut module, &mut context, &mut analyses)
            .unwrap();

        assert!(context.changed());

        assert!(module.operation(cast).is_none());

        assert_eq!(module.operation(consumer).unwrap().operand(0), Some(input),);

        module.verify().unwrap();
    }

    #[test]
    fn cancels_round_trip_casts() {
        let source_type = Type::f64();
        let intermediate_type = Type::integer(64, ir::Signedness::Signless);

        let mut module = Module::new();

        let producer = module
            .append_operation(
                OperationBuilder::new("test.producer").result(source_type.clone()),
                [],
            )
            .unwrap();

        let source = module.operation(producer).unwrap().result(0).unwrap();

        let inner = module
            .append_operation(
                OperationBuilder::new(CAST_OPERATION).result(intermediate_type),
                [source],
            )
            .unwrap();

        let intermediate = module.operation(inner).unwrap().result(0).unwrap();

        let outer = module
            .append_operation(
                OperationBuilder::new(CAST_OPERATION).result(source_type),
                [intermediate],
            )
            .unwrap();

        let output = module.operation(outer).unwrap().result(0).unwrap();

        let consumer = module
            .append_operation(OperationBuilder::new("test.consumer"), [output])
            .unwrap();

        reconcile_unrealized_casts(&mut module, &Default::default()).unwrap();

        assert!(module.operation(inner).is_none());
        assert!(module.operation(outer).is_none());
        assert_eq!(module.operation(consumer).unwrap().operand(0), Some(source),);

        module.verify().unwrap();
    }
}
