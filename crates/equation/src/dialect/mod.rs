// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod interfaces;
mod operations;
mod types;

pub use interfaces::{
    EquationSystemInterface, EvolutionEquationInterface, ResidualInterface, UnknownInterface,
};
pub use operations::{
    EQUATION_DIALECT, EQUATION_STAGE, EVOLUTION_OPERATION, EVOLUTION_RHS_OPERAND,
    EVOLUTION_STATE_OPERAND, EVOLUTION_TIME_STEP_OPERAND, EquationSystemOp, EvolutionOp,
    INTEGRATOR_ATTRIBUTE, NAME_ATTRIBUTE, RESIDUAL_OPERATION, ResidualOp, SYSTEM_OPERATION,
    UNKNOWN_OPERATION, UnknownOp, register_equation_dialect,
};
pub use types::TimeIntegrator;
