// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod analyses;
mod analysis;
mod cfg;
mod dominance;
mod manager;

pub use analysis::Analysis;

pub use analyses::{OperationListAnalysis, OperationNameAnalysis, SymbolDefinitionAnalysis};

pub use cfg::{ControlFlowGraphAnalysis, RegionControlFlowGraph};

pub use dominance::DominanceAnalysis;

pub use manager::AnalysisManager;
