// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod edge;
mod error;
mod plan;
mod planner;
mod registry;
mod stage;

pub use edge::{ConversionEdgeDescriptor, EdgeApplicability};

pub use error::PlanningError;

pub use plan::{ConversionPlan, ConversionStep, PlannedCompilationReport, PlanningConfig};

pub use planner::plan_conversion;

pub use registry::ConversionRegistry;

pub use stage::ConversionStage;
