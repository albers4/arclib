// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use mesh::StructuredLineMesh;

use crate::FvmError;

/// FVM-specific view derived from a method-neutral structured line mesh.
///
/// A future FEM view would derive element nodes, quadrature, and DOFs instead.
#[derive(Debug, Clone, PartialEq)]
pub struct FvmLineMeshView {
    owner: Vec<i32>,
    neighbour: Vec<i32>,
    face_coefficients: Vec<f64>,
    cell_volumes: Vec<f64>,
}

impl FvmLineMeshView {
    pub fn from_mesh(mesh: &StructuredLineMesh) -> Result<Self, FvmError> {
        if mesh.cells() < 2 {
            return Err(FvmError::MeshRequiresAtLeastTwoCells);
        }
        let internal_faces = mesh.internal_faces();
        let mut owner = Vec::with_capacity(internal_faces);
        let mut neighbour = Vec::with_capacity(internal_faces);
        for face in 0..internal_faces {
            owner.push(i32::try_from(face).expect("FVM owner index exceeds i32"));
            neighbour.push(i32::try_from(face + 1).expect("FVM neighbour index exceeds i32"));
        }
        Ok(Self {
            owner,
            neighbour,
            // Face area is one in the reduced 1D model; delta coefficient is A/d.
            face_coefficients: vec![1.0 / mesh.spacing(); internal_faces],
            cell_volumes: vec![mesh.spacing(); mesh.cells()],
        })
    }

    pub fn owner(&self) -> &[i32] {
        &self.owner
    }
    pub fn neighbour(&self) -> &[i32] {
        &self.neighbour
    }
    pub fn face_coefficients(&self) -> &[f64] {
        &self.face_coefficients
    }
    pub fn cell_volumes(&self) -> &[f64] {
        &self.cell_volumes
    }
    pub fn cell_count(&self) -> usize {
        self.cell_volumes.len()
    }
    pub fn internal_face_count(&self) -> usize {
        self.owner.len()
    }
}
