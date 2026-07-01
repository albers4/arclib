// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod error;
mod extension;
mod lowering;
mod model;

pub mod dialect;

pub use dialect::*;
pub use error::MeshError;
pub use extension::{MESH_TO_METHOD_PIPELINE, MeshCompilerExtension};
pub use lowering::{
    LOWER_DOMAIN_TO_MESH_PASS, LOWER_DOMAIN_TO_MESH_PIPELINE, LowerDomainToMeshPass,
};
pub use model::{
    MeshDescription, MeshEntity, MeshResolution, MeshTopology, RadialSphereMesh, StructuredBoxMesh,
    StructuredLineMesh, StructuredPlaneMesh, ToroidalMesh,
};
