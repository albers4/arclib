// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod error;
mod extension;
mod model;

pub mod dialect;

pub use dialect::*;
pub use error::DomainError;
pub use extension::DomainCompilerExtension;
pub use model::{
    BoundaryDescription, BoxDomain, DomainDescription, DomainGeometry, DomainShape, LineDomain,
    PlaneDomain, Point3, SphereDomain, TorusDomain, Vector3,
};
