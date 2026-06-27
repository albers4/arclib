// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{fmt, sync::Arc};

use super::{ConstraintScope, PolicyError, is_valid_key};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Metric(Arc<str>);

impl Metric {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self(Arc::from(name.as_ref()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Metric {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectiveDirection {
    Minimize,
    Maximize,
}

#[derive(Debug, Clone)]
pub struct OptimizationObjective {
    scope: ConstraintScope,
    metric: Metric,
    direction: ObjectiveDirection,
    weight: f64,
}

impl OptimizationObjective {
    pub fn new(
        scope: ConstraintScope,
        metric: impl AsRef<str>,
        direction: ObjectiveDirection,
        weight: f64,
    ) -> Self {
        Self {
            scope,
            metric: Metric::new(metric),
            direction,
            weight,
        }
    }

    pub fn minimize(scope: ConstraintScope, metric: impl AsRef<str>, weight: f64) -> Self {
        Self::new(scope, metric, ObjectiveDirection::Minimize, weight)
    }

    pub fn maximize(scope: ConstraintScope, metric: impl AsRef<str>, weight: f64) -> Self {
        Self::new(scope, metric, ObjectiveDirection::Maximize, weight)
    }

    pub fn scope(&self) -> &ConstraintScope {
        &self.scope
    }

    pub fn metric(&self) -> &Metric {
        &self.metric
    }

    pub fn direction(&self) -> ObjectiveDirection {
        self.direction
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }

    fn validate(&self) -> Result<(), PolicyError> {
        if !is_valid_key(self.metric.as_str()) {
            return Err(PolicyError::InvalidMetric(self.metric.as_str().to_owned()));
        }

        if !self.weight.is_finite() || self.weight <= 0.0 {
            return Err(PolicyError::InvalidWeight {
                owner: self.metric.as_str().to_owned(),
                weight: self.weight,
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct OptimizationObjectives {
    objectives: Vec<OptimizationObjective>,
}

impl OptimizationObjectives {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, objective: OptimizationObjective) {
        self.objectives.push(objective);
    }

    pub fn iter(&self) -> impl Iterator<Item = &OptimizationObjective> {
        self.objectives.iter()
    }

    pub fn validate(&self) -> Result<(), PolicyError> {
        for objective in &self.objectives {
            objective.validate()?;
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.objectives.is_empty()
    }
}
