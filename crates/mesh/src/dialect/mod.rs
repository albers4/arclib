// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod interfaces;
mod operations;
mod types;

pub use interfaces::{
    BoundaryMappingInterface, CellTopologyInterface, FaceTopologyInterface, GeometricMeshInterface,
    MeshFieldInterface, MeshInterface, MeshRequestInterface, StructuredMeshInterface,
};
pub use operations::{
    ASSOCIATION_ATTRIBUTE, AZIMUTHAL_CELLS_ATTRIBUTE, CELLS_ATTRIBUTE, COMPONENTS_ATTRIBUTE,
    DIMENSION_ATTRIBUTE, ELEMENT_ATTRIBUTE, EMBEDDING_DIMENSION_ATTRIBUTE, FIELD_OPERATION,
    GENERATE_OPERATION, GenerateMeshOp, MAJOR_CELLS_ATTRIBUTE, MESH_DIALECT, MESH_STAGE,
    MINOR_CELLS_ATTRIBUTE, MeshFieldOp, NAME_ATTRIBUTE, POLAR_CELLS_ATTRIBUTE,
    RADIAL_CELLS_ATTRIBUTE, RADIAL_SPHERE_OPERATION, SHAPE_ATTRIBUTE, STRUCTURED_BOX_OPERATION,
    STRUCTURED_LINE_OPERATION, STRUCTURED_PLANE_OPERATION, TOPOLOGY_ATTRIBUTE, TOROIDAL_OPERATION,
    U_CELLS_ATTRIBUTE, V_CELLS_ATTRIBUTE, X_CELLS_ATTRIBUTE, Y_CELLS_ATTRIBUTE, Z_CELLS_ATTRIBUTE,
    field_association_name, register_mesh_dialect,
};
pub(crate) use operations::{clone_mesh_request_builder, concrete_operation_for_kind};
pub use types::{MESH_FIELD_TYPE, MESH_TYPE, MeshFieldType, MeshType};
