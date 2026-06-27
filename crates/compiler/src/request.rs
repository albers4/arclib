// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt, sync::Arc};

use crate::{
    TargetProfile,
    policy::{
        CompilationConstraint, ConstraintScope, ConstraintSet, OptimizationObjective,
        OptimizationObjectives, PolicyError, PolicyValue,
    },
    registry::is_valid_registry_name,
};

#[derive(Debug, Clone)]
pub struct CompileRequest {
    pipeline: Arc<str>,
    constraints: ConstraintSet,
    objectives: OptimizationObjectives,
    target: Option<TargetProfile>,
}

impl CompileRequest {
    pub fn builder(pipeline: impl AsRef<str>) -> CompileRequestBuilder {
        CompileRequestBuilder::new(pipeline)
    }

    pub fn pipeline(&self) -> &str {
        &self.pipeline
    }

    pub fn constraints(&self) -> &ConstraintSet {
        &self.constraints
    }

    pub fn objectives(&self) -> &OptimizationObjectives {
        &self.objectives
    }

    pub fn target(&self) -> Option<&TargetProfile> {
        self.target.as_ref()
    }

    pub fn validate(&self) -> Result<(), RequestError> {
        if !is_valid_registry_name(self.pipeline()) {
            return Err(RequestError::InvalidPipeline(self.pipeline().to_owned()));
        }

        self.constraints.validate()?;
        self.objectives.validate()?;

        Ok(())
    }
}

impl Default for CompileRequest {
    fn default() -> Self {
        Self {
            pipeline: Arc::from("default"),

            constraints: ConstraintSet::new(),

            objectives: OptimizationObjectives::new(),

            target: None,
        }
    }
}

pub struct CompileRequestBuilder {
    pipeline: Arc<str>,
    constraints: ConstraintSet,
    objectives: OptimizationObjectives,
    target: Option<TargetProfile>,
}

impl CompileRequestBuilder {
    pub fn new(pipeline: impl AsRef<str>) -> Self {
        Self {
            pipeline: Arc::from(pipeline.as_ref()),

            constraints: ConstraintSet::new(),

            objectives: OptimizationObjectives::new(),

            target: None,
        }
    }

    pub fn constraint(mut self, constraint: CompilationConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn target(mut self, target: TargetProfile) -> Self {
        self.target = Some(target);
        self
    }

    pub fn require(self, property: impl AsRef<str>, value: impl Into<PolicyValue>) -> Self {
        self.require_scoped(ConstraintScope::Global, property, value)
    }

    pub fn require_scoped(
        self,
        scope: ConstraintScope,
        property: impl AsRef<str>,
        value: impl Into<PolicyValue>,
    ) -> Self {
        self.constraint(CompilationConstraint::required(scope, property, value))
    }

    pub fn prefer(
        self,
        property: impl AsRef<str>,
        value: impl Into<PolicyValue>,
        weight: f64,
    ) -> Self {
        self.prefer_scoped(ConstraintScope::Global, property, value, weight)
    }

    pub fn prefer_scoped(
        self,
        scope: ConstraintScope,
        property: impl AsRef<str>,
        value: impl Into<PolicyValue>,
        weight: f64,
    ) -> Self {
        self.constraint(CompilationConstraint::preferred(
            scope, property, value, weight,
        ))
    }

    pub fn hint(self, property: impl AsRef<str>, value: impl Into<PolicyValue>) -> Self {
        self.hint_scoped(ConstraintScope::Global, property, value)
    }

    pub fn hint_scoped(
        self,
        scope: ConstraintScope,
        property: impl AsRef<str>,
        value: impl Into<PolicyValue>,
    ) -> Self {
        self.constraint(CompilationConstraint::hint(scope, property, value))
    }

    pub fn objective(mut self, objective: OptimizationObjective) -> Self {
        self.objectives.push(objective);
        self
    }

    pub fn minimize(self, metric: impl AsRef<str>, weight: f64) -> Self {
        self.objective(OptimizationObjective::minimize(
            ConstraintScope::Global,
            metric,
            weight,
        ))
    }

    pub fn maximize(self, metric: impl AsRef<str>, weight: f64) -> Self {
        self.objective(OptimizationObjective::maximize(
            ConstraintScope::Global,
            metric,
            weight,
        ))
    }

    pub fn build(self) -> Result<CompileRequest, RequestError> {
        let request = CompileRequest {
            pipeline: self.pipeline,
            constraints: self.constraints,
            objectives: self.objectives,
            target: self.target,
        };

        request.validate()?;

        Ok(request)
    }
}

#[derive(Debug)]
pub enum RequestError {
    InvalidPipeline(String),
    Policy(PolicyError),
}

impl From<PolicyError> for RequestError {
    fn from(error: PolicyError) -> Self {
        Self::Policy(error)
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPipeline(name) => {
                write!(formatter, "invalid compilation pipeline '{name}'")
            }

            Self::Policy(error) => {
                write!(formatter, "{error}")
            }
        }
    }
}

impl Error for RequestError {}
