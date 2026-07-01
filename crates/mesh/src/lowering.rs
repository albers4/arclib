// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use compiler::{AnalysisManager, Pass, PassContext, PassError};
use ir::{
    Attribute, GreedyRewriteConfig, Module, OperationId, PatternBenefit, PatternResult,
    PatternRewriter, RewriteError, RewritePattern, RewritePatternSet, apply_patterns_greedily,
};

use crate::{
    GENERATE_OPERATION, SHAPE_ATTRIBUTE, clone_mesh_request_builder, concrete_operation_for_kind,
};

pub const LOWER_DOMAIN_TO_MESH_PASS: &str = "mesh.lower-domain-to-mesh";
pub const LOWER_DOMAIN_TO_MESH_PIPELINE: &str = "mesh.domain-to-mesh";

#[derive(Debug, Clone)]
pub struct LowerDomainToMeshPass {
    config: GreedyRewriteConfig,
}

impl LowerDomainToMeshPass {
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

impl Default for LowerDomainToMeshPass {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass for LowerDomainToMeshPass {
    fn name(&self) -> &'static str {
        LOWER_DOMAIN_TO_MESH_PASS
    }

    fn run(
        &self,
        module: &mut Module,
        context: &mut PassContext,
        _analyses: &mut AnalysisManager,
    ) -> Result<(), PassError> {
        let mut patterns = RewritePatternSet::new();
        patterns.add(
            GENERATE_OPERATION,
            PatternBenefit::DEFAULT,
            LowerMeshRequest,
        );
        let report = apply_patterns_greedily(module, &patterns, &self.config)
            .map_err(|error| PassError::failed(self.name(), error.to_string()))?;
        if report.rewrites > 0 {
            context.mark_changed();
        }
        Ok(())
    }
}

struct LowerMeshRequest;

impl RewritePattern for LowerMeshRequest {
    fn name(&self) -> &'static str {
        "LowerMeshRequest"
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<PatternResult, RewriteError> {
        let (target, builder, operands) = {
            let source = rewriter
                .operation(operation)
                .expect("mesh.generate rewrite root must exist");
            let Some(shape) = source
                .attribute(SHAPE_ATTRIBUTE)
                .and_then(Attribute::as_str)
            else {
                return Ok(PatternResult::NoMatch);
            };
            let Some(target) = concrete_operation_for_kind(shape) else {
                return Ok(PatternResult::NoMatch);
            };
            let builder = clone_mesh_request_builder(source, target)
                .map_err(|message| RewriteError::message(message))?;
            (target, builder, source.operands().to_vec())
        };

        debug_assert_ne!(target, GENERATE_OPERATION);
        let replacement = rewriter.create_operation(builder, operands)?;
        let results = rewriter
            .operation(replacement)
            .expect("new concrete mesh operation must exist")
            .results()
            .to_vec();
        rewriter.replace_operation(operation, &results)?;
        Ok(PatternResult::Rewritten)
    }
}

#[cfg(test)]
mod tests {
    use compiler::CompilerSession;
    use domain::{DomainCompilerExtension, DomainShape, LineDomain, LineOp};
    use ir::Module;

    use crate::{
        GenerateMeshOp, LOWER_DOMAIN_TO_MESH_PIPELINE, MeshCompilerExtension, MeshResolution,
        STRUCTURED_LINE_OPERATION,
    };

    #[test]
    fn lowers_a_line_mesh_request_to_a_concrete_mesh_operation() {
        let session = CompilerSession::builder()
            .register_extension(DomainCompilerExtension::new())
            .unwrap()
            .register_extension(MeshCompilerExtension::new())
            .unwrap()
            .build();

        let mut module = Module::new();
        let line = LineDomain::along_x(0.0, 1.0).unwrap();
        let domain = module
            .append_operation(LineOp::builder("rod", &line), [])
            .unwrap();
        let domain = module.operation(domain).unwrap().result(0).unwrap();

        module
            .append_operation(
                GenerateMeshOp::builder(DomainShape::Line, MeshResolution::Line { cells: 32 }),
                [domain],
            )
            .unwrap();

        session.verify(&module).unwrap();
        session
            .run_pipeline(LOWER_DOMAIN_TO_MESH_PIPELINE, &mut module)
            .unwrap();
        session.verify(&module).unwrap();

        assert!(module.operations().into_iter().any(|id| {
            module
                .operation(id)
                .is_some_and(|operation| operation.name().as_str() == STRUCTURED_LINE_OPERATION)
        }));
    }
}
