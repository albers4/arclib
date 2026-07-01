// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod interfaces;
mod operations;
mod types;

pub use interfaces::{
    AnalyticGeometryInterface, BoundaryInterface, DomainInterface, MeshableDomainInterface,
};
pub use operations::{
    BOUNDARY_DIMENSION_ATTRIBUTE, BOUNDARY_OPERATION, BOX_OPERATION, BoundaryOp, BoxOp,
    DOMAIN_DIALECT, DOMAIN_STAGE, EMBEDDING_DIMENSION_ATTRIBUTE, INTRINSIC_DIMENSION_ATTRIBUTE,
    LINE_OPERATION, LineOp, NAME_ATTRIBUTE, PLANE_OPERATION, PlaneOp, SELECTOR_ATTRIBUTE,
    SHAPE_ATTRIBUTE, SPHERE_OPERATION, SphereOp, TORUS_OPERATION, TorusOp, register_domain_dialect,
};
pub use types::{BoundaryType, DOMAIN_BOUNDARY_TYPE, DOMAIN_REGION_TYPE, DomainType};
