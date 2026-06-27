// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use ir::{OperationId, SourceLocation, SymbolRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticSeverity {
    Remark,
    Note,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticAnchor {
    Source(SourceLocation),
    Operation(OperationId),
    Symbol(SymbolRef),
}

#[derive(Debug, Clone)]
pub struct DiagnosticNote {
    message: Arc<str>,
    anchor: Option<DiagnosticAnchor>,
}

impl DiagnosticNote {
    pub fn new(message: impl AsRef<str>) -> Self {
        Self {
            message: Arc::from(message.as_ref()),
            anchor: None,
        }
    }

    pub fn at(mut self, anchor: DiagnosticAnchor) -> Self {
        self.anchor = Some(anchor);
        self
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn anchor(&self) -> Option<&DiagnosticAnchor> {
        self.anchor.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    severity: DiagnosticSeverity,
    code: Option<Arc<str>>,
    message: Arc<str>,

    pass: Option<&'static str>,
    anchor: Option<DiagnosticAnchor>,

    notes: Vec<DiagnosticNote>,
}

impl Diagnostic {
    pub fn new(severity: DiagnosticSeverity, message: impl AsRef<str>) -> Self {
        Self {
            severity,
            code: None,
            message: Arc::from(message.as_ref()),

            pass: None,
            anchor: None,

            notes: Vec::new(),
        }
    }

    pub fn remark(message: impl AsRef<str>) -> Self {
        Self::new(DiagnosticSeverity::Remark, message)
    }

    pub fn note(message: impl AsRef<str>) -> Self {
        Self::new(DiagnosticSeverity::Note, message)
    }

    pub fn warning(message: impl AsRef<str>) -> Self {
        Self::new(DiagnosticSeverity::Warning, message)
    }

    pub fn error(message: impl AsRef<str>) -> Self {
        Self::new(DiagnosticSeverity::Error, message)
    }

    pub fn with_code(mut self, code: impl AsRef<str>) -> Self {
        self.code = Some(Arc::from(code.as_ref()));

        self
    }

    pub fn at_operation(mut self, operation: OperationId) -> Self {
        self.anchor = Some(DiagnosticAnchor::Operation(operation));

        self
    }

    pub fn at_source(mut self, location: SourceLocation) -> Self {
        self.anchor = Some(DiagnosticAnchor::Source(location));

        self
    }

    pub fn at_symbol(mut self, symbol: SymbolRef) -> Self {
        self.anchor = Some(DiagnosticAnchor::Symbol(symbol));

        self
    }

    pub fn with_note(mut self, note: DiagnosticNote) -> Self {
        self.notes.push(note);
        self
    }

    pub fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn pass(&self) -> Option<&'static str> {
        self.pass
    }

    pub fn anchor(&self) -> Option<&DiagnosticAnchor> {
        self.anchor.as_ref()
    }

    pub fn notes(&self) -> &[DiagnosticNote] {
        &self.notes
    }

    pub(crate) fn attach_pass(&mut self, pass: &'static str) {
        if self.pass.is_none() {
            self.pass = Some(pass);
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiagnosticEngine {
    diagnostics: Vec<Diagnostic>,
    error_count: usize,
}

impl DiagnosticEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn emit(&mut self, diagnostic: Diagnostic) {
        if diagnostic.severity() == DiagnosticSeverity::Error {
            self.error_count += 1;
        }

        self.diagnostics.push(diagnostic);
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn error_count(&self) -> usize {
        self.error_count
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub(crate) fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}
