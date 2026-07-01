// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use ir::{Type, TypeParameter};

use crate::{MeshEntity, MeshTopology};

pub const MESH_TYPE: &str = "mesh.mesh";
pub const MESH_FIELD_TYPE: &str = "mesh.field";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshType {
    intrinsic_dimension: u32,
    embedding_dimension: u32,
    topology: MeshTopology,
}

impl MeshType {
    pub const fn new(
        intrinsic_dimension: u32,
        embedding_dimension: u32,
        topology: MeshTopology,
    ) -> Self {
        Self {
            intrinsic_dimension,
            embedding_dimension,
            topology,
        }
    }

    pub const fn intrinsic_dimension(self) -> u32 {
        self.intrinsic_dimension
    }

    pub const fn embedding_dimension(self) -> u32 {
        self.embedding_dimension
    }

    pub const fn topology(self) -> MeshTopology {
        self.topology
    }

    pub fn ir_type(self) -> Type {
        Type::dialect(
            MESH_TYPE,
            vec![
                TypeParameter::Integer(i64::from(self.intrinsic_dimension)),
                TypeParameter::Integer(i64::from(self.embedding_dimension)),
                TypeParameter::String(Arc::from(self.topology.as_str())),
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeshFieldType {
    element: Arc<str>,
    components: u32,
    association: MeshEntity,
    mesh_dimension: u32,
}

impl MeshFieldType {
    pub fn new(
        element: impl AsRef<str>,
        components: u32,
        association: MeshEntity,
        mesh_dimension: u32,
    ) -> Self {
        Self {
            element: Arc::from(element.as_ref()),
            components,
            association,
            mesh_dimension,
        }
    }

    pub fn cell_scalar_f64(mesh_dimension: u32) -> Self {
        Self::new("f64", 1, MeshEntity::Cell, mesh_dimension)
    }

    pub fn face_scalar_f64(mesh_dimension: u32) -> Self {
        Self::new("f64", 1, MeshEntity::Face, mesh_dimension)
    }

    pub fn element(&self) -> &str {
        &self.element
    }

    pub const fn components(&self) -> u32 {
        self.components
    }

    pub const fn association(&self) -> MeshEntity {
        self.association
    }

    pub const fn mesh_dimension(&self) -> u32 {
        self.mesh_dimension
    }

    pub fn ir_type(&self) -> Type {
        Type::dialect(
            MESH_FIELD_TYPE,
            vec![
                TypeParameter::String(self.element.clone()),
                TypeParameter::Integer(i64::from(self.components)),
                TypeParameter::String(Arc::from(self.association.as_str())),
                TypeParameter::Integer(i64::from(self.mesh_dimension)),
            ],
        )
    }
}
