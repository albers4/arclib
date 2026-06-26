// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::{BlockId, ValueId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockSuccessor {
    block: BlockId,
    operands: Vec<ValueId>,
}

impl BlockSuccessor {
    pub fn new(block: BlockId) -> Self {
        Self {
            block,
            operands: Vec::new(),
        }
    }

    pub fn with_operands(block: BlockId, operands: impl IntoIterator<Item = ValueId>) -> Self {
        Self {
            block,
            operands: operands.into_iter().collect(),
        }
    }

    pub fn block(&self) -> BlockId {
        self.block
    }

    pub fn operands(&self) -> &[ValueId] {
        &self.operands
    }

    pub fn into_parts(self) -> (BlockId, Vec<ValueId>) {
        (self.block, self.operands)
    }
}
