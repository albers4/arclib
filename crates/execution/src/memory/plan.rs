// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::collections::HashMap;

use crate::{BufferSpec, ResourceId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemorySlotId(u32);

impl MemorySlotId {
    pub const fn index(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferLifetime {
    first_use: usize,
    last_use: usize,
}

impl BufferLifetime {
    pub const fn new(first_use: usize, last_use: usize) -> Self {
        Self {
            first_use,
            last_use,
        }
    }

    pub const fn first_use(self) -> usize {
        self.first_use
    }

    pub const fn last_use(self) -> usize {
        self.last_use
    }

    pub const fn overlaps(self, other: Self) -> bool {
        self.first_use <= other.last_use && other.first_use <= self.last_use
    }
}

#[derive(Debug, Clone)]
pub struct BufferRequest {
    resource: ResourceId,
    spec: BufferSpec,
    lifetime: BufferLifetime,
}

impl BufferRequest {
    pub fn new(resource: ResourceId, spec: BufferSpec, lifetime: BufferLifetime) -> Self {
        Self {
            resource,
            spec,
            lifetime,
        }
    }

    pub const fn resource(&self) -> ResourceId {
        self.resource
    }

    pub fn spec(&self) -> &BufferSpec {
        &self.spec
    }

    pub const fn lifetime(&self) -> BufferLifetime {
        self.lifetime
    }
}

#[derive(Debug, Clone)]
pub struct MemorySlot {
    id: MemorySlotId,
    spec: BufferSpec,
    resources: Vec<ResourceId>,
}

impl MemorySlot {
    pub const fn id(&self) -> MemorySlotId {
        self.id
    }

    pub fn spec(&self) -> &BufferSpec {
        &self.spec
    }

    pub fn resources(&self) -> &[ResourceId] {
        &self.resources
    }
}

#[derive(Debug, Clone, Default)]
pub struct MemoryPlan {
    slots: Vec<MemorySlot>,
    assignments: HashMap<ResourceId, MemorySlotId>,
}

impl MemoryPlan {
    pub(crate) fn new(
        slots: Vec<MemorySlot>,
        assignments: HashMap<ResourceId, MemorySlotId>,
    ) -> Self {
        Self { slots, assignments }
    }

    pub fn slots(&self) -> &[MemorySlot] {
        &self.slots
    }

    pub fn slot_for(&self, resource: ResourceId) -> Option<MemorySlotId> {
        self.assignments.get(&resource).copied()
    }

    pub fn slot(&self, id: MemorySlotId) -> Option<&MemorySlot> {
        self.slots.get(id.0 as usize)
    }
}

pub(crate) struct MemorySlotBuilder {
    pub id: MemorySlotId,
    pub spec: BufferSpec,
    pub resources: Vec<ResourceId>,
    pub available_after: usize,
}

impl MemorySlotBuilder {
    pub fn finish(self) -> MemorySlot {
        MemorySlot {
            id: self.id,
            spec: self.spec,
            resources: self.resources,
        }
    }
}

pub(crate) fn new_slot_id(index: usize) -> MemorySlotId {
    MemorySlotId(u32::try_from(index).expect("memory slot ID overflow"))
}
