// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use ir::{Type, TypeParameter};

use crate::DomainShape;

pub const DOMAIN_REGION_TYPE: &str = "domain.region";
pub const DOMAIN_BOUNDARY_TYPE: &str = "domain.boundary";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DomainType {
    intrinsic_dimension: u32,
    embedding_dimension: u32,
    shape: DomainShape,
}

impl DomainType {
    pub const fn new(
        intrinsic_dimension: u32,
        embedding_dimension: u32,
        shape: DomainShape,
    ) -> Self {
        Self {
            intrinsic_dimension,
            embedding_dimension,
            shape,
        }
    }

    pub const fn for_shape(shape: DomainShape) -> Self {
        Self::new(shape.intrinsic_dimension(), 3, shape)
    }

    pub const fn intrinsic_dimension(self) -> u32 {
        self.intrinsic_dimension
    }

    pub const fn embedding_dimension(self) -> u32 {
        self.embedding_dimension
    }

    pub const fn shape(self) -> DomainShape {
        self.shape
    }

    pub fn ir_type(self) -> Type {
        Type::dialect(
            DOMAIN_REGION_TYPE,
            vec![
                TypeParameter::Integer(i64::from(self.intrinsic_dimension)),
                TypeParameter::Integer(i64::from(self.embedding_dimension)),
                TypeParameter::String(Arc::from(self.shape.as_str())),
            ],
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoundaryType {
    dimension: u32,
    embedding_dimension: u32,
}

impl BoundaryType {
    pub const fn new(dimension: u32, embedding_dimension: u32) -> Self {
        Self {
            dimension,
            embedding_dimension,
        }
    }

    pub fn ir_type(self) -> Type {
        Type::dialect(
            DOMAIN_BOUNDARY_TYPE,
            vec![
                TypeParameter::Integer(i64::from(self.dimension)),
                TypeParameter::Integer(i64::from(self.embedding_dimension)),
            ],
        )
    }
}
