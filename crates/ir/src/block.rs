// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::{BlockId, OperationId, RegionId, Type, ValueId, storage::IrStorage};

#[derive(Debug, Default)]
pub struct BlockBuilder {
    argument_types: Vec<Type>,
}

impl BlockBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn argument(mut self, ty: Type) -> Self {
        self.argument_types.push(ty);
        self
    }

    pub fn arguments(mut self, types: impl IntoIterator<Item = Type>) -> Self {
        self.argument_types.extend(types);
        self
    }

    pub fn into_argument_types(self) -> Vec<Type> {
        self.argument_types
    }
}

#[derive(Debug)]
pub struct BlockData {
    pub parent_region: RegionId,
    pub arguments: Vec<ValueId>,
    pub operations: Vec<OperationId>,
}

pub struct BlockRef<'a> {
    storage: &'a IrStorage,
    id: BlockId,
}

impl<'a> BlockRef<'a> {
    pub fn new(storage: &'a IrStorage, id: BlockId) -> Self {
        Self { storage, id }
    }

    fn data(&self) -> &BlockData {
        self.storage
            .block(self.id)
            .expect("validated BlockRef must remain valid")
    }

    pub fn id(&self) -> BlockId {
        self.id
    }

    pub fn parent_region(&self) -> RegionId {
        self.data().parent_region
    }

    pub fn arguments(&self) -> &[ValueId] {
        &self.data().arguments
    }

    pub fn argument(&self, index: usize) -> Option<ValueId> {
        self.data().arguments.get(index).copied()
    }

    pub fn operations(&self) -> &[OperationId] {
        &self.data().operations
    }

    pub fn first_operation(&self) -> Option<OperationId> {
        self.data().operations.first().copied()
    }

    pub fn last_operation(&self) -> Option<OperationId> {
        self.data().operations.last().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.data().operations.is_empty()
    }
}
