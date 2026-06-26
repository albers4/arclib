// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

use crate::{BlockId, DialectRegistryError, OperationId, RegionId, SymbolError, Type, ValueId};

#[derive(Debug)]
pub enum IrError {
    ForeignHandle {
        kind: &'static str,
    },

    MissingOperation(OperationId),
    MissingRegion(RegionId),
    MissingBlock(BlockId),
    MissingValue(ValueId),

    OperandIndexOutOfBounds {
        operation: OperationId,
        index: usize,
        operand_count: usize,
    },

    SuccessorIndexOutOfBounds {
        operation: OperationId,
        index: usize,
        successor_count: usize,
    },

    SuccessorOperandIndexOutOfBounds {
        operation: OperationId,
        successor_index: usize,
        operand_index: usize,
        operand_count: usize,
    },

    SuccessorOutsideRegion {
        operation: OperationId,
        successor: BlockId,
    },

    SuccessorOperandCountMismatch {
        operation: OperationId,
        successor_index: usize,
        block: BlockId,
        expected: usize,
        actual: usize,
    },

    SuccessorOperandTypeMismatch {
        operation: OperationId,
        successor_index: usize,
        operand_index: usize,
        value: ValueId,
        expected: Type,
        actual: Type,
    },

    BlockHasExternalPredecessor {
        block: BlockId,
        predecessor: OperationId,
    },

    InvalidModule(String),
    Corrupt(String),

    OperationHasNoParentBlock(OperationId),

    OperationNotInParentBlock {
        operation: OperationId,
        block: BlockId,
    },

    BlockArgumentIndexOutOfBounds {
        block: BlockId,
        index: usize,
        argument_count: usize,
    },

    ValueTypeMismatch {
        from: ValueId,
        to: ValueId,
        from_type: Type,
        to_type: Type,
    },

    CannotEraseRootOperation(OperationId),

    ValueEscapesErasedOperation {
        value: ValueId,
        user: OperationId,
    },

    BlockArgumentHasUses {
        block: BlockId,
        index: usize,
        value: ValueId,
    },

    CannotMoveRegionIntoItself(RegionId),

    TargetRegionNotEmpty(RegionId),

    Symbol(SymbolError),

    Dialect(DialectRegistryError),

    SuccessorOutsideParentRegion {
        parent_block: BlockId,
        successor: BlockId,
    },
}

impl fmt::Display for IrError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForeignHandle { kind } => {
                write!(formatter, "{kind} belongs to a different IR module")
            }

            Self::MissingOperation(operation) => {
                write!(formatter, "operation {operation:?} does not exist")
            }

            Self::MissingRegion(region) => {
                write!(formatter, "region {region:?} does not exist")
            }

            Self::MissingBlock(block) => {
                write!(formatter, "block {block:?} does not exist")
            }

            Self::MissingValue(value) => {
                write!(formatter, "value {value:?} does not exist")
            }

            Self::OperandIndexOutOfBounds {
                operation,
                index,
                operand_count,
            } => {
                write!(
                    formatter,
                    "operand index {index} is out of bounds for \
                     operation {operation:?} with \
                     {operand_count} operands"
                )
            }

            Self::SuccessorIndexOutOfBounds {
                operation,
                index,
                successor_count,
            } => {
                write!(
                    formatter,
                    "successor index {index} is out of bounds for \
                     operation {operation:?} with \
                     {successor_count} successors"
                )
            }

            Self::SuccessorOperandIndexOutOfBounds {
                operation,
                successor_index,
                operand_index,
                operand_count,
            } => {
                write!(
                    formatter,
                    "successor operand index {operand_index} is out \
                     of bounds for successor {successor_index} of \
                     operation {operation:?}, which has \
                     {operand_count} edge operands"
                )
            }

            Self::SuccessorOutsideRegion {
                operation,
                successor,
            } => {
                write!(
                    formatter,
                    "successor block {successor:?} of operation \
                     {operation:?} is outside the operation's \
                     parent region"
                )
            }

            Self::SuccessorOperandCountMismatch {
                operation,
                successor_index,
                block,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "successor {successor_index} of operation \
                     {operation:?} passes {actual} operands to \
                     block {block:?}, which expects {expected}"
                )
            }

            Self::SuccessorOperandTypeMismatch {
                operation,
                successor_index,
                operand_index,
                value,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "successor operand {operand_index} on edge \
                     {successor_index} of operation {operation:?} \
                     uses value {value:?} of type {actual:?}; \
                     expected {expected:?}"
                )
            }

            Self::BlockHasExternalPredecessor { block, predecessor } => {
                write!(
                    formatter,
                    "cannot erase block {block:?}; operation \
                     {predecessor:?} outside the erased subtree \
                     branches to it"
                )
            }

            Self::InvalidModule(message) | Self::Corrupt(message) => formatter.write_str(message),

            Self::OperationHasNoParentBlock(operation) => {
                write!(formatter, "operation {operation:?} has no parent block")
            }

            Self::OperationNotInParentBlock { operation, block } => {
                write!(
                    formatter,
                    "operation {operation:?} is not present in \
                     its parent block {block:?}"
                )
            }

            Self::BlockArgumentIndexOutOfBounds {
                block,
                index,
                argument_count,
            } => {
                write!(
                    formatter,
                    "block argument index {index} is out of bounds \
                     for block {block:?} with \
                     {argument_count} arguments"
                )
            }

            Self::ValueTypeMismatch {
                from,
                to,
                from_type,
                to_type,
            } => {
                write!(
                    formatter,
                    "cannot replace value {from:?} of type \
                     {from_type:?} with value {to:?} of type \
                     {to_type:?}"
                )
            }

            Self::CannotEraseRootOperation(operation) => {
                write!(formatter, "cannot erase root operation {operation:?}")
            }

            Self::ValueEscapesErasedOperation { value, user } => {
                write!(
                    formatter,
                    "value {value:?} is used by operation \
                     {user:?} outside the erased operation subtree"
                )
            }

            Self::BlockArgumentHasUses {
                block,
                index,
                value,
            } => {
                write!(
                    formatter,
                    "cannot erase argument {index} of block \
                     {block:?}; value {value:?} still has uses"
                )
            }

            Self::CannotMoveRegionIntoItself(region) => {
                write!(formatter, "cannot move region {region:?} into itself")
            }

            Self::TargetRegionNotEmpty(region) => {
                write!(formatter, "target region {region:?} is not empty")
            }

            Self::Symbol(error) => {
                write!(formatter, "{error}")
            }

            Self::Dialect(error) => {
                write!(formatter, "{error}")
            }

            Self::SuccessorOutsideParentRegion {
                parent_block,
                successor,
            } => {
                write!(
                    formatter,
                    "successor block {successor:?} is not in \
                    the same region as parent block \
                    {parent_block:?}"
                )
            }
        }
    }
}

impl Error for IrError {}

impl From<SymbolError> for IrError {
    fn from(error: SymbolError) -> Self {
        Self::Symbol(error)
    }
}

impl From<DialectRegistryError> for IrError {
    fn from(error: DialectRegistryError) -> Self {
        Self::Dialect(error)
    }
}
