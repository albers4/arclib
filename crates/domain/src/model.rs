// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use crate::DomainError;

pub type Point3 = [f64; 3];
pub type Vector3 = [f64; 3];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DomainShape {
    Line,
    Plane,
    Box,
    Sphere,
    Torus,
}

impl DomainShape {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Plane => "plane",
            Self::Box => "box",
            Self::Sphere => "sphere",
            Self::Torus => "torus",
        }
    }

    pub const fn intrinsic_dimension(self) -> u32 {
        match self {
            Self::Line => 1,
            Self::Plane => 2,
            Self::Box | Self::Sphere | Self::Torus => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineDomain {
    start: Point3,
    end: Point3,
}

impl LineDomain {
    pub fn new(start: Point3, end: Point3) -> Result<Self, DomainError> {
        finite_points([start, end])?;
        if squared_norm(sub(end, start)) <= f64::EPSILON {
            return Err(DomainError::DegenerateLine);
        }
        Ok(Self { start, end })
    }

    pub fn along_x(lower: f64, upper: f64) -> Result<Self, DomainError> {
        Self::new([lower, 0.0, 0.0], [upper, 0.0, 0.0])
    }

    pub const fn start(&self) -> Point3 {
        self.start
    }

    pub const fn end(&self) -> Point3 {
        self.end
    }

    pub fn direction(&self) -> Vector3 {
        let delta = sub(self.end, self.start);
        let length = self.length();
        [delta[0] / length, delta[1] / length, delta[2] / length]
    }

    pub fn length(&self) -> f64 {
        squared_norm(sub(self.end, self.start)).sqrt()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaneDomain {
    origin: Point3,
    axis_u: Vector3,
    axis_v: Vector3,
    extent_u: f64,
    extent_v: f64,
}

impl PlaneDomain {
    pub fn new(
        origin: Point3,
        axis_u: Vector3,
        axis_v: Vector3,
        extent_u: f64,
        extent_v: f64,
    ) -> Result<Self, DomainError> {
        finite_points([origin, axis_u, axis_v])?;
        positive("u", extent_u)?;
        positive("v", extent_v)?;
        let cross = cross(axis_u, axis_v);
        if squared_norm(axis_u) <= f64::EPSILON
            || squared_norm(axis_v) <= f64::EPSILON
            || squared_norm(cross) <= f64::EPSILON
        {
            return Err(DomainError::DegeneratePlane);
        }
        Ok(Self {
            origin,
            axis_u: normalize(axis_u),
            axis_v: normalize(axis_v),
            extent_u,
            extent_v,
        })
    }

    pub fn xy(lower_x: f64, upper_x: f64, lower_y: f64, upper_y: f64) -> Result<Self, DomainError> {
        if !(lower_x < upper_x) {
            return Err(DomainError::InvalidBounds {
                axis: "x",
                lower: lower_x,
                upper: upper_x,
            });
        }
        if !(lower_y < upper_y) {
            return Err(DomainError::InvalidBounds {
                axis: "y",
                lower: lower_y,
                upper: upper_y,
            });
        }
        Self::new(
            [lower_x, lower_y, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            upper_x - lower_x,
            upper_y - lower_y,
        )
    }

    pub const fn origin(&self) -> Point3 {
        self.origin
    }
    pub const fn axis_u(&self) -> Vector3 {
        self.axis_u
    }
    pub const fn axis_v(&self) -> Vector3 {
        self.axis_v
    }
    pub const fn extent_u(&self) -> f64 {
        self.extent_u
    }
    pub const fn extent_v(&self) -> f64 {
        self.extent_v
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxDomain {
    lower: Point3,
    upper: Point3,
}

impl BoxDomain {
    pub fn new(lower: Point3, upper: Point3) -> Result<Self, DomainError> {
        finite_points([lower, upper])?;
        for (axis, index) in [("x", 0), ("y", 1), ("z", 2)] {
            if !(lower[index] < upper[index]) {
                return Err(DomainError::InvalidBounds {
                    axis,
                    lower: lower[index],
                    upper: upper[index],
                });
            }
        }
        Ok(Self { lower, upper })
    }

    pub const fn lower(&self) -> Point3 {
        self.lower
    }
    pub const fn upper(&self) -> Point3 {
        self.upper
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SphereDomain {
    center: Point3,
    radius: f64,
}

impl SphereDomain {
    pub fn new(center: Point3, radius: f64) -> Result<Self, DomainError> {
        finite_points([center])?;
        radius_value("sphere", radius)?;
        Ok(Self { center, radius })
    }

    pub const fn center(&self) -> Point3 {
        self.center
    }
    pub const fn radius(&self) -> f64 {
        self.radius
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TorusDomain {
    center: Point3,
    axis: Vector3,
    major_radius: f64,
    minor_radius: f64,
}

impl TorusDomain {
    pub fn new(
        center: Point3,
        axis: Vector3,
        major_radius: f64,
        minor_radius: f64,
    ) -> Result<Self, DomainError> {
        finite_points([center, axis])?;
        if squared_norm(axis) <= f64::EPSILON {
            return Err(DomainError::DegenerateAxis);
        }
        if !major_radius.is_finite()
            || !minor_radius.is_finite()
            || minor_radius <= 0.0
            || major_radius <= minor_radius
        {
            return Err(DomainError::InvalidTorusRadii {
                major: major_radius,
                minor: minor_radius,
            });
        }
        Ok(Self {
            center,
            axis: normalize(axis),
            major_radius,
            minor_radius,
        })
    }

    pub const fn center(&self) -> Point3 {
        self.center
    }
    pub const fn axis(&self) -> Vector3 {
        self.axis
    }
    pub const fn major_radius(&self) -> f64 {
        self.major_radius
    }
    pub const fn minor_radius(&self) -> f64 {
        self.minor_radius
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DomainGeometry {
    Line(LineDomain),
    Plane(PlaneDomain),
    Box(BoxDomain),
    Sphere(SphereDomain),
    Torus(TorusDomain),
}

impl DomainGeometry {
    pub const fn shape(&self) -> DomainShape {
        match self {
            Self::Line(_) => DomainShape::Line,
            Self::Plane(_) => DomainShape::Plane,
            Self::Box(_) => DomainShape::Box,
            Self::Sphere(_) => DomainShape::Sphere,
            Self::Torus(_) => DomainShape::Torus,
        }
    }

    pub const fn intrinsic_dimension(&self) -> u32 {
        self.shape().intrinsic_dimension()
    }

    pub const fn embedding_dimension(&self) -> u32 {
        3
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DomainDescription {
    name: Arc<str>,
    geometry: DomainGeometry,
}

impl DomainDescription {
    pub fn new(name: impl AsRef<str>, geometry: DomainGeometry) -> Result<Self, DomainError> {
        let name = name.as_ref().trim();
        if name.is_empty() {
            return Err(DomainError::EmptyName);
        }
        Ok(Self {
            name: Arc::from(name),
            geometry,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn geometry(&self) -> &DomainGeometry {
        &self.geometry
    }

    pub fn canonical_boundaries(&self) -> Vec<BoundaryDescription> {
        let selectors: &[(&str, &str)] = match self.geometry.shape() {
            DomainShape::Line => &[("start", "start"), ("end", "end")],
            DomainShape::Plane => &[
                ("u_min", "u_min"),
                ("u_max", "u_max"),
                ("v_min", "v_min"),
                ("v_max", "v_max"),
            ],
            DomainShape::Box => &[
                ("x_min", "x_min"),
                ("x_max", "x_max"),
                ("y_min", "y_min"),
                ("y_max", "y_max"),
                ("z_min", "z_min"),
                ("z_max", "z_max"),
            ],
            DomainShape::Sphere | DomainShape::Torus => &[("surface", "surface")],
        };
        selectors
            .iter()
            .map(|(name, selector)| BoundaryDescription {
                name: Arc::from(*name),
                selector: Arc::from(*selector),
                dimension: self.geometry.intrinsic_dimension() - 1,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryDescription {
    name: Arc<str>,
    selector: Arc<str>,
    dimension: u32,
}

impl BoundaryDescription {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn selector(&self) -> &str {
        &self.selector
    }
    pub const fn dimension(&self) -> u32 {
        self.dimension
    }
}

fn finite_points<const N: usize>(points: [Point3; N]) -> Result<(), DomainError> {
    if points.iter().flatten().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(DomainError::NonFiniteCoordinate)
    }
}

fn positive(name: &'static str, value: f64) -> Result<(), DomainError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(DomainError::InvalidExtent { name, value })
    }
}

fn radius_value(name: &'static str, value: f64) -> Result<(), DomainError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(DomainError::InvalidRadius { name, value })
    }
}

fn sub(lhs: Vector3, rhs: Vector3) -> Vector3 {
    [lhs[0] - rhs[0], lhs[1] - rhs[1], lhs[2] - rhs[2]]
}

fn squared_norm(value: Vector3) -> f64 {
    value[0] * value[0] + value[1] * value[1] + value[2] * value[2]
}

fn normalize(value: Vector3) -> Vector3 {
    let norm = squared_norm(value).sqrt();
    [value[0] / norm, value[1] / norm, value[2] / norm]
}

fn cross(lhs: Vector3, rhs: Vector3) -> Vector3 {
    [
        lhs[1] * rhs[2] - lhs[2] * rhs[1],
        lhs[2] * rhs[0] - lhs[0] * rhs[2],
        lhs[0] * rhs[1] - lhs[1] * rhs[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_all_supported_shapes() {
        assert!(LineDomain::along_x(0.0, 1.0).is_ok());
        assert!(PlaneDomain::xy(0.0, 1.0, 0.0, 2.0).is_ok());
        assert!(BoxDomain::new([0.0; 3], [1.0; 3]).is_ok());
        assert!(SphereDomain::new([0.0; 3], 1.0).is_ok());
        assert!(TorusDomain::new([0.0; 3], [0.0, 0.0, 1.0], 2.0, 0.5).is_ok());
    }
}
