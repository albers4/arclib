// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use compiler::{AnalysisManager, Pass, PassContext, PassError, reconcile_unrealized_casts};
use equation::{EVOLUTION_OPERATION, UNKNOWN_OPERATION};
use ir::{
    Attribute, GreedyRewriteConfig, Module, OperationBuilder, OperationId, PatternBenefit,
    PatternResult, PatternRewriter, RewriteError, RewritePattern, RewritePatternSet, Type,
    UNREALIZED_CAST, ValueId, ValueProducer, apply_patterns_greedily,
};
use mesh::{FIELD_OPERATION as MESH_FIELD_OPERATION, MeshFieldOp};
use operator::{
    FACTOR_ATTRIBUTE, FIELD_OPERATION as OPERATOR_FIELD_OPERATION,
    LAPLACIAN_OPERATION as OPERATOR_LAPLACIAN_OPERATION,
    SCALE_OPERATION as OPERATOR_SCALE_OPERATION,
};

use crate::{
    ConstantOp, EULER_CELL_COUNT_OPERAND, EULER_DESTINATION_OPERAND, EXPLICIT_EULER_OPERATION,
    ExplicitEulerOp, LAPLACIAN_CELL_COUNT_OPERAND, LAPLACIAN_OPERATION, LaplacianOp,
    PREPARE_CELL_COUNT_RESULT, PREPARE_CELL_VOLUME_RESULT, PREPARE_FACE_COEFFICIENT_RESULT,
    PREPARE_FACE_COUNT_RESULT, PREPARE_NEIGHBOUR_RESULT, PREPARE_OWNER_RESULT, PrepareMeshOp,
    SCALE_CELL_COUNT_OPERAND, SCALE_DESTINATION_OPERAND, SCALE_OPERATION, ScaleOp,
};

pub const LOWER_METHOD_TO_FVM_PASS: &str = "fvm.lower-method-to-fvm";
pub const LOWER_METHOD_TO_FVM_PIPELINE: &str = "fvm.method-to-fvm";

#[derive(Debug, Clone)]
pub struct LowerMethodToFvmPass {
    config: GreedyRewriteConfig,
}

impl LowerMethodToFvmPass {
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

impl Default for LowerMethodToFvmPass {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for LowerMethodToFvmPass {
    fn name(&self) -> &'static str {
        LOWER_METHOD_TO_FVM_PASS
    }

    fn run(
        &self,
        module: &mut Module,
        context: &mut PassContext,
        _analyses: &mut AnalysisManager,
    ) -> Result<(), PassError> {
        let mut patterns = RewritePatternSet::new();
        patterns.add(
            UNKNOWN_OPERATION,
            PatternBenefit::new(10),
            EraseDeadAliasPattern::new("EraseDeadEquationUnknown"),
        );
        patterns.add(
            OPERATOR_FIELD_OPERATION,
            PatternBenefit::new(10),
            EraseDeadAliasPattern::new("EraseDeadOperatorField"),
        );
        patterns.add(
            OPERATOR_LAPLACIAN_OPERATION,
            PatternBenefit::new(80),
            LowerLaplacian,
        );
        patterns.add(
            OPERATOR_SCALE_OPERATION,
            PatternBenefit::new(70),
            LowerScale,
        );
        patterns.add(EVOLUTION_OPERATION, PatternBenefit::new(60), LowerEvolution);

        let report = apply_patterns_greedily(module, &patterns, &self.config)
            .map_err(|error| PassError::failed(self.name(), error.to_string()))?;

        let reconcile_report = reconcile_unrealized_casts(module, &self.config)
            .map_err(|error| PassError::failed(self.name(), error.to_string()))?;
        if report.rewrites > 0 || reconcile_report.rewrites > 0 {
            context.mark_changed();
        }
        Ok(())
    }
}

struct EraseDeadAliasPattern {
    name: &'static str,
}

impl EraseDeadAliasPattern {
    const fn new(name: &'static str) -> Self {
        Self { name }
    }
}

impl RewritePattern for EraseDeadAliasPattern {
    fn name(&self) -> &'static str {
        self.name
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<PatternResult, RewriteError> {
        let is_dead = rewriter
            .operation(operation)
            .expect("matched alias must exist")
            .results()
            .iter()
            .all(|result| {
                !rewriter
                    .value(*result)
                    .expect("alias result must exist")
                    .has_uses()
            });

        if !is_dead {
            return Ok(PatternResult::NoMatch);
        }

        rewriter.erase_operation(operation)?;

        Ok(PatternResult::Rewritten)
    }
}

fn bridge_value(
    rewriter: &mut PatternRewriter<'_>,
    value: ValueId,
    target_type: Type,
) -> Result<ValueId, RewriteError> {
    let source_type = rewriter
        .value(value)
        .ok_or_else(|| {
            RewriteError::message(
                "missing value while creating \
                 conversion bridge",
            )
        })?
        .ty()
        .clone();

    if source_type == target_type {
        return Ok(value);
    }

    let cast = rewriter.create_operation(
        OperationBuilder::new(UNREALIZED_CAST).result(target_type),
        [value],
    )?;

    rewriter
        .operation(cast)
        .and_then(|operation| operation.result(0))
        .ok_or_else(|| RewriteError::message("unrealized cast has no result"))
}

struct LowerLaplacian;

impl RewritePattern for LowerLaplacian {
    fn name(&self) -> &'static str {
        "LowerOperatorLaplacianToFvm"
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<PatternResult, RewriteError> {
        let (source, symbolic_result_type) = {
            let source_operation = rewriter.operation(operation).expect(
                "operator.laplacian root \
                        must exist",
            );

            let source = source_operation
                .operands()
                .first()
                .copied()
                .ok_or_else(|| {
                    RewriteError::message(
                        "operator.laplacian has \
                        no field operand",
                    )
                })?;

            let result_type = source_operation
                .result_type(0)
                .ok_or_else(|| {
                    RewriteError::message(
                        "operator.laplacian has \
                            no result type",
                    )
                })?
                .clone();

            (source, result_type)
        };

        let source = peel_aliases(rewriter, source);
        let Some(mesh) = mesh_for_field(rewriter, source) else {
            return Ok(PatternResult::NoMatch);
        };
        let field_type = rewriter
            .value(source)
            .ok_or_else(|| RewriteError::message("missing Laplacian field value"))?
            .ty()
            .clone();

        let prepare = rewriter.create_operation(PrepareMeshOp::builder(1), [mesh])?;
        let prepared = rewriter
            .operation(prepare)
            .expect("new fvm.prepare_mesh must exist")
            .results()
            .to_vec();

        let destination = rewriter.create_operation(
            MeshFieldOp::builder_untyped("fvm.laplacian", field_type.clone()),
            [mesh],
        )?;
        let destination = rewriter
            .operation(destination)
            .expect("new mesh.field must exist")
            .result(0)
            .expect("mesh.field must have one result");

        let discrete = rewriter.create_operation(
            LaplacianOp::builder(field_type),
            [
                source,
                prepared[PREPARE_OWNER_RESULT],
                prepared[PREPARE_NEIGHBOUR_RESULT],
                prepared[PREPARE_FACE_COEFFICIENT_RESULT],
                prepared[PREPARE_CELL_VOLUME_RESULT],
                destination,
                prepared[PREPARE_CELL_COUNT_RESULT],
                prepared[PREPARE_FACE_COUNT_RESULT],
            ],
        )?;
        let result = rewriter
            .operation(discrete)
            .expect("new fvm.laplacian must exist")
            .result(0)
            .expect("fvm.laplacian must have one result");
        let replacement = bridge_value(rewriter, result, symbolic_result_type)?;
        rewriter.replace_operation(operation, &[replacement])?;
        Ok(PatternResult::Rewritten)
    }
}

struct LowerScale;

impl RewritePattern for LowerScale {
    fn name(&self) -> &'static str {
        "LowerOperatorScaleToFvm"
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<PatternResult, RewriteError> {
        let (source, factor, symbolic_result_type) = {
            let source_operation = rewriter
                .operation(operation)
                .expect("operator.scale root must exist");

            let Some(source) = source_operation.operands().first().copied() else {
                return Ok(PatternResult::NoMatch);
            };

            let Some(Attribute::Float(factor)) = source_operation.attribute(FACTOR_ATTRIBUTE)
            else {
                return Ok(PatternResult::NoMatch);
            };

            let result_type = source_operation
                .result_type(0)
                .ok_or_else(|| {
                    RewriteError::message(
                        "operator.scale has \
                            no result type",
                    )
                })?
                .clone();

            (source, *factor, result_type)
        };

        let source = peel_aliases(rewriter, source);
        let Some(cell_count) = cell_count_for_value(rewriter, source) else {
            return Ok(PatternResult::NoMatch);
        };
        let result_type = rewriter
            .value(source)
            .ok_or_else(|| RewriteError::message("missing scale source value"))?
            .ty()
            .clone();
        let Some(mesh) = mesh_for_discrete_value(rewriter, source) else {
            return Ok(PatternResult::NoMatch);
        };
        let destination = rewriter.create_operation(
            MeshFieldOp::builder_untyped("fvm.scale", result_type.clone()),
            [mesh],
        )?;
        let destination = rewriter
            .operation(destination)
            .expect("new scale destination must exist")
            .result(0)
            .expect("scale destination must have one result");
        let constant = rewriter.create_operation(ConstantOp::builder(factor), [])?;
        let factor_value = rewriter
            .operation(constant)
            .expect("new fvm.constant must exist")
            .result(0)
            .expect("fvm.constant must have one result");
        let scale = rewriter.create_operation(
            ScaleOp::builder(result_type),
            [source, factor_value, destination, cell_count],
        )?;
        let result = rewriter
            .operation(scale)
            .expect("new fvm.scale must exist")
            .result(0)
            .expect("fvm.scale must have one result");

        let replacement = bridge_value(rewriter, result, symbolic_result_type)?;
        rewriter.replace_operation(operation, &[replacement])?;
        Ok(PatternResult::Rewritten)
    }
}

struct LowerEvolution;

impl RewritePattern for LowerEvolution {
    fn name(&self) -> &'static str {
        "LowerEquationEvolutionToFvm"
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<PatternResult, RewriteError> {
        let operands = rewriter
            .operation(operation)
            .expect("equation.evolution rewrite root must exist")
            .operands()
            .to_vec();
        if operands.len() != 3 {
            return Ok(PatternResult::NoMatch);
        }
        let state = peel_aliases(rewriter, operands[0]);
        let rhs = peel_aliases(rewriter, operands[1]);
        let time_step = operands[2];
        let Some(cell_count) = cell_count_for_value(rewriter, rhs) else {
            return Ok(PatternResult::NoMatch);
        };
        let state_type = rewriter
            .value(state)
            .ok_or_else(|| RewriteError::message("missing evolution state value"))?
            .ty()
            .clone();
        let Some(mesh) = mesh_for_field(rewriter, state) else {
            return Ok(PatternResult::NoMatch);
        };
        let destination = rewriter.create_operation(
            MeshFieldOp::builder_untyped("fvm.next_state", state_type.clone()),
            [mesh],
        )?;
        let destination = rewriter
            .operation(destination)
            .expect("new evolution destination must exist")
            .result(0)
            .expect("evolution destination must have one result");
        let euler = rewriter.create_operation(
            ExplicitEulerOp::builder(state_type),
            [state, rhs, time_step, destination, cell_count],
        )?;
        let result = rewriter
            .operation(euler)
            .expect("new fvm.explicit_euler must exist")
            .result(0)
            .expect("fvm.explicit_euler must have one result");
        rewriter.replace_operation(operation, &[result])?;
        Ok(PatternResult::Rewritten)
    }
}

fn peel_aliases(rewriter: &PatternRewriter<'_>, mut value: ValueId) -> ValueId {
    for _ in 0..8 {
        let Some(producer) = producer(rewriter, value) else {
            break;
        };
        let Some(operation) = rewriter.operation(producer) else {
            break;
        };
        let is_alias = matches!(
            operation.name().as_str(),
            OPERATOR_FIELD_OPERATION | UNKNOWN_OPERATION
        );

        let is_conversion_bridge = operation.name().as_str() == UNREALIZED_CAST
            && operation.operands().len() == 1
            && operation.results().len() == 1;

        if is_alias || is_conversion_bridge {
            let Some(next) = operation.operands().first().copied() else {
                break;
            };
            value = next;
        } else {
            break;
        }
    }
    value
}

fn mesh_for_field(rewriter: &PatternRewriter<'_>, field: ValueId) -> Option<ValueId> {
    let producer = producer(rewriter, field)?;
    let operation = rewriter.operation(producer)?;
    if operation.name().as_str() == MESH_FIELD_OPERATION {
        operation.operands().first().copied()
    } else {
        None
    }
}

fn mesh_for_discrete_value(rewriter: &PatternRewriter<'_>, value: ValueId) -> Option<ValueId> {
    let producer = producer(rewriter, value)?;
    let operation = rewriter.operation(producer)?;
    let destination = match operation.name().as_str() {
        LAPLACIAN_OPERATION => operation
            .operands()
            .get(crate::LAPLACIAN_DESTINATION_OPERAND)
            .copied(),
        SCALE_OPERATION => operation.operands().get(SCALE_DESTINATION_OPERAND).copied(),
        EXPLICIT_EULER_OPERATION => operation.operands().get(EULER_DESTINATION_OPERAND).copied(),
        _ => None,
    }?;
    mesh_for_field(rewriter, destination)
}

fn cell_count_for_value(rewriter: &PatternRewriter<'_>, value: ValueId) -> Option<ValueId> {
    let producer = producer(rewriter, value)?;
    let operation = rewriter.operation(producer)?;
    match operation.name().as_str() {
        LAPLACIAN_OPERATION => operation
            .operands()
            .get(LAPLACIAN_CELL_COUNT_OPERAND)
            .copied(),
        SCALE_OPERATION => operation.operands().get(SCALE_CELL_COUNT_OPERAND).copied(),
        EXPLICIT_EULER_OPERATION => operation.operands().get(EULER_CELL_COUNT_OPERAND).copied(),
        _ => None,
    }
}

fn producer(rewriter: &PatternRewriter<'_>, value: ValueId) -> Option<OperationId> {
    match rewriter.value(value)?.producer() {
        ValueProducer::OperationResult { operation, .. } => Some(operation),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use compiler::{AnalysisManager, Pass, PassContext};
    use equation::{EvolutionOp, TimeIntegrator};
    use ir::{Module, OperationBuilder, Type};
    use mesh::{MeshFieldOp, MeshFieldType, MeshTopology, MeshType};
    use operator::{
        ExpressionType, FieldOp, LaplacianOp as SymbolicLaplacianOp, ScaleOp as SymbolicScaleOp,
    };

    use crate::{
        EXPLICIT_EULER_OPERATION, LAPLACIAN_OPERATION, LowerMethodToFvmPass, SCALE_OPERATION,
    };

    #[test]
    fn lowers_symbolic_diffusion_chain_to_fvm() {
        let mut module = Module::new();
        let mesh = module
            .append_operation(
                OperationBuilder::new("test.mesh")
                    .result(MeshType::new(1, 3, MeshTopology::Structured).ir_type()),
                [],
            )
            .unwrap();
        let mesh = module.operation(mesh).unwrap().result(0).unwrap();
        let field_type = MeshFieldType::cell_scalar_f64(1);
        let field = module
            .append_operation(MeshFieldOp::builder("temperature", &field_type), [mesh])
            .unwrap();
        let field = module.operation(field).unwrap().result(0).unwrap();
        let expression = ExpressionType::scalar_f64();
        let symbolic = module
            .append_operation(FieldOp::builder(&expression), [field])
            .unwrap();
        let symbolic = module.operation(symbolic).unwrap().result(0).unwrap();
        let laplacian = module
            .append_operation(SymbolicLaplacianOp::builder(&expression), [symbolic])
            .unwrap();
        let laplacian = module.operation(laplacian).unwrap().result(0).unwrap();
        let rhs = module
            .append_operation(SymbolicScaleOp::builder(&expression, 0.01), [laplacian])
            .unwrap();
        let rhs = module.operation(rhs).unwrap().result(0).unwrap();
        let dt = module
            .append_operation(OperationBuilder::new("test.dt").result(Type::f64()), [])
            .unwrap();
        let dt = module.operation(dt).unwrap().result(0).unwrap();
        module
            .append_operation(
                EvolutionOp::builder(
                    "temperature",
                    field_type.ir_type(),
                    TimeIntegrator::ExplicitEuler,
                ),
                [field, rhs, dt],
            )
            .unwrap();

        let mut context = PassContext::default();
        let mut analyses = AnalysisManager::default();
        LowerMethodToFvmPass::new()
            .run(&mut module, &mut context, &mut analyses)
            .unwrap();

        let names = module
            .operations()
            .into_iter()
            .filter_map(|id| module.operation(id))
            .map(|operation| operation.name().as_str().to_owned())
            .collect::<Vec<_>>();
        assert!(names.iter().any(|name| name == LAPLACIAN_OPERATION));
        assert!(names.iter().any(|name| name == SCALE_OPERATION));
        assert!(names.iter().any(|name| name == EXPLICIT_EULER_OPERATION));
    }
}
