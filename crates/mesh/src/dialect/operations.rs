// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use ir::{
    Attribute, DialectDescriptor, DialectRegistry, DialectRegistryError, OperationBuilder,
    OperationDescriptor, OperationRef, Pure, Type,
};

use domain::DomainShape;

use crate::{MeshEntity, MeshResolution};

use super::{
    BoundaryMappingInterface, CellTopologyInterface, FaceTopologyInterface, GeometricMeshInterface,
    MeshFieldInterface, MeshFieldType, MeshInterface, MeshRequestInterface, MeshType,
    StructuredMeshInterface,
};

pub const MESH_DIALECT: &str = "mesh";
pub const MESH_STAGE: &str = "mesh";

pub const GENERATE_OPERATION: &str = "mesh.generate";
pub const STRUCTURED_LINE_OPERATION: &str = "mesh.structured_line";
pub const STRUCTURED_PLANE_OPERATION: &str = "mesh.structured_plane";
pub const STRUCTURED_BOX_OPERATION: &str = "mesh.structured_box";
pub const RADIAL_SPHERE_OPERATION: &str = "mesh.radial_sphere";
pub const TOROIDAL_OPERATION: &str = "mesh.toroidal";
pub const FIELD_OPERATION: &str = "mesh.field";

pub const SHAPE_ATTRIBUTE: &str = "mesh.domain_shape";
pub const TOPOLOGY_ATTRIBUTE: &str = "mesh.topology";
pub const DIMENSION_ATTRIBUTE: &str = "mesh.dimension";
pub const EMBEDDING_DIMENSION_ATTRIBUTE: &str = "mesh.embedding_dimension";
pub const NAME_ATTRIBUTE: &str = "mesh.name";
pub const ELEMENT_ATTRIBUTE: &str = "mesh.element";
pub const COMPONENTS_ATTRIBUTE: &str = "mesh.components";
pub const ASSOCIATION_ATTRIBUTE: &str = "mesh.association";

pub const CELLS_ATTRIBUTE: &str = "mesh.cells";
pub const U_CELLS_ATTRIBUTE: &str = "mesh.u_cells";
pub const V_CELLS_ATTRIBUTE: &str = "mesh.v_cells";
pub const X_CELLS_ATTRIBUTE: &str = "mesh.x_cells";
pub const Y_CELLS_ATTRIBUTE: &str = "mesh.y_cells";
pub const Z_CELLS_ATTRIBUTE: &str = "mesh.z_cells";
pub const RADIAL_CELLS_ATTRIBUTE: &str = "mesh.radial_cells";
pub const POLAR_CELLS_ATTRIBUTE: &str = "mesh.polar_cells";
pub const AZIMUTHAL_CELLS_ATTRIBUTE: &str = "mesh.azimuthal_cells";
pub const MAJOR_CELLS_ATTRIBUTE: &str = "mesh.major_cells";
pub const MINOR_CELLS_ATTRIBUTE: &str = "mesh.minor_cells";

pub struct GenerateMeshOp;

impl GenerateMeshOp {
    pub fn builder(shape: DomainShape, resolution: MeshResolution) -> OperationBuilder {
        let topology = resolution.topology();
        let mut builder = OperationBuilder::new(GENERATE_OPERATION)
            .attribute(SHAPE_ATTRIBUTE, Attribute::string(shape.as_str()))
            .attribute(TOPOLOGY_ATTRIBUTE, Attribute::string(topology.as_str()))
            .attribute(
                DIMENSION_ATTRIBUTE,
                Attribute::Integer(i64::from(resolution.intrinsic_dimension())),
            )
            .attribute(EMBEDDING_DIMENSION_ATTRIBUTE, Attribute::Integer(3))
            .result(MeshType::new(resolution.intrinsic_dimension(), 3, topology).ir_type());

        builder = match resolution {
            MeshResolution::Line { cells } => {
                builder.attribute(CELLS_ATTRIBUTE, usize_attribute(cells))
            }
            MeshResolution::Plane { u_cells, v_cells } => builder
                .attribute(U_CELLS_ATTRIBUTE, usize_attribute(u_cells))
                .attribute(V_CELLS_ATTRIBUTE, usize_attribute(v_cells)),
            MeshResolution::Box {
                x_cells,
                y_cells,
                z_cells,
            } => builder
                .attribute(X_CELLS_ATTRIBUTE, usize_attribute(x_cells))
                .attribute(Y_CELLS_ATTRIBUTE, usize_attribute(y_cells))
                .attribute(Z_CELLS_ATTRIBUTE, usize_attribute(z_cells)),
            MeshResolution::Sphere {
                radial_cells,
                polar_cells,
                azimuthal_cells,
            } => builder
                .attribute(RADIAL_CELLS_ATTRIBUTE, usize_attribute(radial_cells))
                .attribute(POLAR_CELLS_ATTRIBUTE, usize_attribute(polar_cells))
                .attribute(AZIMUTHAL_CELLS_ATTRIBUTE, usize_attribute(azimuthal_cells)),
            MeshResolution::Torus {
                major_cells,
                minor_cells,
                radial_cells,
            } => builder
                .attribute(MAJOR_CELLS_ATTRIBUTE, usize_attribute(major_cells))
                .attribute(MINOR_CELLS_ATTRIBUTE, usize_attribute(minor_cells))
                .attribute(RADIAL_CELLS_ATTRIBUTE, usize_attribute(radial_cells)),
        };

        builder
    }
}

pub struct MeshFieldOp;

impl MeshFieldOp {
    pub fn builder(name: impl AsRef<str>, field: &MeshFieldType) -> OperationBuilder {
        OperationBuilder::new(FIELD_OPERATION)
            .attribute(NAME_ATTRIBUTE, Attribute::string(name))
            .attribute(ELEMENT_ATTRIBUTE, Attribute::string(field.element()))
            .attribute(
                COMPONENTS_ATTRIBUTE,
                Attribute::Integer(i64::from(field.components())),
            )
            .attribute(
                ASSOCIATION_ATTRIBUTE,
                Attribute::string(field.association().as_str()),
            )
            .attribute(
                DIMENSION_ATTRIBUTE,
                Attribute::Integer(i64::from(field.mesh_dimension())),
            )
            .result(field.ir_type())
    }

    pub fn builder_untyped(name: impl AsRef<str>, result_type: Type) -> OperationBuilder {
        OperationBuilder::new(FIELD_OPERATION)
            .attribute(NAME_ATTRIBUTE, Attribute::string(name))
            .result(result_type)
    }
}

pub fn register_mesh_dialect(registry: &mut DialectRegistry) -> Result<(), DialectRegistryError> {
    let mut dialect = DialectDescriptor::new(MESH_DIALECT);

    dialect.register_operation(
        OperationDescriptor::new(GENERATE_OPERATION)
            .with_interface(MeshRequestInterface)
            .with_verifier(verify_generate),
    )?;

    for (name, structured) in [
        (STRUCTURED_LINE_OPERATION, true),
        (STRUCTURED_PLANE_OPERATION, true),
        (STRUCTURED_BOX_OPERATION, true),
        (RADIAL_SPHERE_OPERATION, false),
        (TOROIDAL_OPERATION, false),
    ] {
        let mut descriptor = OperationDescriptor::new(name)
            .with_trait::<Pure>()
            .with_interface(MeshInterface)
            .with_interface(GeometricMeshInterface)
            .with_interface(CellTopologyInterface)
            .with_interface(FaceTopologyInterface)
            .with_interface(BoundaryMappingInterface)
            .with_verifier(verify_concrete_mesh);
        if structured {
            descriptor = descriptor.with_interface(StructuredMeshInterface);
        }
        dialect.register_operation(descriptor)?;
    }

    dialect.register_operation(
        OperationDescriptor::new(FIELD_OPERATION)
            .with_interface(MeshFieldInterface)
            .with_verifier(verify_field),
    )?;

    registry.register_dialect(dialect)
}

pub(crate) fn concrete_operation_for_kind(kind: &str) -> Option<&'static str> {
    match kind {
        "line" => Some(STRUCTURED_LINE_OPERATION),
        "plane" => Some(STRUCTURED_PLANE_OPERATION),
        "box" => Some(STRUCTURED_BOX_OPERATION),
        "sphere" => Some(RADIAL_SPHERE_OPERATION),
        "torus" => Some(TOROIDAL_OPERATION),
        _ => None,
    }
}

pub(crate) fn clone_mesh_request_builder(
    operation: OperationRef<'_>,
    target: &'static str,
) -> Result<OperationBuilder, String> {
    let result_type = operation
        .result_type(0)
        .ok_or_else(|| "mesh.generate is missing its result type".to_owned())?
        .clone();

    let mut builder = OperationBuilder::new(target).result(result_type);
    for name in [
        SHAPE_ATTRIBUTE,
        TOPOLOGY_ATTRIBUTE,
        DIMENSION_ATTRIBUTE,
        EMBEDDING_DIMENSION_ATTRIBUTE,
        CELLS_ATTRIBUTE,
        U_CELLS_ATTRIBUTE,
        V_CELLS_ATTRIBUTE,
        X_CELLS_ATTRIBUTE,
        Y_CELLS_ATTRIBUTE,
        Z_CELLS_ATTRIBUTE,
        RADIAL_CELLS_ATTRIBUTE,
        POLAR_CELLS_ATTRIBUTE,
        AZIMUTHAL_CELLS_ATTRIBUTE,
        MAJOR_CELLS_ATTRIBUTE,
        MINOR_CELLS_ATTRIBUTE,
    ] {
        if let Some(attribute) = operation.attribute(name) {
            builder = builder.attribute(name, attribute.clone());
        }
    }
    Ok(builder)
}

fn verify_generate(operation: OperationRef<'_>) -> Result<(), String> {
    verify_mesh_leaf(operation)?;
    nonempty_string(operation, SHAPE_ATTRIBUTE)?;
    nonempty_string(operation, TOPOLOGY_ATTRIBUTE)?;
    positive_integer(operation, DIMENSION_ATTRIBUTE)?;
    positive_integer(operation, EMBEDDING_DIMENSION_ATTRIBUTE)?;
    verify_resolution(operation)
}

fn verify_concrete_mesh(operation: OperationRef<'_>) -> Result<(), String> {
    verify_mesh_leaf(operation)?;
    nonempty_string(operation, SHAPE_ATTRIBUTE)?;
    nonempty_string(operation, TOPOLOGY_ATTRIBUTE)?;
    positive_integer(operation, DIMENSION_ATTRIBUTE)?;
    verify_resolution(operation)
}

fn verify_mesh_leaf(operation: OperationRef<'_>) -> Result<(), String> {
    if operation.operands().len() != 1 {
        return Err(format!("{} requires one domain operand", operation.name()));
    }
    if operation.results().len() != 1 {
        return Err(format!("{} requires one mesh result", operation.name()));
    }
    if !operation.regions().is_empty() || !operation.successors().is_empty() {
        return Err(format!("{} must be a leaf operation", operation.name()));
    }
    Ok(())
}

fn verify_field(operation: OperationRef<'_>) -> Result<(), String> {
    if operation.operands().len() != 1 || operation.results().len() != 1 {
        return Err("mesh.field requires one mesh operand and one field result".into());
    }
    if !operation.regions().is_empty() || !operation.successors().is_empty() {
        return Err("mesh.field must be a leaf operation".into());
    }
    nonempty_string(operation, NAME_ATTRIBUTE)
}

fn verify_resolution(operation: OperationRef<'_>) -> Result<(), String> {
    let shape = operation
        .attribute(SHAPE_ATTRIBUTE)
        .and_then(Attribute::as_str)
        .ok_or_else(|| "mesh.domain_shape must be a string".to_owned())?;
    let required: &[&str] = match shape {
        "line" => &[CELLS_ATTRIBUTE],
        "plane" => &[U_CELLS_ATTRIBUTE, V_CELLS_ATTRIBUTE],
        "box" => &[X_CELLS_ATTRIBUTE, Y_CELLS_ATTRIBUTE, Z_CELLS_ATTRIBUTE],
        "sphere" => &[
            RADIAL_CELLS_ATTRIBUTE,
            POLAR_CELLS_ATTRIBUTE,
            AZIMUTHAL_CELLS_ATTRIBUTE,
        ],
        "torus" => &[
            MAJOR_CELLS_ATTRIBUTE,
            MINOR_CELLS_ATTRIBUTE,
            RADIAL_CELLS_ATTRIBUTE,
        ],
        other => return Err(format!("unsupported mesh domain shape '{other}'")),
    };
    for name in required {
        positive_integer(operation, name)?;
    }
    Ok(())
}

fn positive_integer(operation: OperationRef<'_>, name: &str) -> Result<i64, String> {
    match operation.attribute(name) {
        Some(Attribute::Integer(value)) if *value > 0 => Ok(*value),
        Some(Attribute::Integer(_)) => Err(format!("attribute '{name}' must be positive")),
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

fn usize_attribute(value: usize) -> Attribute {
    Attribute::Integer(i64::try_from(value).expect("mesh resolution exceeds i64"))
}

pub const fn field_association_name(entity: MeshEntity) -> &'static str {
    entity.as_str()
}
