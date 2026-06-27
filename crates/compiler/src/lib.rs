// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod analysis;
mod conversion;
mod diagnostic;
mod extension;
mod pass;
mod passes;
mod planning;
mod policy;
mod registry;
mod request;
mod session;
mod target;

pub use pass::{
    NoopInstrumentation, Pass, PassContext, PassError, PassFailure, PassInstrumentation,
    PassManager, PassReport,
};

pub use analysis::{
    Analysis, AnalysisManager, ControlFlowGraphAnalysis, DominanceAnalysis, OperationListAnalysis,
    OperationNameAnalysis, RegionControlFlowGraph, SymbolDefinitionAnalysis,
};

pub use extension::CompilerExtension;

pub use registry::{
    CompilerRegistry, PassRegistry, PipelineDescriptor, PipelineRegistry, RegistryError,
};

pub use session::{CompilerSession, CompilerSessionBuilder, SessionError, SessionOptions};

pub use policy::{
    CompilationConstraint, ConstraintPredicate, ConstraintScope, ConstraintSet, ConstraintStrength,
    DecisionLog, DecisionSubject, Metric, ObjectiveDirection, OptimizationObjective,
    OptimizationObjectives, PolicyDecision, PolicyDecisionKind, PolicyError, PolicyValue, Property,
};

pub use request::{CompileRequest, CompileRequestBuilder, RequestError};

pub use diagnostic::{
    Diagnostic, DiagnosticAnchor, DiagnosticEngine, DiagnosticNote, DiagnosticSeverity,
};

pub use conversion::{
    ConversionConfig, ConversionError, ConversionMode, ConversionPatternSet, ConversionReport,
    ConversionTarget, DialectConversionPass, Legality, ReconcileUnrealizedCastsPass, TypeConverter,
    apply_conversion, apply_full_conversion, apply_partial_conversion, reconcile_unrealized_casts,
};

pub use planning::{
    ConversionEdgeDescriptor, ConversionPlan, ConversionRegistry, ConversionStage, ConversionStep,
    EdgeApplicability, PlannedCompilationReport, PlanningConfig, PlanningError, plan_conversion,
};

pub use target::{CapabilityRequirement, TargetProfile, predicate_matches};

#[cfg(test)]
mod tests {
    use ir::{
        Attribute, BlockBuilder, DialectDescriptor, Module, OperationBuilder, OperationDescriptor,
        PatternBenefit, Type,
    };

    use crate::{
        CompileRequest, CompilerExtension, CompilerRegistry, CompilerSession, ConstraintScope,
        ConstraintStrength, ConversionError, ConversionPatternSet, ConversionTarget,
        DecisionSubject, Diagnostic, DiagnosticSeverity, Legality, Pass, PassContext, PassError,
        PassManager, PipelineDescriptor, PolicyDecision, PolicyDecisionKind, PolicyError,
        RegistryError, RequestError,
        analysis::AnalysisManager,
        apply_full_conversion, apply_partial_conversion,
        conversion::{
            ConversionValueMapping, TypeConversionRuleResult, TypeConverter,
            convert_region_entry_signature,
        },
    };

    use ir::{OperationId, PatternResult, PatternRewriter, RewriteError, RewritePattern};

    struct TestExtension;

    impl CompilerExtension for TestExtension {
        fn name(&self) -> &'static str {
            "test"
        }

        fn register(&self, registry: &mut CompilerRegistry) -> Result<(), RegistryError> {
            let mut dialect = DialectDescriptor::new("test");

            dialect.register_operation(OperationDescriptor::new("test.input"))?;

            dialect.register_operation(OperationDescriptor::new("test.done"))?;

            registry.dialects_mut().register_dialect(dialect)?;

            registry
                .passes_mut()
                .register("test.add-done", || AddDonePass)?;

            registry
                .pipelines_mut()
                .register(PipelineDescriptor::new("test.default").pass("test.add-done"))?;

            Ok(())
        }
    }

    struct AddDonePass;

    impl Pass for AddDonePass {
        fn name(&self) -> &'static str {
            "test.add-done"
        }

        fn run(
            &self,
            module: &mut Module,
            context: &mut PassContext,
            _analyses: &mut AnalysisManager,
        ) -> Result<(), PassError> {
            module
                .append_operation(OperationBuilder::new("test.done"), [])
                .map_err(|error| PassError::failed(self.name(), error.to_string()))?;

            context.mark_changed();

            Ok(())
        }
    }

    #[test]
    fn extension_registers_dialect_pass_and_pipeline() {
        let session = CompilerSession::builder()
            .register_extension(TestExtension)
            .unwrap()
            .build();

        assert!(session.registry().has_extension("test"));

        assert!(session.registry().passes().contains("test.add-done"));

        assert!(session.registry().pipelines().get("test.default").is_some());
    }

    #[test]
    fn runs_registered_pipeline() {
        let session = CompilerSession::builder()
            .register_extension(TestExtension)
            .unwrap()
            .build();

        let mut module = Module::new();

        module
            .append_operation(OperationBuilder::new("test.input"), [])
            .unwrap();

        let report = session.run_pipeline("test.default", &mut module).unwrap();

        assert!(report.changed);
        assert_eq!(report.passes_run, 1);

        let names: Vec<_> = module
            .operations()
            .into_iter()
            .filter_map(|operation| {
                module
                    .operation(operation)
                    .map(|operation| operation.name().as_str().to_owned())
            })
            .collect();

        assert!(names.iter().any(|name| { name == "test.done" }));
    }

    #[test]
    fn rejects_duplicate_extension() {
        let mut registry = CompilerRegistry::new();

        registry.register_extension(TestExtension).unwrap();

        let error = registry.register_extension(TestExtension).unwrap_err();

        assert!(matches!(error, RegistryError::DuplicateExtension(_)));
    }

    #[test]
    fn rejects_pipeline_with_missing_pass_at_build_time() {
        let mut registry = CompilerRegistry::new();

        registry
            .pipelines_mut()
            .register(PipelineDescriptor::new("broken").pass("missing.pass"))
            .unwrap();

        let error = match registry.pipelines().build("broken", registry.passes()) {
            Ok(_) => {
                panic!("expected pipeline construction to fail")
            }

            Err(error) => error,
        };

        assert!(matches!(error, RegistryError::MissingPass(_)));
    }

    #[test]
    fn builds_compile_request() {
        let request = CompileRequest::builder("simulation.default")
            .require("numeric.precision", "f64")
            .prefer("target.device", "gpu", 10.0)
            .hint("lbm.tile-size", 16_i64)
            .minimize("latency", 1.0)
            .build()
            .unwrap();

        assert_eq!(request.pipeline(), "simulation.default",);

        assert_eq!(request.constraints().len(), 3,);
    }

    #[test]
    fn rejects_invalid_preference_weight() {
        let error = CompileRequest::builder("simulation.default")
            .prefer("target.device", "gpu", 0.0)
            .build()
            .unwrap_err();

        assert!(matches!(
            error,
            RequestError::Policy(PolicyError::InvalidWeight { .. })
        ));
    }

    #[test]
    fn rejects_conflicting_required_values() {
        let error = CompileRequest::builder("simulation.default")
            .require("numeric.precision", "f32")
            .require("numeric.precision", "f64")
            .build()
            .unwrap_err();

        assert!(matches!(
            error,
            RequestError::Policy(PolicyError::ConflictingRequiredConstraints { .. })
        ));
    }

    struct RequestReadingPass;

    impl Pass for RequestReadingPass {
        fn name(&self) -> &'static str {
            "test.read-request"
        }

        fn run(
            &self,
            _module: &mut Module,
            context: &mut PassContext,
            _analyses: &mut AnalysisManager,
        ) -> Result<(), PassError> {
            let required_precision = context.request().constraints().iter().any(|constraint| {
                constraint.property().as_str() == "numeric.precision"
                    && matches!(constraint.strength(), ConstraintStrength::Required)
            });

            if !required_precision {
                return Err(PassError::failed(
                    self.name(),
                    "missing required precision constraint",
                ));
            }

            Ok(())
        }
    }

    struct RequestExtension;

    impl CompilerExtension for RequestExtension {
        fn name(&self) -> &'static str {
            "request-test"
        }

        fn register(&self, registry: &mut CompilerRegistry) -> Result<(), RegistryError> {
            registry
                .passes_mut()
                .register("test.read-request", || RequestReadingPass)?;

            registry
                .pipelines_mut()
                .register(PipelineDescriptor::new("test.request").pass("test.read-request"))?;

            Ok(())
        }
    }

    #[test]
    fn passes_can_read_compile_request() {
        let session = CompilerSession::builder()
            .register_extension(RequestExtension)
            .unwrap()
            .build();

        let request = CompileRequest::builder("test.request")
            .require("numeric.precision", "f64")
            .build()
            .unwrap();

        let mut module = Module::new();

        session.compile(&request, &mut module).unwrap();
    }

    struct ReportingPass;

    impl Pass for ReportingPass {
        fn name(&self) -> &'static str {
            "test.reporting"
        }

        fn run(
            &self,
            _module: &mut Module,
            context: &mut PassContext,
            _analyses: &mut AnalysisManager,
        ) -> Result<(), PassError> {
            context.emit(
                Diagnostic::warning("preferred GPU target was unavailable")
                    .with_code("target.preference-rejected"),
            );

            context.record_decision(
                PolicyDecision::new(
                    PolicyDecisionKind::PreferenceRejected,
                    ConstraintScope::Global,
                    DecisionSubject::named("target.device"),
                    "no registered GPU target matched \
                     the required capabilities",
                )
                .requested("gpu")
                .selected("cpu"),
            );

            Ok(())
        }
    }

    #[test]
    fn reports_diagnostics_and_decisions() {
        let mut manager = PassManager::new();

        manager.add_pass(ReportingPass);

        let mut module = Module::new();

        let registry = ir::DialectRegistry::with_builtin();

        let report = manager
            .run_with_request(&mut module, &registry, &CompileRequest::default())
            .unwrap();

        assert_eq!(report.diagnostics.len(), 1,);

        assert_eq!(
            report.diagnostics[0].severity(),
            DiagnosticSeverity::Warning,
        );

        assert_eq!(report.diagnostics[0].pass(), Some("test.reporting"),);

        assert_eq!(report.decisions.len(), 1,);

        assert_eq!(report.decisions[0].pass(), Some("test.reporting"),);
    }

    struct FluidToOperatorPattern;

    impl RewritePattern for FluidToOperatorPattern {
        fn name(&self) -> &'static str {
            "FluidToOperatorPattern"
        }

        fn match_and_rewrite(
            &self,
            operation: OperationId,
            rewriter: &mut PatternRewriter<'_>,
        ) -> Result<PatternResult, RewriteError> {
            let name = rewriter
                .operation(operation)
                .unwrap()
                .name()
                .as_str()
                .to_owned();

            if name != "fluid.system" {
                return Ok(PatternResult::NoMatch);
            }

            rewriter.create_operation(OperationBuilder::new("operator.system"), [])?;

            rewriter.replace_operation(operation, &[])?;

            Ok(PatternResult::Rewritten)
        }
    }

    #[test]
    fn partial_conversion_allows_unknown_operations() {
        let mut module = Module::new();

        module
            .append_operation(OperationBuilder::new("fluid.system"), [])
            .unwrap();

        module
            .append_operation(OperationBuilder::new("custom.unclassified"), [])
            .unwrap();

        let mut target = ConversionTarget::new();

        target
            .mark_dialect_legal("builtin")
            .mark_dialect_illegal("fluid")
            .mark_dialect_legal("operator");

        let mut patterns = ConversionPatternSet::new();

        patterns.add(
            "fluid.system",
            PatternBenefit::DEFAULT,
            FluidToOperatorPattern,
        );

        let report = apply_partial_conversion(&mut module, &target, &patterns).unwrap();

        assert_eq!(report.rewrites, 1,);

        assert_eq!(report.remaining_unknown_operations, 1,);

        let names: Vec<_> = module
            .operations()
            .into_iter()
            .filter_map(|operation| {
                module
                    .operation(operation)
                    .map(|operation| operation.name().as_str().to_owned())
            })
            .collect();

        assert!(names.iter().any(|name| { name == "operator.system" }));

        assert!(!names.iter().any(|name| { name == "fluid.system" }));

        module.verify().unwrap();
    }

    #[test]
    fn full_conversion_rejects_unknown_operations() {
        let mut module = Module::new();

        module
            .append_operation(OperationBuilder::new("custom.unknown"), [])
            .unwrap();

        let mut target = ConversionTarget::new();

        target.mark_dialect_legal("builtin");

        let patterns = ConversionPatternSet::new();

        let error = apply_full_conversion(&mut module, &target, &patterns).unwrap_err();

        assert!(matches!(
            error,
            ConversionError::UnlegalizableOperation {
                legality: Legality::Unknown,
                ..
            }
        ));
    }

    #[test]
    fn partial_conversion_rejects_unconverted_illegal_operation() {
        let mut module = Module::new();

        module
            .append_operation(OperationBuilder::new("fluid.system"), [])
            .unwrap();

        let mut target = ConversionTarget::new();

        target
            .mark_dialect_legal("builtin")
            .mark_dialect_illegal("fluid");

        let patterns = ConversionPatternSet::new();

        let error = apply_partial_conversion(&mut module, &target, &patterns).unwrap_err();

        assert!(matches!(
            error,
            ConversionError::UnlegalizableOperation {
                legality: Legality::Illegal,
                ..
            }
        ));
    }

    struct FinalizeOperationPattern;

    impl RewritePattern for FinalizeOperationPattern {
        fn name(&self) -> &'static str {
            "FinalizeOperationPattern"
        }

        fn match_and_rewrite(
            &self,
            operation: OperationId,
            rewriter: &mut PatternRewriter<'_>,
        ) -> Result<PatternResult, RewriteError> {
            let finalized = matches!(
                rewriter
                    .operation(operation)
                    .unwrap()
                    .attribute("finalized"),
                Some(Attribute::Bool(true))
            );

            if finalized {
                return Ok(PatternResult::NoMatch);
            }

            rewriter.set_attribute(operation, "finalized", Attribute::Bool(true))?;

            Ok(PatternResult::Rewritten)
        }
    }

    #[test]
    fn dynamic_legality_becomes_legal_after_rewrite() {
        let mut module = Module::new();

        module
            .append_operation(OperationBuilder::new("test.operation"), [])
            .unwrap();

        let mut target = ConversionTarget::new();

        target
            .mark_dialect_legal("builtin")
            .mark_operation_dynamically_legal("test.operation", |operation| {
                Ok(matches!(
                    operation.attribute("finalized",),
                    Some(Attribute::Bool(true))
                ))
            });

        let mut patterns = ConversionPatternSet::new();

        patterns.add(
            "test.operation",
            PatternBenefit::DEFAULT,
            FinalizeOperationPattern,
        );

        let report = apply_full_conversion(&mut module, &target, &patterns).unwrap();

        assert_eq!(report.rewrites, 1,);

        module.verify().unwrap();
    }

    #[test]
    fn converts_one_type_to_multiple_types() {
        let field_type = Type::dialect("field.value", []);

        let data_type = Type::dialect("buffer.data", []);

        let metadata_type = Type::dialect("buffer.metadata", []);

        let mut converter = TypeConverter::new();

        let field_type_clone = field_type.clone();

        let data_type_clone = data_type.clone();

        let metadata_type_clone = metadata_type.clone();

        converter
            .enable_identity_fallback()
            .add_conversion(move |source| {
                if source != &field_type_clone {
                    return TypeConversionRuleResult::NotApplicable;
                }

                TypeConversionRuleResult::Converted(vec![
                    data_type_clone.clone(),
                    metadata_type_clone.clone(),
                ])
            })
            .add_unrealized_cast_materializations();

        assert_eq!(
            converter.convert_type(&field_type,).unwrap(),
            vec![data_type, metadata_type,],
        );

        assert!(!converter.is_legal(&field_type,).unwrap());

        assert!(converter.is_legal(&Type::f64(),).unwrap());
    }

    #[test]
    fn moves_region_contents_between_operations() {
        let mut module = Module::new();

        let source = module
            .append_operation(OperationBuilder::new("test.source").region(), [])
            .unwrap();

        let target = module
            .append_operation(OperationBuilder::new("test.target").region(), [])
            .unwrap();

        let source_region = module.operation(source).unwrap().regions()[0];

        let target_region = module.operation(target).unwrap().regions()[0];

        let block = module
            .append_block(source_region, BlockBuilder::new().argument(Type::f64()))
            .unwrap();

        module
            .append_operation_to_block(block, OperationBuilder::new("test.body"), [])
            .unwrap();

        module
            .move_region_contents(source_region, target_region)
            .unwrap();

        assert!(module.region(source_region).unwrap().is_empty());

        assert_eq!(module.region(target_region).unwrap().blocks(), &[block],);

        assert_eq!(module.block(block).unwrap().parent_region(), target_region,);

        module.verify().unwrap();
    }

    #[test]
    fn expands_region_entry_argument() {
        let field_type = Type::dialect("field.value", vec![]);

        let data_type = Type::dialect("buffer.data", vec![]);

        let metadata_type = Type::dialect("buffer.metadata", vec![]);

        let mut module = Module::new();

        let owner = module
            .append_operation(OperationBuilder::new("test.owner").region(), [])
            .unwrap();

        let region = module.operation(owner).unwrap().regions()[0];

        let block = module
            .append_block(
                region,
                BlockBuilder::new()
                    .argument(field_type.clone())
                    .argument(Type::f64()),
            )
            .unwrap();

        let old_field = module.block(block).unwrap().argument(0).unwrap();

        let consumer = module
            .append_operation_to_block(block, OperationBuilder::new("test.consume"), [old_field])
            .unwrap();

        let mut converter = TypeConverter::new();

        let source = field_type.clone();

        let data = data_type.clone();

        let metadata = metadata_type.clone();

        converter
            .enable_identity_fallback()
            .add_conversion(move |ty| {
                if ty != &source {
                    return TypeConversionRuleResult::NotApplicable;
                }

                TypeConversionRuleResult::Converted(vec![data.clone(), metadata.clone()])
            })
            .add_unrealized_cast_materializations();

        let mut mapping = ConversionValueMapping::new();

        let mut rewriter = PatternRewriter::new(&mut module, owner).unwrap();

        let report =
            convert_region_entry_signature(&mut rewriter, region, &converter, &mut mapping)
                .unwrap();

        assert_eq!(report.converted_arguments(), 1,);

        let arguments = module.block(block).unwrap().arguments().to_vec();

        assert_eq!(arguments.len(), 3,);

        assert_eq!(module.value(arguments[0]).unwrap().ty(), &data_type,);

        assert_eq!(module.value(arguments[1]).unwrap().ty(), &metadata_type,);

        assert_eq!(module.value(arguments[2]).unwrap().ty(), &Type::f64(),);

        let bridged_operand = module.operation(consumer).unwrap().operand(0).unwrap();

        assert_eq!(module.value(bridged_operand).unwrap().ty(), &field_type,);

        module.verify().unwrap();
    }

    //#[test]
    //fn successor_operands_are_ssa_uses() {
    //    let mut module =
    //        Module::new();

    //    let region =
    //        module.body_region();

    //    let entry =
    //        module.body_block();

    //    let target =
    //        module
    //            .append_block(
    //                region,
    //                BlockBuilder::new()
    //                    .argument(
    //                        Type::f64(),
    //                    ),
    //            )
    //            .unwrap();

    //    let first = module
    //        .append_operation_to_block(
    //            entry,
    //            OperationBuilder::new(
    //                "test.value",
    //            )
    //            .result(Type::f64()),
    //            [],
    //            [],
    //        )
    //        .unwrap();

    //    let second = module
    //        .append_operation_to_block(
    //            entry,
    //            OperationBuilder::new(
    //                "test.value",
    //            )
    //            .result(Type::f64()),
    //            [],
    //            [],
    //        )
    //        .unwrap();

    //    let first_value =
    //        module
    //            .operation(first)
    //            .unwrap()
    //            .result(0)
    //            .unwrap();

    //    let second_value =
    //        module
    //            .operation(second)
    //            .unwrap()
    //            .result(0)
    //            .unwrap();

    //    let branch = module
    //        .insert_operation_with_successors(
    //            InsertionPoint::End(entry),
    //            OperationBuilder::new(
    //                "test.branch",
    //            ),
    //            [],
    //            [
    //                BlockSuccessor::
    //                    with_operands(
    //                        target,
    //                        [first_value],
    //                    ),
    //            ],
    //        )
    //        .unwrap();

    //    module
    //        .replace_all_uses(
    //            first_value,
    //            second_value,
    //        )
    //        .unwrap();

    //    assert_eq!(
    //        module
    //            .operation(branch)
    //            .unwrap()
    //            .successor_operands(0)
    //            .unwrap(),
    //        &[second_value],
    //    );

    //    module.verify().unwrap();
    //}

    //#[test]
    //fn signature_conversion_updates_incoming_edge() {
    //    // Create:
    //    //
    //    // branch ^target(%field)
    //    // ^target(%arg: field.value)
    //    //
    //    // Convert field.value to:
    //    // buffer.data + buffer.metadata
    //    //
    //    // Assert:
    //    //
    //    // branch ^target(%data, %metadata)
    //    // ^target(%data_arg, %metadata_arg)

    //    // Use the same field TypeConverter
    //    // from your existing 1-to-N tests.

    //    // Important assertions:

    //    assert_eq!(
    //        module
    //            .operation(branch)
    //            .unwrap()
    //            .successor_operands(0)
    //            .unwrap()
    //            .len(),
    //        2,
    //    );

    //    assert_eq!(
    //        module
    //            .block(target)
    //            .unwrap()
    //            .arguments()
    //            .len(),
    //        2,
    //    );

    //    module.verify().unwrap();
    //}
}
