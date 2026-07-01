// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use domain::{
    BoxDomain, DomainDescription, DomainGeometry, LineDomain, PlaneDomain, SphereDomain,
    TorusDomain,
};

use crate::MeshError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshTopology {
    Structured,
    Unstructured,
    Radial,
    Toroidal,
}

impl MeshTopology {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Structured => "structured",
            Self::Unstructured => "unstructured",
            Self::Radial => "radial",
            Self::Toroidal => "toroidal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshEntity {
    Vertex,
    Edge,
    Face,
    Cell,
}

impl MeshEntity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Vertex => "vertex",
            Self::Edge => "edge",
            Self::Face => "face",
            Self::Cell => "cell",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshResolution {
    Line {
        cells: usize,
    },
    Plane {
        u_cells: usize,
        v_cells: usize,
    },
    Box {
        x_cells: usize,
        y_cells: usize,
        z_cells: usize,
    },
    Sphere {
        radial_cells: usize,
        polar_cells: usize,
        azimuthal_cells: usize,
    },
    Torus {
        major_cells: usize,
        minor_cells: usize,
        radial_cells: usize,
    },
}

impl MeshResolution {
    pub fn validate(self) -> Result<(), MeshError> {
        match self {
            Self::Line { cells } => positive("line", cells),
            Self::Plane { u_cells, v_cells } => {
                positive("u", u_cells)?;
                positive("v", v_cells)
            }
            Self::Box {
                x_cells,
                y_cells,
                z_cells,
            } => {
                positive("x", x_cells)?;
                positive("y", y_cells)?;
                positive("z", z_cells)
            }
            Self::Sphere {
                radial_cells,
                polar_cells,
                azimuthal_cells,
            } => {
                positive("radial", radial_cells)?;
                positive("polar", polar_cells)?;
                positive("azimuthal", azimuthal_cells)
            }
            Self::Torus {
                major_cells,
                minor_cells,
                radial_cells,
            } => {
                positive("major", major_cells)?;
                positive("minor", minor_cells)?;
                positive("radial", radial_cells)
            }
        }
    }

    pub const fn intrinsic_dimension(self) -> u32 {
        match self {
            Self::Line { .. } => 1,
            Self::Plane { .. } => 2,
            Self::Box { .. } | Self::Sphere { .. } | Self::Torus { .. } => 3,
        }
    }

    pub const fn topology(self) -> MeshTopology {
        match self {
            Self::Line { .. } | Self::Plane { .. } | Self::Box { .. } => MeshTopology::Structured,
            Self::Sphere { .. } => MeshTopology::Radial,
            Self::Torus { .. } => MeshTopology::Toroidal,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructuredLineMesh {
    domain: LineDomain,
    cells: usize,
    spacing: f64,
}

impl StructuredLineMesh {
    pub fn new(domain: LineDomain, cells: usize) -> Result<Self, MeshError> {
        positive("line", cells)?;
        let spacing = domain.length() / cells as f64;
        Ok(Self {
            domain,
            cells,
            spacing,
        })
    }

    pub const fn domain(&self) -> &LineDomain {
        &self.domain
    }

    pub const fn cells(&self) -> usize {
        self.cells
    }

    pub const fn internal_faces(&self) -> usize {
        self.cells.saturating_sub(1)
    }

    pub const fn spacing(&self) -> f64 {
        self.spacing
    }

    pub fn cell_centers(&self) -> Vec<[f64; 3]> {
        let start = self.domain.start();
        let direction = self.domain.direction();
        (0..self.cells)
            .map(|cell| {
                let distance = (cell as f64 + 0.5) * self.spacing;
                [
                    start[0] + direction[0] * distance,
                    start[1] + direction[1] * distance,
                    start[2] + direction[2] * distance,
                ]
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructuredPlaneMesh {
    domain: PlaneDomain,
    u_cells: usize,
    v_cells: usize,
}

impl StructuredPlaneMesh {
    pub fn new(domain: PlaneDomain, u_cells: usize, v_cells: usize) -> Result<Self, MeshError> {
        positive("u", u_cells)?;
        positive("v", v_cells)?;
        Ok(Self {
            domain,
            u_cells,
            v_cells,
        })
    }

    pub const fn domain(&self) -> &PlaneDomain {
        &self.domain
    }
    pub const fn u_cells(&self) -> usize {
        self.u_cells
    }
    pub const fn v_cells(&self) -> usize {
        self.v_cells
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructuredBoxMesh {
    domain: BoxDomain,
    x_cells: usize,
    y_cells: usize,
    z_cells: usize,
}

impl StructuredBoxMesh {
    pub fn new(
        domain: BoxDomain,
        x_cells: usize,
        y_cells: usize,
        z_cells: usize,
    ) -> Result<Self, MeshError> {
        positive("x", x_cells)?;
        positive("y", y_cells)?;
        positive("z", z_cells)?;
        Ok(Self {
            domain,
            x_cells,
            y_cells,
            z_cells,
        })
    }

    pub const fn domain(&self) -> &BoxDomain {
        &self.domain
    }
    pub const fn x_cells(&self) -> usize {
        self.x_cells
    }
    pub const fn y_cells(&self) -> usize {
        self.y_cells
    }
    pub const fn z_cells(&self) -> usize {
        self.z_cells
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadialSphereMesh {
    domain: SphereDomain,
    radial_cells: usize,
    polar_cells: usize,
    azimuthal_cells: usize,
}

impl RadialSphereMesh {
    pub fn new(
        domain: SphereDomain,
        radial_cells: usize,
        polar_cells: usize,
        azimuthal_cells: usize,
    ) -> Result<Self, MeshError> {
        positive("radial", radial_cells)?;
        positive("polar", polar_cells)?;
        positive("azimuthal", azimuthal_cells)?;
        Ok(Self {
            domain,
            radial_cells,
            polar_cells,
            azimuthal_cells,
        })
    }

    pub const fn domain(&self) -> &SphereDomain {
        &self.domain
    }
    pub const fn radial_cells(&self) -> usize {
        self.radial_cells
    }
    pub const fn polar_cells(&self) -> usize {
        self.polar_cells
    }
    pub const fn azimuthal_cells(&self) -> usize {
        self.azimuthal_cells
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToroidalMesh {
    domain: TorusDomain,
    major_cells: usize,
    minor_cells: usize,
    radial_cells: usize,
}

impl ToroidalMesh {
    pub fn new(
        domain: TorusDomain,
        major_cells: usize,
        minor_cells: usize,
        radial_cells: usize,
    ) -> Result<Self, MeshError> {
        positive("major", major_cells)?;
        positive("minor", minor_cells)?;
        positive("radial", radial_cells)?;
        Ok(Self {
            domain,
            major_cells,
            minor_cells,
            radial_cells,
        })
    }

    pub const fn domain(&self) -> &TorusDomain {
        &self.domain
    }
    pub const fn major_cells(&self) -> usize {
        self.major_cells
    }
    pub const fn minor_cells(&self) -> usize {
        self.minor_cells
    }
    pub const fn radial_cells(&self) -> usize {
        self.radial_cells
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MeshDescription {
    StructuredLine(StructuredLineMesh),
    StructuredPlane(StructuredPlaneMesh),
    StructuredBox(StructuredBoxMesh),
    RadialSphere(RadialSphereMesh),
    Toroidal(ToroidalMesh),
}

impl MeshDescription {
    pub fn generate(
        domain: &DomainDescription,
        resolution: MeshResolution,
    ) -> Result<Self, MeshError> {
        resolution.validate()?;
        match (domain.geometry(), resolution) {
            (DomainGeometry::Line(line), MeshResolution::Line { cells }) => Ok(
                Self::StructuredLine(StructuredLineMesh::new(line.clone(), cells)?),
            ),
            (DomainGeometry::Plane(plane), MeshResolution::Plane { u_cells, v_cells }) => Ok(
                Self::StructuredPlane(StructuredPlaneMesh::new(plane.clone(), u_cells, v_cells)?),
            ),
            (
                DomainGeometry::Box(domain),
                MeshResolution::Box {
                    x_cells,
                    y_cells,
                    z_cells,
                },
            ) => Ok(Self::StructuredBox(StructuredBoxMesh::new(
                domain.clone(),
                x_cells,
                y_cells,
                z_cells,
            )?)),
            (
                DomainGeometry::Sphere(sphere),
                MeshResolution::Sphere {
                    radial_cells,
                    polar_cells,
                    azimuthal_cells,
                },
            ) => Ok(Self::RadialSphere(RadialSphereMesh::new(
                sphere.clone(),
                radial_cells,
                polar_cells,
                azimuthal_cells,
            )?)),
            (
                DomainGeometry::Torus(torus),
                MeshResolution::Torus {
                    major_cells,
                    minor_cells,
                    radial_cells,
                },
            ) => Ok(Self::Toroidal(ToroidalMesh::new(
                torus.clone(),
                major_cells,
                minor_cells,
                radial_cells,
            )?)),
            _ => Err(MeshError::ResolutionShapeMismatch),
        }
    }
}

fn positive(axis: &'static str, cells: usize) -> Result<(), MeshError> {
    if cells > 0 {
        Ok(())
    } else {
        Err(MeshError::InvalidResolution { axis, cells })
    }
}

#[cfg(test)]
mod tests {
    use domain::{DomainDescription, DomainGeometry, LineDomain};

    use super::*;

    #[test]
    fn generates_embedded_line_mesh() {
        let domain = DomainDescription::new(
            "line",
            DomainGeometry::Line(LineDomain::new([0.0, 0.0, 0.0], [1.0, 1.0, 0.0]).unwrap()),
        )
        .unwrap();
        let mesh = MeshDescription::generate(&domain, MeshResolution::Line { cells: 10 }).unwrap();
        let MeshDescription::StructuredLine(mesh) = mesh else {
            panic!("expected a line mesh");
        };
        assert!((mesh.spacing() - 2.0_f64.sqrt() / 10.0).abs() < 1.0e-12);
    }
}
