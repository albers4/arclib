// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod builtin;
mod control_flow;
mod descriptor;
mod dialect;
mod error;
mod interface;
mod registry;
mod traits;

pub use builtin::register_builtin_dialect;

pub use descriptor::OperationDescriptor;

pub use dialect::DialectDescriptor;

pub use error::DialectRegistryError;

pub use interface::{InterfaceMap, OperationVerifier, VerifyInterface};

pub use registry::{DialectRegistry, UnknownOperationPolicy};

pub use traits::{ConstantLike, IsolatedFromAbove, Pure, Terminator};

pub use control_flow::{
    BranchOpInterface, RegionBranchOpInterface, RegionBranchPoint, RegionBranchSuccessor,
    RegionBranchTerminatorInterface,
};
