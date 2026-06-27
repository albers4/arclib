// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::collections::HashMap;

use crate::{Diagnostic, Metric, PolicyDecision, PolicyValue, Property};

use super::ConversionStage;

#[derive(Debug, Clone)]
pub struct ConversionStep {
    edge: String,

    source: ConversionStage,

    target: ConversionStage,

    pipeline: String,
}

impl ConversionStep {
    pub(crate) fn new(
        edge: impl Into<String>,
        source: ConversionStage,
        target: ConversionStage,
        pipeline: impl Into<String>,
    ) -> Self {
        Self {
            edge: edge.into(),
            source,
            target,
            pipeline: pipeline.into(),
        }
    }

    pub fn edge(&self) -> &str {
        &self.edge
    }

    pub fn source(&self) -> &ConversionStage {
        &self.source
    }

    pub fn target(&self) -> &ConversionStage {
        &self.target
    }

    pub fn pipeline(&self) -> &str {
        &self.pipeline
    }
}

#[derive(Debug, Clone)]
pub struct ConversionPlan {
    source: ConversionStage,

    target: ConversionStage,

    steps: Vec<ConversionStep>,

    score: f64,

    properties: HashMap<Property, PolicyValue>,

    metrics: HashMap<Metric, f64>,

    decisions: Vec<PolicyDecision>,
}

impl ConversionPlan {
    pub(crate) fn new(
        source: ConversionStage,

        target: ConversionStage,

        steps: Vec<ConversionStep>,

        score: f64,

        properties: HashMap<Property, PolicyValue>,

        metrics: HashMap<Metric, f64>,

        decisions: Vec<PolicyDecision>,
    ) -> Self {
        Self {
            source,
            target,
            steps,
            score,
            properties,
            metrics,
            decisions,
        }
    }

    pub fn source(&self) -> &ConversionStage {
        &self.source
    }

    pub fn target(&self) -> &ConversionStage {
        &self.target
    }

    pub fn steps(&self) -> &[ConversionStep] {
        &self.steps
    }

    pub fn score(&self) -> f64 {
        self.score
    }

    pub fn properties(&self) -> &HashMap<Property, PolicyValue> {
        &self.properties
    }

    pub fn metrics(&self) -> &HashMap<Metric, f64> {
        &self.metrics
    }

    pub fn decisions(&self) -> &[PolicyDecision] {
        &self.decisions
    }
}

#[derive(Debug, Clone)]
pub struct PlanningConfig {
    pub max_depth: usize,
    pub max_candidates: usize,
}

impl Default for PlanningConfig {
    fn default() -> Self {
        Self {
            max_depth: 16,
            max_candidates: 10_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlannedCompilationReport {
    plan: ConversionPlan,

    changed: bool,
    passes_run: usize,

    diagnostics: Vec<Diagnostic>,

    decisions: Vec<PolicyDecision>,
}

impl PlannedCompilationReport {
    pub(crate) fn new(
        plan: ConversionPlan,
        changed: bool,
        passes_run: usize,
        diagnostics: Vec<Diagnostic>,
        decisions: Vec<PolicyDecision>,
    ) -> Self {
        Self {
            plan,
            changed,
            passes_run,
            diagnostics,
            decisions,
        }
    }

    pub fn plan(&self) -> &ConversionPlan {
        &self.plan
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn passes_run(&self) -> usize {
        self.passes_run
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn decisions(&self) -> &[PolicyDecision] {
        &self.decisions
    }
}
