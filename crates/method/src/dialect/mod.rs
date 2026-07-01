// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod family;
mod interfaces;
mod operations;

pub use family::MethodFamily;

pub use interfaces::{DiscreteMethodInterface, DiscretizationRequestInterface, MethodOpInterface};

pub use operations::{
    DISCRETIZE_OPERATION, DiscretizeOp, METHOD_DIALECT, METHOD_FAMILY_ATTRIBUTE,
    METHOD_KIND_ATTRIBUTE, METHOD_STAGE, register_method_dialect,
};
