// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

use ir::{DialectRegistry, IrError, Module, UnknownOperationPolicy};

use crate::{
    CompileRequest, CompilerExtension, CompilerRegistry, ConversionPlan, ConversionStage,
    Diagnostic, PassFailure, PassManager, PassReport, PlannedCompilationReport, PlanningConfig,
    PlanningError, PolicyDecision, RegistryError, RequestError, plan_conversion,
};

#[derive(Debug, Clone, Copy)]
pub struct SessionOptions {
    pub verify_input: bool,
    pub verify_each_pass: bool,

    pub unknown_operation_policy: UnknownOperationPolicy,
}

impl Default for SessionOptions {
    fn default() -> Self {
        Self {
            verify_input: true,
            verify_each_pass: true,

            unknown_operation_policy: UnknownOperationPolicy::Reject,
        }
    }
}

pub struct CompilerSession {
    registry: CompilerRegistry,
    options: SessionOptions,
}

impl CompilerSession {
    pub fn builder() -> CompilerSessionBuilder {
        CompilerSessionBuilder::new()
    }

    pub fn registry(&self) -> &CompilerRegistry {
        &self.registry
    }

    pub fn dialects(&self) -> &DialectRegistry {
        self.registry.dialects()
    }

    pub fn options(&self) -> &SessionOptions {
        &self.options
    }

    pub fn verify(&self, module: &Module) -> Result<(), SessionError> {
        module.verify_with_registry(
            self.registry.dialects(),
            self.options.unknown_operation_policy,
        )?;

        Ok(())
    }

    pub fn create_pipeline(&self, name: &str) -> Result<PassManager, SessionError> {
        let manager = self
            .registry
            .pipelines()
            .build(name, self.registry.passes())?
            .verify_each(self.options.verify_each_pass)
            .unknown_policy(self.options.unknown_operation_policy);

        Ok(manager)
    }

    pub fn run_pipeline(
        &self,
        name: &str,
        module: &mut Module,
    ) -> Result<PassReport, SessionError> {
        let request = CompileRequest::builder(name).build()?;

        self.compile(&request, module)
    }

    pub fn run_pass_manager(
        &self,
        manager: &PassManager,
        module: &mut Module,
    ) -> Result<PassReport, SessionError> {
        if self.options.verify_input {
            self.verify(module)?;
        }

        Ok(manager.run(module, self.registry.dialects())?)
    }

    pub fn run_pass_manager_with_request(
        &self,
        manager: &PassManager,
        request: &CompileRequest,
        module: &mut Module,
    ) -> Result<PassReport, SessionError> {
        request.validate()?;

        if self.options.verify_input {
            self.verify(module)?;
        }

        Ok(manager.run_with_request(module, self.registry.dialects(), request)?)
    }

    pub fn compile(
        &self,
        request: &CompileRequest,
        module: &mut Module,
    ) -> Result<PassReport, SessionError> {
        request.validate()?;

        if self.options.verify_input {
            self.verify(module)?;
        }

        let manager = self.create_pipeline(request.pipeline())?;

        Ok(manager.run_with_request(module, self.registry.dialects(), request)?)
    }

    pub fn plan_conversion(
        &self,

        source: impl Into<ConversionStage>,

        target: impl Into<ConversionStage>,

        module: &Module,

        request: &CompileRequest,
    ) -> Result<ConversionPlan, SessionError> {
        request.validate()?;

        Ok(plan_conversion(
            module,
            request,
            source,
            target,
            self.registry.conversions(),
            self.registry.pipelines(),
            &PlanningConfig::default(),
        )?)
    }

    pub fn plan_conversion_with_config(
        &self,

        source: impl Into<ConversionStage>,

        target: impl Into<ConversionStage>,

        module: &Module,

        request: &CompileRequest,

        config: &PlanningConfig,
    ) -> Result<ConversionPlan, SessionError> {
        request.validate()?;

        Ok(plan_conversion(
            module,
            request,
            source,
            target,
            self.registry.conversions(),
            self.registry.pipelines(),
            config,
        )?)
    }

    pub fn execute_conversion_plan(
        &self,

        plan: &ConversionPlan,

        request: &CompileRequest,

        module: &mut Module,
    ) -> Result<PlannedCompilationReport, SessionError> {
        request.validate()?;

        if self.options.verify_input {
            self.verify(module)?;
        }

        let mut changed = false;

        let mut passes_run = 0;

        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        let mut decisions: Vec<PolicyDecision> = plan.decisions().to_vec();

        for step in plan.steps() {
            let manager = self.create_pipeline(step.pipeline())?;

            let report = manager.run_with_request(module, self.registry.dialects(), request)?;

            changed |= report.changed;

            passes_run += report.passes_run;

            diagnostics.extend(report.diagnostics);

            decisions.extend(report.decisions);
        }

        Ok(PlannedCompilationReport::new(
            plan.clone(),
            changed,
            passes_run,
            diagnostics,
            decisions,
        ))
    }

    pub fn compile_to(
        &self,

        source: impl Into<ConversionStage>,

        target: impl Into<ConversionStage>,

        request: &CompileRequest,

        module: &mut Module,
    ) -> Result<PlannedCompilationReport, SessionError> {
        let plan = self.plan_conversion(source, target, module, request)?;

        self.execute_conversion_plan(&plan, request, module)
    }
}

pub struct CompilerSessionBuilder {
    registry: CompilerRegistry,
    options: SessionOptions,
}

impl CompilerSessionBuilder {
    pub fn new() -> Self {
        Self {
            registry: CompilerRegistry::new(),

            options: SessionOptions::default(),
        }
    }

    pub fn register_extension<E>(mut self, extension: E) -> Result<Self, RegistryError>
    where
        E: CompilerExtension,
    {
        self.registry.register_extension(extension)?;

        Ok(self)
    }

    pub fn options(mut self, options: SessionOptions) -> Self {
        self.options = options;
        self
    }

    pub fn unknown_operation_policy(mut self, policy: UnknownOperationPolicy) -> Self {
        self.options.unknown_operation_policy = policy;

        self
    }

    pub fn verify_input(mut self, enabled: bool) -> Self {
        self.options.verify_input = enabled;

        self
    }

    pub fn verify_each_pass(mut self, enabled: bool) -> Self {
        self.options.verify_each_pass = enabled;

        self
    }

    pub fn build(self) -> CompilerSession {
        CompilerSession {
            registry: self.registry,
            options: self.options,
        }
    }
}

impl Default for CompilerSessionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum SessionError {
    Registry(RegistryError),
    Ir(IrError),
    Request(RequestError),
    Pass(PassFailure),
    Planning(PlanningError),
}

impl From<RegistryError> for SessionError {
    fn from(error: RegistryError) -> Self {
        Self::Registry(error)
    }
}

impl From<PassFailure> for SessionError {
    fn from(error: PassFailure) -> Self {
        Self::Pass(error)
    }
}

impl From<IrError> for SessionError {
    fn from(error: IrError) -> Self {
        Self::Ir(error)
    }
}

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registry(error) => {
                write!(formatter, "{error}")
            }

            Self::Pass(error) => {
                write!(formatter, "{error}")
            }

            Self::Ir(error) => {
                write!(formatter, "{error}")
            }

            Self::Request(error) => {
                write!(formatter, "{error}")
            }

            Self::Planning(error) => {
                write!(formatter, "{error}")
            }
        }
    }
}

impl Error for SessionError {}

impl From<RequestError> for SessionError {
    fn from(error: RequestError) -> Self {
        Self::Request(error)
    }
}

impl From<PlanningError> for SessionError {
    fn from(error: PlanningError) -> Self {
        Self::Planning(error)
    }
}
