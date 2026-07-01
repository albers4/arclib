// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod attribute;
mod block;
mod dialect;
mod error;
mod id;
mod insertion;
mod location;
mod module;
mod name;
mod operation;
mod region;
mod rewrite;
mod storage;
mod successor;
mod symbol;
mod ty;
mod value;

pub use attribute::{Attribute, AttributeMap};

pub use error::IrError;

pub use id::{
    BlockId, BlockKey, OperationId, OperationKey, RegionId, RegionKey, StorageId, ValueId, ValueKey,
};

pub use location::SourceLocation;

pub use module::{Module, ModuleError};

pub use name::OperationName;

pub use operation::{OperationBuilder, OperationMut, OperationRef};

pub use ty::{Signedness, Type, TypeParameter};

pub use value::{Use, UsePosition, ValueProducer, ValueRef};

pub use block::{BlockBuilder, BlockRef};

pub use insertion::InsertionPoint;

pub use region::RegionRef;

pub use symbol::{
    SYMBOL_NAME_ATTRIBUTE, SYMBOL_TABLE_ATTRIBUTE, SYMBOL_VISIBILITY_ATTRIBUTE, SymbolError,
    SymbolName, SymbolRef, SymbolTableRef, SymbolVisibility,
};

pub use dialect::{
    BranchOpInterface, ConstantLike, DialectDescriptor, DialectRegistry, DialectRegistryError,
    InterfaceMap, IsolatedFromAbove, OperationDescriptor, OperationVerifier, Pure,
    RegionBranchOpInterface, RegionBranchPoint, RegionBranchSuccessor,
    RegionBranchTerminatorInterface, Terminator, UNREALIZED_CAST, UnknownOperationPolicy,
    VerifyInterface, register_builtin_dialect,
};

pub use rewrite::{
    GreedyRewriteConfig, GreedyRewriteReport, PatternApplication, PatternBenefit, PatternResult,
    PatternRewriter, RewriteError, RewritePattern, RewritePatternSet, apply_patterns_greedily,
    apply_patterns_to_operation,
};

pub use successor::BlockSuccessor;
