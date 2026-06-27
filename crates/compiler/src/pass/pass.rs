// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use ir::Module;

use crate::{
    CompileRequest, DecisionLog, Diagnostic, DiagnosticEngine, PolicyDecision,
    analysis::AnalysisManager,
};

use super::PassError;

pub trait Pass: Send + Sync {
    fn name(&self) -> &'static str;

    fn run(
        &self,
        module: &mut Module,
        context: &mut PassContext,
        analyses: &mut AnalysisManager,
    ) -> Result<(), PassError>;
}

pub struct PassContext {
    changed: bool,

    request: Arc<CompileRequest>,

    current_pass: Option<&'static str>,

    diagnostics: DiagnosticEngine,

    decisions: DecisionLog,
}

impl PassContext {
    pub(crate) fn new(request: Arc<CompileRequest>) -> Self {
        Self {
            changed: false,
            request,
            current_pass: None,

            diagnostics: DiagnosticEngine::new(),

            decisions: DecisionLog::new(),
        }
    }

    pub fn request(&self) -> &CompileRequest {
        &self.request
    }

    pub fn current_pass(&self) -> Option<&'static str> {
        self.current_pass
    }

    pub fn emit(&mut self, mut diagnostic: Diagnostic) {
        if let Some(pass) = self.current_pass {
            diagnostic.attach_pass(pass);
        }

        self.diagnostics.emit(diagnostic);
    }

    pub fn record_decision(&mut self, mut decision: PolicyDecision) {
        if let Some(pass) = self.current_pass {
            decision.attach_pass(pass);
        }

        self.decisions.record(decision);
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        self.diagnostics.diagnostics()
    }

    pub fn decisions(&self) -> &[PolicyDecision] {
        self.decisions.decisions()
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics.error_count()
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    pub fn mark_changed(&mut self) {
        self.changed = true;
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub(crate) fn begin_pass(&mut self, pass: &'static str) {
        self.current_pass = Some(pass);
        self.changed = false;
    }

    pub(crate) fn end_pass(&mut self) {
        self.current_pass = None;
    }

    pub(crate) fn into_reports(self) -> (Vec<Diagnostic>, Vec<PolicyDecision>) {
        (
            self.diagnostics.into_diagnostics(),
            self.decisions.into_decisions(),
        )
    }
}

impl Default for PassContext {
    fn default() -> Self {
        Self::new(Arc::new(CompileRequest::default()))
    }
}
