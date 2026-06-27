// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{collections::HashMap, fmt, sync::Arc};

use ir::Module;

use crate::{CapabilityRequirement, CompileRequest, Metric, PolicyValue, Property};

use super::ConversionStage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeApplicability {
    Applicable,

    Unavailable { reason: Arc<str> },
}

impl EdgeApplicability {
    pub fn unavailable(reason: impl AsRef<str>) -> Self {
        Self::Unavailable {
            reason: Arc::from(reason.as_ref()),
        }
    }

    pub fn is_applicable(&self) -> bool {
        matches!(self, Self::Applicable)
    }
}

type ApplicabilityCallback =
    Arc<dyn Fn(&Module, &CompileRequest) -> EdgeApplicability + Send + Sync>;

pub struct ConversionEdgeDescriptor {
    name: Arc<str>,

    source: ConversionStage,
    target: ConversionStage,

    pipeline: Arc<str>,

    base_cost: f64,

    properties: HashMap<Property, PolicyValue>,

    metrics: HashMap<Metric, f64>,

    applicability: ApplicabilityCallback,

    target_requirements: Vec<CapabilityRequirement>,
}

impl ConversionEdgeDescriptor {
    pub fn new(
        name: impl AsRef<str>,
        source: impl Into<ConversionStage>,
        target: impl Into<ConversionStage>,
        pipeline: impl AsRef<str>,
    ) -> Self {
        Self {
            name: Arc::from(name.as_ref()),

            source: source.into(),

            target: target.into(),

            pipeline: Arc::from(pipeline.as_ref()),

            base_cost: 1.0,

            properties: HashMap::new(),

            metrics: HashMap::new(),

            applicability: Arc::new(|_, _| EdgeApplicability::Applicable),

            target_requirements: Vec::new(),
        }
    }

    pub fn with_base_cost(mut self, cost: f64) -> Self {
        self.base_cost = cost;
        self
    }

    pub fn property(mut self, name: impl AsRef<str>, value: impl Into<PolicyValue>) -> Self {
        self.properties.insert(Property::new(name), value.into());

        self
    }

    /// Metrics are normalized planner units.
    ///
    /// Packages should not place raw seconds,
    /// bytes, or FLOPs here unless all competing
    /// packages use the same scale.
    pub fn metric(mut self, name: impl AsRef<str>, estimate: f64) -> Self {
        self.metrics.insert(Metric::new(name), estimate);

        self
    }

    pub fn applicable_when<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Module, &CompileRequest) -> EdgeApplicability + Send + Sync + 'static,
    {
        self.applicability = Arc::new(callback);

        self
    }

    pub fn requires_target(mut self, requirement: CapabilityRequirement) -> Self {
        self.target_requirements.push(requirement);

        self
    }

    pub fn target_requirements(&self) -> &[CapabilityRequirement] {
        &self.target_requirements
    }

    pub fn supports_target(&self, request: &CompileRequest) -> bool {
        self.target_requirements
            .iter()
            .all(|requirement| requirement.is_satisfied_by(request.target()))
    }

    pub fn name(&self) -> &str {
        &self.name
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

    pub fn base_cost(&self) -> f64 {
        self.base_cost
    }

    pub fn properties(&self) -> &HashMap<Property, PolicyValue> {
        &self.properties
    }

    pub fn metrics(&self) -> &HashMap<Metric, f64> {
        &self.metrics
    }

    pub fn applicability(&self, module: &Module, request: &CompileRequest) -> EdgeApplicability {
        (self.applicability)(module, request)
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if !valid_name(self.name()) {
            return Err(format!("invalid conversion edge name '{}'", self.name(),));
        }

        if !valid_name(self.source.as_str()) {
            return Err(format!("invalid source stage '{}'", self.source,));
        }

        if !valid_name(self.target.as_str()) {
            return Err(format!("invalid target stage '{}'", self.target,));
        }

        if !valid_name(self.pipeline()) {
            return Err(format!("invalid pipeline name '{}'", self.pipeline(),));
        }

        if !self.base_cost.is_finite() || self.base_cost < 0.0 {
            return Err("base cost must be finite and \
                 non-negative"
                .into());
        }

        for (metric, value) in &self.metrics {
            if !value.is_finite() || *value < 0.0 {
                return Err(format!(
                    "metric '{metric}' must \
                         be finite and \
                         non-negative"
                ));
            }
        }

        Ok(())
    }
}

impl fmt::Debug for ConversionEdgeDescriptor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConversionEdgeDescriptor")
            .field("name", &self.name)
            .field("source", &self.source)
            .field("target", &self.target)
            .field("pipeline", &self.pipeline)
            .field("base_cost", &self.base_cost)
            .field("properties", &self.properties)
            .field("metrics", &self.metrics)
            .field("target_requirements", &self.target_requirements)
            .finish_non_exhaustive()
    }
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
}
