// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::{BlockId, OperationId, RegionId, storage::IrStorage};

#[derive(Debug)]
pub struct RegionData {
    pub parent_operation: OperationId,
    pub blocks: Vec<BlockId>,
}

pub struct RegionRef<'a> {
    storage: &'a IrStorage,
    id: RegionId,
}

impl<'a> RegionRef<'a> {
    pub fn new(storage: &'a IrStorage, id: RegionId) -> Self {
        Self { storage, id }
    }

    fn data(&self) -> &RegionData {
        self.storage
            .region(self.id)
            .expect("validated RegionRef must remain valid")
    }

    pub fn id(&self) -> RegionId {
        self.id
    }

    pub fn parent_operation(&self) -> OperationId {
        self.data().parent_operation
    }

    pub fn blocks(&self) -> &[BlockId] {
        &self.data().blocks
    }

    pub fn entry_block(&self) -> Option<BlockId> {
        self.data().blocks.first().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.data().blocks.is_empty()
    }
}
