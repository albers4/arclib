// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use ir::{DialectRegistry, Module, UnknownOperationPolicy};

use crate::{
    CompileRequest, Diagnostic, PolicyDecision,
    analysis::AnalysisManager,
    pass::{NoopInstrumentation, Pass, PassContext, PassError, PassFailure, PassInstrumentation},
};

pub struct PassManager {
    passes: Vec<Box<dyn Pass>>,
    instrumentation: Box<dyn PassInstrumentation>,

    verify_each: bool,
    unknown_policy: UnknownOperationPolicy,
}

impl PassManager {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            instrumentation: Box::new(NoopInstrumentation),
            verify_each: true,
            unknown_policy: UnknownOperationPolicy::Reject,
        }
    }

    pub fn add_pass<P>(&mut self, pass: P) -> &mut Self
    where
        P: Pass + 'static,
    {
        self.passes.push(Box::new(pass));
        self
    }

    pub fn add_boxed_pass(&mut self, pass: Box<dyn Pass>) -> &mut Self {
        self.passes.push(pass);
        self
    }

    pub fn instrumentation<I>(&mut self, instrumentation: I) -> &mut Self
    where
        I: PassInstrumentation + 'static,
    {
        self.instrumentation = Box::new(instrumentation);
        self
    }

    pub fn verify_each(mut self, enabled: bool) -> Self {
        self.verify_each = enabled;
        self
    }

    pub fn unknown_policy(mut self, policy: UnknownOperationPolicy) -> Self {
        self.unknown_policy = policy;
        self
    }

    pub fn run(
        &self,
        module: &mut Module,
        registry: &DialectRegistry,
    ) -> Result<PassReport, PassFailure> {
        self.run_with_request(module, registry, &CompileRequest::default())
    }

    pub fn run_with_request(
        &self,
        module: &mut Module,
        registry: &DialectRegistry,
        request: &CompileRequest,
    ) -> Result<PassReport, PassFailure> {
        let mut context = PassContext::new(Arc::new(request.clone()));

        let mut analyses = AnalysisManager::new();

        let mut changed = false;
        let mut passes_run = 0;

        for pass in &self.passes {
            context.begin_pass(pass.name());

            let previous_error_count = context.error_count();

            self.instrumentation.before_pass(pass.name(), module);

            let mut result = pass.run(module, &mut context, &mut analyses);

            if result.is_ok() && context.error_count() > previous_error_count {
                result = Err(PassError::failed(
                    pass.name(),
                    "pass emitted an error diagnostic",
                ));
            }

            self.instrumentation.after_pass(
                pass.name(),
                module,
                result.as_ref().map(|_| ()).map_err(|error| error),
            );

            if let Err(error) = result {
                let (diagnostics, decisions) = context.into_reports();

                return Err(PassFailure::new(error, diagnostics, decisions));
            }

            passes_run += 1;
            changed |= context.changed();

            if context.changed() {
                analyses.invalidate_all();
            }

            if self.verify_each {
                if let Err(error) = module.verify_with_registry(registry, self.unknown_policy) {
                    let pass_error = PassError::failed(pass.name(), error.to_string());

                    let (diagnostics, decisions) = context.into_reports();

                    return Err(PassFailure::new(pass_error, diagnostics, decisions));
                }
            }

            context.end_pass();
        }

        let (diagnostics, decisions) = context.into_reports();

        Ok(PassReport {
            changed,
            passes_run,
            diagnostics,
            decisions,
        })
    }
}

impl Default for PassManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PassReport {
    pub changed: bool,
    pub passes_run: usize,

    pub diagnostics: Vec<Diagnostic>,

    pub decisions: Vec<PolicyDecision>,
}
