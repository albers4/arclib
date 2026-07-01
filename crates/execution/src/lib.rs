// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod command;
mod error;
mod graph;
mod plan;
mod resource;

pub mod memory;

pub use command::{BufferCopy, BufferFill, ExecutionCommand, KernelInvocation};
pub use error::{ExecutionError, ExecutionPlanError};
pub use graph::{
    ExecutionGraph, ExecutionGraphBuilder, ExecutionNode, ExecutionNodeId, ExecutionSchedule,
};
pub use memory::{
    BufferLifetime, BufferRequest, BufferSpec, MemoryPlan, MemoryPlanningError, MemorySlot,
    MemorySlotId, plan_execution_graph_memory, plan_execution_schedule_memory, plan_memory,
};
pub use plan::ExecutionPlan;
pub use resource::{
    BufferDeclaration, BufferProvision, MemorySpace, ResourceDeclaration, ResourceId,
    ResourceTable, ScalarValue,
};
