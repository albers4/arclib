// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

use ir::{BlockId, OperationId, RegionId, Type, ValueId};

#[derive(Debug)]
pub enum TypeConversionError {
    NoConversion {
        source: Type,
    },

    RuleFailed {
        source: Type,
        message: String,
    },

    NoMaterialization {
        kind: &'static str,
        requested: Vec<Type>,
    },

    MaterializationFailed {
        kind: &'static str,
        message: String,
    },

    MaterializationCountMismatch {
        kind: &'static str,
        expected: usize,
        actual: usize,
    },

    MaterializationTypeMismatch {
        kind: &'static str,
        value: ValueId,
        expected: Type,
        actual: Type,
    },

    ExpectedSingleValue {
        operand: usize,
        actual: usize,
    },

    ResultCountMismatch {
        expected: usize,
        actual: usize,
    },

    ResultTypeMismatch {
        result: usize,
        expected: Vec<Type>,
        actual: Vec<Type>,
    },

    SignatureArgumentOutOfBounds {
        index: usize,
        argument_count: usize,
    },

    IncompleteSignatureConversion {
        index: usize,
    },

    SignatureArgumentCountMismatch {
        expected: usize,
        actual: usize,
    },

    CannotDropUsedArgument {
        block: BlockId,
        index: usize,
        value: ValueId,
    },

    RegionHasNoEntryBlock(RegionId),

    MissingRegion(RegionId),

    MissingBlock(BlockId),

    MissingValue(ValueId),

    RegionNotEmptyDuringReplacement {
        operation: OperationId,
        region: RegionId,
    },
}

impl fmt::Display for TypeConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoConversion { source } => {
                write!(formatter, "no conversion is registered for type {source:?}")
            }

            Self::RuleFailed { source, message } => {
                write!(formatter, "conversion of type {source:?} failed: {message}")
            }

            Self::NoMaterialization { kind, requested } => {
                write!(
                    formatter,
                    "no {kind} materialization can produce {requested:?}"
                )
            }

            Self::MaterializationFailed { kind, message } => {
                write!(formatter, "{kind} materialization failed: {message}")
            }

            Self::MaterializationCountMismatch {
                kind,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "{kind} materialization produced {actual} values; \
                     expected {expected}"
                )
            }

            Self::MaterializationTypeMismatch {
                kind,
                value,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "{kind} materialization produced value {value:?} \
                     of type {actual:?}; expected {expected:?}"
                )
            }

            Self::ExpectedSingleValue { operand, actual } => {
                write!(
                    formatter,
                    "converted operand {operand} contains {actual} values; \
                     expected exactly one"
                )
            }

            Self::ResultCountMismatch { expected, actual } => {
                write!(
                    formatter,
                    "conversion supplied {actual} result groups; \
                     expected {expected}"
                )
            }

            Self::ResultTypeMismatch {
                result,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "converted result {result} has types {actual:?}; \
                     expected {expected:?}"
                )
            }

            Self::SignatureArgumentOutOfBounds {
                index,
                argument_count,
            } => {
                write!(
                    formatter,
                    "signature argument {index} is \
                    out of bounds for \
                    {argument_count} arguments"
                )
            }

            Self::IncompleteSignatureConversion { index } => {
                write!(
                    formatter,
                    "signature conversion has no \
                    mapping for argument {index}"
                )
            }

            Self::SignatureArgumentCountMismatch { expected, actual } => {
                write!(
                    formatter,
                    "signature conversion expects \
                    {expected} arguments but block \
                    has {actual}"
                )
            }

            Self::CannotDropUsedArgument {
                block,
                index,
                value,
            } => {
                write!(
                    formatter,
                    "cannot drop used argument {index} \
                    ({value:?}) of block {block:?}"
                )
            }

            Self::RegionHasNoEntryBlock(region) => {
                write!(formatter, "region {region:?} has no entry block")
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

            Self::RegionNotEmptyDuringReplacement { operation, region } => {
                write!(
                    formatter,
                    "cannot replace operation \
                    {operation:?}; region {region:?} \
                    still owns blocks—move its \
                    contents first"
                )
            }
        }
    }
}

impl Error for TypeConversionError {}
