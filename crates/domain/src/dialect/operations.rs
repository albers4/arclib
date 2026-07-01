// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use ir::{
    Attribute, DialectDescriptor, DialectRegistry, DialectRegistryError, OperationBuilder,
    OperationDescriptor, OperationRef, Pure,
};

use crate::{BoxDomain, DomainShape, LineDomain, PlaneDomain, SphereDomain, TorusDomain};

use super::{
    AnalyticGeometryInterface, BoundaryInterface, BoundaryType, DomainInterface, DomainType,
    MeshableDomainInterface,
};

pub const DOMAIN_DIALECT: &str = "domain";
pub const DOMAIN_STAGE: &str = "domain";

pub const LINE_OPERATION: &str = "domain.line";
pub const PLANE_OPERATION: &str = "domain.plane";
pub const BOX_OPERATION: &str = "domain.box";
pub const SPHERE_OPERATION: &str = "domain.sphere";
pub const TORUS_OPERATION: &str = "domain.torus";
pub const BOUNDARY_OPERATION: &str = "domain.boundary";

pub const NAME_ATTRIBUTE: &str = "domain.name";
pub const SELECTOR_ATTRIBUTE: &str = "domain.selector";
pub const SHAPE_ATTRIBUTE: &str = "domain.shape";
pub const INTRINSIC_DIMENSION_ATTRIBUTE: &str = "domain.intrinsic_dimension";
pub const EMBEDDING_DIMENSION_ATTRIBUTE: &str = "domain.embedding_dimension";
pub const BOUNDARY_DIMENSION_ATTRIBUTE: &str = "domain.boundary_dimension";

pub struct LineOp;

impl LineOp {
    pub fn builder(name: impl AsRef<str>, line: &LineDomain) -> OperationBuilder {
        let start = line.start();
        let end = line.end();
        geometry_builder(LINE_OPERATION, name, DomainShape::Line)
            .attribute("domain.start.x", Attribute::Float(start[0]))
            .attribute("domain.start.y", Attribute::Float(start[1]))
            .attribute("domain.start.z", Attribute::Float(start[2]))
            .attribute("domain.end.x", Attribute::Float(end[0]))
            .attribute("domain.end.y", Attribute::Float(end[1]))
            .attribute("domain.end.z", Attribute::Float(end[2]))
    }
}

pub struct PlaneOp;

impl PlaneOp {
    pub fn builder(name: impl AsRef<str>, plane: &PlaneDomain) -> OperationBuilder {
        let origin = plane.origin();
        let u = plane.axis_u();
        let v = plane.axis_v();
        geometry_builder(PLANE_OPERATION, name, DomainShape::Plane)
            .attribute("domain.origin.x", Attribute::Float(origin[0]))
            .attribute("domain.origin.y", Attribute::Float(origin[1]))
            .attribute("domain.origin.z", Attribute::Float(origin[2]))
            .attribute("domain.axis_u.x", Attribute::Float(u[0]))
            .attribute("domain.axis_u.y", Attribute::Float(u[1]))
            .attribute("domain.axis_u.z", Attribute::Float(u[2]))
            .attribute("domain.axis_v.x", Attribute::Float(v[0]))
            .attribute("domain.axis_v.y", Attribute::Float(v[1]))
            .attribute("domain.axis_v.z", Attribute::Float(v[2]))
            .attribute("domain.extent_u", Attribute::Float(plane.extent_u()))
            .attribute("domain.extent_v", Attribute::Float(plane.extent_v()))
    }
}

pub struct BoxOp;

impl BoxOp {
    pub fn builder(name: impl AsRef<str>, domain: &BoxDomain) -> OperationBuilder {
        let lower = domain.lower();
        let upper = domain.upper();
        geometry_builder(BOX_OPERATION, name, DomainShape::Box)
            .attribute("domain.lower.x", Attribute::Float(lower[0]))
            .attribute("domain.lower.y", Attribute::Float(lower[1]))
            .attribute("domain.lower.z", Attribute::Float(lower[2]))
            .attribute("domain.upper.x", Attribute::Float(upper[0]))
            .attribute("domain.upper.y", Attribute::Float(upper[1]))
            .attribute("domain.upper.z", Attribute::Float(upper[2]))
    }
}

pub struct SphereOp;

impl SphereOp {
    pub fn builder(name: impl AsRef<str>, sphere: &SphereDomain) -> OperationBuilder {
        let center = sphere.center();
        geometry_builder(SPHERE_OPERATION, name, DomainShape::Sphere)
            .attribute("domain.center.x", Attribute::Float(center[0]))
            .attribute("domain.center.y", Attribute::Float(center[1]))
            .attribute("domain.center.z", Attribute::Float(center[2]))
            .attribute("domain.radius", Attribute::Float(sphere.radius()))
    }
}

pub struct TorusOp;

impl TorusOp {
    pub fn builder(name: impl AsRef<str>, torus: &TorusDomain) -> OperationBuilder {
        let center = torus.center();
        let axis = torus.axis();
        geometry_builder(TORUS_OPERATION, name, DomainShape::Torus)
            .attribute("domain.center.x", Attribute::Float(center[0]))
            .attribute("domain.center.y", Attribute::Float(center[1]))
            .attribute("domain.center.z", Attribute::Float(center[2]))
            .attribute("domain.axis.x", Attribute::Float(axis[0]))
            .attribute("domain.axis.y", Attribute::Float(axis[1]))
            .attribute("domain.axis.z", Attribute::Float(axis[2]))
            .attribute(
                "domain.major_radius",
                Attribute::Float(torus.major_radius()),
            )
            .attribute(
                "domain.minor_radius",
                Attribute::Float(torus.minor_radius()),
            )
    }
}

pub struct BoundaryOp;

impl BoundaryOp {
    pub fn builder(
        name: impl AsRef<str>,
        selector: impl AsRef<str>,
        parent_dimension: u32,
        embedding_dimension: u32,
    ) -> OperationBuilder {
        OperationBuilder::new(BOUNDARY_OPERATION)
            .attribute(NAME_ATTRIBUTE, Attribute::string(name))
            .attribute(SELECTOR_ATTRIBUTE, Attribute::string(selector))
            .attribute(
                BOUNDARY_DIMENSION_ATTRIBUTE,
                Attribute::Integer(i64::from(parent_dimension.saturating_sub(1))),
            )
            .attribute(
                EMBEDDING_DIMENSION_ATTRIBUTE,
                Attribute::Integer(i64::from(embedding_dimension)),
            )
            .result(
                BoundaryType::new(parent_dimension.saturating_sub(1), embedding_dimension)
                    .ir_type(),
            )
    }
}

pub fn register_domain_dialect(registry: &mut DialectRegistry) -> Result<(), DialectRegistryError> {
    let mut dialect = DialectDescriptor::new(DOMAIN_DIALECT);

    dialect.register_operation(
        OperationDescriptor::new(LINE_OPERATION)
            .with_trait::<Pure>()
            .with_interface(DomainInterface)
            .with_interface(AnalyticGeometryInterface)
            .with_interface(MeshableDomainInterface)
            .with_verifier(verify_line),
    )?;

    dialect.register_operation(
        OperationDescriptor::new(PLANE_OPERATION)
            .with_trait::<Pure>()
            .with_interface(DomainInterface)
            .with_interface(AnalyticGeometryInterface)
            .with_interface(MeshableDomainInterface)
            .with_verifier(verify_plane),
    )?;

    dialect.register_operation(
        OperationDescriptor::new(BOX_OPERATION)
            .with_trait::<Pure>()
            .with_interface(DomainInterface)
            .with_interface(AnalyticGeometryInterface)
            .with_interface(MeshableDomainInterface)
            .with_verifier(verify_box),
    )?;

    dialect.register_operation(
        OperationDescriptor::new(SPHERE_OPERATION)
            .with_trait::<Pure>()
            .with_interface(DomainInterface)
            .with_interface(AnalyticGeometryInterface)
            .with_interface(MeshableDomainInterface)
            .with_verifier(verify_sphere),
    )?;

    dialect.register_operation(
        OperationDescriptor::new(TORUS_OPERATION)
            .with_trait::<Pure>()
            .with_interface(DomainInterface)
            .with_interface(AnalyticGeometryInterface)
            .with_interface(MeshableDomainInterface)
            .with_verifier(verify_torus),
    )?;

    dialect.register_operation(
        OperationDescriptor::new(BOUNDARY_OPERATION)
            .with_trait::<Pure>()
            .with_interface(BoundaryInterface)
            .with_verifier(verify_boundary),
    )?;

    registry.register_dialect(dialect)
}

fn geometry_builder(
    operation: &'static str,
    name: impl AsRef<str>,
    shape: DomainShape,
) -> OperationBuilder {
    OperationBuilder::new(operation)
        .attribute(NAME_ATTRIBUTE, Attribute::string(name))
        .attribute(SHAPE_ATTRIBUTE, Attribute::string(shape.as_str()))
        .attribute(
            INTRINSIC_DIMENSION_ATTRIBUTE,
            Attribute::Integer(i64::from(shape.intrinsic_dimension())),
        )
        .attribute(EMBEDDING_DIMENSION_ATTRIBUTE, Attribute::Integer(3))
        .result(DomainType::for_shape(shape).ir_type())
}

fn verify_line(operation: OperationRef<'_>) -> Result<(), String> {
    verify_geometry_common(operation, DomainShape::Line)?;
    let start = point(operation, "domain.start")?;
    let end = point(operation, "domain.end")?;
    let length_squared =
        (end[0] - start[0]).powi(2) + (end[1] - start[1]).powi(2) + (end[2] - start[2]).powi(2);
    if length_squared <= f64::EPSILON {
        return Err("domain.line endpoints must not coincide".into());
    }
    Ok(())
}

fn verify_plane(operation: OperationRef<'_>) -> Result<(), String> {
    verify_geometry_common(operation, DomainShape::Plane)?;
    point(operation, "domain.origin")?;
    let u = point(operation, "domain.axis_u")?;
    let v = point(operation, "domain.axis_v")?;
    positive_float(operation, "domain.extent_u")?;
    positive_float(operation, "domain.extent_v")?;
    let cross = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let cross_norm = cross.iter().map(|value| value * value).sum::<f64>();
    if cross_norm <= f64::EPSILON {
        return Err("domain.plane axes must be linearly independent".into());
    }
    Ok(())
}

fn verify_box(operation: OperationRef<'_>) -> Result<(), String> {
    verify_geometry_common(operation, DomainShape::Box)?;
    let lower = point(operation, "domain.lower")?;
    let upper = point(operation, "domain.upper")?;
    if (0..3).any(|index| lower[index] >= upper[index]) {
        return Err("domain.box requires lower coordinates below upper coordinates".into());
    }
    Ok(())
}

fn verify_sphere(operation: OperationRef<'_>) -> Result<(), String> {
    verify_geometry_common(operation, DomainShape::Sphere)?;
    point(operation, "domain.center")?;
    positive_float(operation, "domain.radius")?;
    Ok(())
}

fn verify_torus(operation: OperationRef<'_>) -> Result<(), String> {
    verify_geometry_common(operation, DomainShape::Torus)?;
    point(operation, "domain.center")?;
    let axis = point(operation, "domain.axis")?;
    if axis.iter().map(|value| value * value).sum::<f64>() <= f64::EPSILON {
        return Err("domain.torus axis must be nonzero".into());
    }
    let major = positive_float(operation, "domain.major_radius")?;
    let minor = positive_float(operation, "domain.minor_radius")?;
    if major <= minor {
        return Err("domain.torus requires major_radius > minor_radius".into());
    }
    Ok(())
}

fn verify_boundary(operation: OperationRef<'_>) -> Result<(), String> {
    verify_leaf(operation, 1, 1)?;
    nonempty_string(operation, NAME_ATTRIBUTE)?;
    nonempty_string(operation, SELECTOR_ATTRIBUTE)?;
    nonnegative_integer(operation, BOUNDARY_DIMENSION_ATTRIBUTE)?;
    positive_integer(operation, EMBEDDING_DIMENSION_ATTRIBUTE)?;
    Ok(())
}

fn verify_geometry_common(operation: OperationRef<'_>, shape: DomainShape) -> Result<(), String> {
    verify_leaf(operation, 0, 1)?;
    nonempty_string(operation, NAME_ATTRIBUTE)?;
    match operation
        .attribute(SHAPE_ATTRIBUTE)
        .and_then(Attribute::as_str)
    {
        Some(actual) if actual == shape.as_str() => {}
        _ => {
            return Err(format!(
                "{} requires domain.shape = '{}'",
                operation.name(),
                shape.as_str()
            ));
        }
    }
    if operation.result_type(0) != Some(&DomainType::for_shape(shape).ir_type()) {
        return Err(format!(
            "{} has an invalid domain result type",
            operation.name()
        ));
    }
    Ok(())
}

fn verify_leaf(operation: OperationRef<'_>, operands: usize, results: usize) -> Result<(), String> {
    if operation.operands().len() != operands {
        return Err(format!("{} requires {operands} operands", operation.name()));
    }
    if operation.results().len() != results {
        return Err(format!("{} requires {results} results", operation.name()));
    }
    if !operation.regions().is_empty() || !operation.successors().is_empty() {
        return Err(format!("{} must be a leaf operation", operation.name()));
    }
    Ok(())
}

fn point(operation: OperationRef<'_>, prefix: &str) -> Result<[f64; 3], String> {
    Ok([
        finite_float(operation, &format!("{prefix}.x"))?,
        finite_float(operation, &format!("{prefix}.y"))?,
        finite_float(operation, &format!("{prefix}.z"))?,
    ])
}

fn finite_float(operation: OperationRef<'_>, name: &str) -> Result<f64, String> {
    match operation.attribute(name) {
        Some(Attribute::Float(value)) if value.is_finite() => Ok(*value),
        Some(Attribute::Float(_)) => Err(format!("attribute '{name}' must be finite")),
        Some(_) => Err(format!("attribute '{name}' must be a float")),
        None => Err(format!("missing required attribute '{name}'")),
    }
}

fn positive_float(operation: OperationRef<'_>, name: &str) -> Result<f64, String> {
    let value = finite_float(operation, name)?;
    if value > 0.0 {
        Ok(value)
    } else {
        Err(format!("attribute '{name}' must be positive"))
    }
}

fn positive_integer(operation: OperationRef<'_>, name: &str) -> Result<i64, String> {
    match operation.attribute(name) {
        Some(Attribute::Integer(value)) if *value > 0 => Ok(*value),
        Some(Attribute::Integer(_)) => Err(format!("attribute '{name}' must be positive")),
        Some(_) => Err(format!("attribute '{name}' must be an integer")),
        None => Err(format!("missing required attribute '{name}'")),
    }
}

fn nonnegative_integer(operation: OperationRef<'_>, name: &str) -> Result<i64, String> {
    match operation.attribute(name) {
        Some(Attribute::Integer(value)) if *value >= 0 => Ok(*value),
        Some(Attribute::Integer(_)) => Err(format!("attribute '{name}' must be non-negative")),
        Some(_) => Err(format!("attribute '{name}' must be an integer")),
        None => Err(format!("missing required attribute '{name}'")),
    }
}

fn nonempty_string(operation: OperationRef<'_>, name: &str) -> Result<(), String> {
    match operation.attribute(name).and_then(Attribute::as_str) {
        Some(value) if !value.trim().is_empty() => Ok(()),
        Some(_) => Err(format!("attribute '{name}' must not be empty")),
        None => Err(format!("attribute '{name}' must be a string")),
    }
}
