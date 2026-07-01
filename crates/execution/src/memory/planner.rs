// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::collections::HashMap;

use crate::memory::{MemorySlotBuilder, new_slot_id};
use crate::{
    BufferLifetime, BufferPersistence, BufferProvision, BufferRequest, BufferSpec, ExecutionGraph,
    ExecutionSchedule, MemoryPlan, MemoryPlanningError, ResourceId, ResourceTable,
};

pub fn plan_memory(
    requests: impl IntoIterator<Item = BufferRequest>,
) -> Result<MemoryPlan, MemoryPlanningError> {
    let mut requests: Vec<_> = requests.into_iter().collect();

    for request in &requests {
        request
            .spec()
            .validate()
            .map_err(|message| MemoryPlanningError::InvalidBufferSpec {
                resource: request.resource(),
                message,
            })?;

        let lifetime = request.lifetime();
        if lifetime.first_use() > lifetime.last_use() {
            return Err(MemoryPlanningError::InvalidLifetime {
                resource: request.resource(),
                first_use: lifetime.first_use(),
                last_use: lifetime.last_use(),
            });
        }
    }

    requests.sort_by_key(|request| {
        (
            request.lifetime().first_use(),
            request.lifetime().last_use(),
            request.resource(),
        )
    });

    let mut slots: Vec<MemorySlotBuilder> = Vec::new();
    let mut assignments = HashMap::new();

    for request in requests {
        let lifetime = request.lifetime();
        let reusable = slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| {
                slot.available_after < lifetime.first_use()
                    && slot.spec.memory_space() == request.spec().memory_space()
            })
            .min_by_key(|(_, slot)| slot.spec.bytes().max(request.spec().bytes()))
            .map(|(index, _)| index);

        let slot_index = if let Some(index) = reusable {
            let slot = &mut slots[index];
            let bytes = slot.spec.bytes().max(request.spec().bytes());
            let alignment = slot.spec.alignment().max(request.spec().alignment());
            slot.spec = BufferSpec::new(bytes, slot.spec.memory_space()).with_alignment(alignment);
            slot.available_after = lifetime.last_use();
            slot.resources.push(request.resource());
            index
        } else {
            let index = slots.len();
            slots.push(MemorySlotBuilder {
                id: new_slot_id(index),
                spec: request.spec().clone(),
                resources: vec![request.resource()],
                available_after: lifetime.last_use(),
            });
            index
        };

        assignments.insert(request.resource(), slots[slot_index].id);
    }

    Ok(MemoryPlan::new(
        slots.into_iter().map(MemorySlotBuilder::finish).collect(),
        assignments,
    ))
}

pub fn plan_execution_graph_memory(
    graph: &ExecutionGraph,
    resources: &ResourceTable,
) -> Result<MemoryPlan, MemoryPlanningError> {
    let schedule = graph.schedule()?;
    plan_execution_schedule_memory(graph, &schedule, resources)
}

pub fn plan_execution_schedule_memory(
    graph: &ExecutionGraph,
    schedule: &ExecutionSchedule,
    resources: &ResourceTable,
) -> Result<MemoryPlan, MemoryPlanningError> {
    let final_position = schedule.order().len().saturating_sub(1);
    let mut lifetimes: HashMap<ResourceId, BufferLifetime> = HashMap::new();

    for (resource, decleration) in resources.iter() {
        let Some(buffer) = decleration.buffer() else {
            continue;
        };

        if buffer.provision() == BufferProvision::Runtime
            && buffer.persistence() == BufferPersistence::Persistent
        {
            lifetimes.insert(resource, BufferLifetime::new(0, final_position));
        }
    }

    for (position, node_id) in schedule.order().iter().copied().enumerate() {
        let command = graph
            .command(node_id)
            .expect("execution schedule contains only existing nodes");

        for (resource, _) in command.accesses() {
            let Some(buffer) = resources.buffer(resource) else {
                continue;
            };
            if buffer.provision() != BufferProvision::Runtime
                || buffer.persistence() == BufferPersistence::Persistent
            {
                continue;
            }

            lifetimes
                .entry(resource)
                .and_modify(|lifetime| {
                    *lifetime = BufferLifetime::new(
                        lifetime.first_use().min(position),
                        lifetime.last_use().max(position),
                    );
                })
                .or_insert_with(|| BufferLifetime::new(position, position));
        }
    }

    let requests = lifetimes.into_iter().map(|(resource, lifetime)| {
        let spec = resources
            .buffer(resource)
            .expect("buffer lifetime exists only for buffer resources")
            .spec()
            .clone();
        BufferRequest::new(resource, spec, lifetime)
    });

    plan_memory(requests)
}

#[cfg(test)]
mod tests {
    use crate::{
        BufferLifetime, BufferRequest, BufferSpec, MemorySpace, ResourceTable, plan_memory,
    };

    #[test]
    fn reuses_non_overlapping_buffers() {
        let mut resources = ResourceTable::new();
        let first = resources.declare_buffer(BufferSpec::new(1024, MemorySpace::Host));
        let second = resources.declare_buffer(BufferSpec::new(512, MemorySpace::Host));

        let plan = plan_memory([
            BufferRequest::new(
                first,
                resources.buffer(first).unwrap().spec().clone(),
                BufferLifetime::new(0, 2),
            ),
            BufferRequest::new(
                second,
                resources.buffer(second).unwrap().spec().clone(),
                BufferLifetime::new(3, 5),
            ),
        ])
        .unwrap();

        assert_eq!(plan.slots().len(), 1);
        assert_eq!(plan.slot_for(first), plan.slot_for(second));
        assert_eq!(plan.slots()[0].spec().bytes(), 1024);
    }

    #[test]
    fn does_not_reuse_overlapping_buffers() {
        let mut resources = ResourceTable::new();
        let first = resources.declare_buffer(BufferSpec::new(1024, MemorySpace::Host));
        let second = resources.declare_buffer(BufferSpec::new(512, MemorySpace::Host));

        let plan = plan_memory([
            BufferRequest::new(
                first,
                resources.buffer(first).unwrap().spec().clone(),
                BufferLifetime::new(0, 3),
            ),
            BufferRequest::new(
                second,
                resources.buffer(second).unwrap().spec().clone(),
                BufferLifetime::new(3, 5),
            ),
        ])
        .unwrap();

        assert_eq!(plan.slots().len(), 2);
    }
}
