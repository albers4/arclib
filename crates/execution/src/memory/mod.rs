// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod error;
mod plan;
mod planner;
mod spec;

pub use error::MemoryPlanningError;
pub use plan::{BufferLifetime, BufferRequest, MemoryPlan, MemorySlot, MemorySlotId};
pub use planner::{plan_execution_graph_memory, plan_execution_schedule_memory, plan_memory};
pub use spec::BufferSpec;

pub(crate) use plan::{MemorySlotBuilder, new_slot_id};
