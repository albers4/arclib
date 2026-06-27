// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use ir::OperationId;

use super::{ConstraintScope, Metric, PolicyValue, Property};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecisionKind {
    RequiredSatisfied,
    RequiredUnsatisfied,

    PreferenceSelected,
    PreferenceRejected,

    HintApplied,
    HintIgnored,

    ObjectiveTradeoff,

    Other(Arc<str>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionSubject {
    Property(Property),
    Metric(Metric),
    Named(Arc<str>),
}

impl DecisionSubject {
    pub fn named(name: impl AsRef<str>) -> Self {
        Self::Named(Arc::from(name.as_ref()))
    }
}

#[derive(Debug, Clone)]
pub struct PolicyDecision {
    kind: PolicyDecisionKind,

    scope: ConstraintScope,
    subject: DecisionSubject,

    requested: Option<PolicyValue>,
    selected: Option<PolicyValue>,

    score: Option<f64>,
    reason: Arc<str>,

    pass: Option<&'static str>,
    operation: Option<OperationId>,
}

impl PolicyDecision {
    pub fn new(
        kind: PolicyDecisionKind,
        scope: ConstraintScope,
        subject: DecisionSubject,
        reason: impl AsRef<str>,
    ) -> Self {
        Self {
            kind,
            scope,
            subject,

            requested: None,
            selected: None,

            score: None,
            reason: Arc::from(reason.as_ref()),

            pass: None,
            operation: None,
        }
    }

    pub fn for_property(
        kind: PolicyDecisionKind,
        scope: ConstraintScope,
        property: Property,
        reason: impl AsRef<str>,
    ) -> Self {
        Self::new(kind, scope, DecisionSubject::Property(property), reason)
    }

    pub fn for_metric(
        kind: PolicyDecisionKind,
        scope: ConstraintScope,
        metric: Metric,
        reason: impl AsRef<str>,
    ) -> Self {
        Self::new(kind, scope, DecisionSubject::Metric(metric), reason)
    }

    pub fn requested(mut self, value: impl Into<PolicyValue>) -> Self {
        self.requested = Some(value.into());
        self
    }

    pub fn selected(mut self, value: impl Into<PolicyValue>) -> Self {
        self.selected = Some(value.into());
        self
    }

    pub fn score(mut self, score: f64) -> Self {
        self.score = Some(score);
        self
    }

    pub fn at_operation(mut self, operation: OperationId) -> Self {
        self.operation = Some(operation);
        self
    }

    pub fn kind(&self) -> &PolicyDecisionKind {
        &self.kind
    }

    pub fn scope(&self) -> &ConstraintScope {
        &self.scope
    }

    pub fn subject(&self) -> &DecisionSubject {
        &self.subject
    }

    pub fn requested_value(&self) -> Option<&PolicyValue> {
        self.requested.as_ref()
    }

    pub fn selected_value(&self) -> Option<&PolicyValue> {
        self.selected.as_ref()
    }

    pub fn score_value(&self) -> Option<f64> {
        self.score
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn pass(&self) -> Option<&'static str> {
        self.pass
    }

    pub fn operation(&self) -> Option<OperationId> {
        self.operation
    }

    pub(crate) fn attach_pass(&mut self, pass: &'static str) {
        if self.pass.is_none() {
            self.pass = Some(pass);
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DecisionLog {
    decisions: Vec<PolicyDecision>,
}

impl DecisionLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, decision: PolicyDecision) {
        self.decisions.push(decision);
    }

    pub fn decisions(&self) -> &[PolicyDecision] {
        &self.decisions
    }

    pub(crate) fn into_decisions(self) -> Vec<PolicyDecision> {
        self.decisions
    }
}
