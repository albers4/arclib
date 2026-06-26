// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::{BlockId, OperationId, Type, ValueId, storage::IrStorage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueProducer {
    OperationResult {
        operation: OperationId,
        index: usize,
    },

    BlockArgument {
        block: BlockId,
        index: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsePosition {
    Operand {
        index: usize,
    },

    SuccessorOperand {
        successor_index: usize,
        operand_index: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Use {
    pub user: OperationId,
    pub position: UsePosition,
}

impl Use {
    pub const fn operand(user: OperationId, index: usize) -> Self {
        Self {
            user,
            position: UsePosition::Operand { index },
        }
    }

    pub const fn successor_operand(
        user: OperationId,
        successor_index: usize,
        operand_index: usize,
    ) -> Self {
        Self {
            user,
            position: UsePosition::SuccessorOperand {
                successor_index,
                operand_index,
            },
        }
    }

    pub fn user(&self) -> OperationId {
        self.user
    }

    pub fn position(&self) -> UsePosition {
        self.position
    }
}

#[derive(Debug)]
pub struct ValueData {
    pub ty: Type,
    pub producer: ValueProducer,
    pub uses: Vec<Use>,
}

pub struct ValueRef<'a> {
    pub storage: &'a IrStorage,
    pub id: ValueId,
}

impl<'a> ValueRef<'a> {
    pub fn new(storage: &'a IrStorage, id: ValueId) -> Self {
        Self { storage, id }
    }

    fn data(&self) -> &ValueData {
        self.storage
            .value(self.id)
            .expect("validated ValueRef must remain valid")
    }

    pub fn id(&self) -> ValueId {
        self.id
    }

    pub fn ty(&self) -> &Type {
        &self.data().ty
    }

    pub fn producer(&self) -> ValueProducer {
        self.data().producer
    }

    pub fn uses(&self) -> &[Use] {
        &self.data().uses
    }

    pub fn has_uses(&self) -> bool {
        !self.data().uses.is_empty()
    }
}
