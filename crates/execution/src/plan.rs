// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::{
    ExecutionGraph, ExecutionPlanError, ExecutionSchedule, MemoryPlan, MemoryPlanningError,
    ResourceDeclaration, ResourceTable, plan_execution_schedule_memory,
};

#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    resources: ResourceTable,
    graph: ExecutionGraph,
    schedule: ExecutionSchedule,
    memory: MemoryPlan,
}

impl ExecutionPlan {
    pub fn new(
        resources: ResourceTable,
        graph: ExecutionGraph,
    ) -> Result<Self, ExecutionPlanError> {
        for (resource, declaration) in resources.iter() {
            if let ResourceDeclaration::Buffer(buffer) = declaration {
                buffer.spec().validate().map_err(|message| {
                    MemoryPlanningError::InvalidBufferSpec { resource, message }
                })?;
            }
        }

        graph.validate_resources(&resources)?;
        let schedule = graph.schedule()?;
        let memory = plan_execution_schedule_memory(&graph, &schedule, &resources)?;

        Ok(Self {
            resources,
            graph,
            schedule,
            memory,
        })
    }

    pub fn resources(&self) -> &ResourceTable {
        &self.resources
    }

    pub fn graph(&self) -> &ExecutionGraph {
        &self.graph
    }

    pub fn schedule(&self) -> &ExecutionSchedule {
        &self.schedule
    }

    pub fn memory(&self) -> &MemoryPlan {
        &self.memory
    }

    pub fn into_parts(self) -> (ResourceTable, ExecutionGraph, ExecutionSchedule, MemoryPlan) {
        (self.resources, self.graph, self.schedule, self.memory)
    }
}
