// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use ir::{Type, TypeParameter};

pub const FVM_BUFFER_TYPE: &str = "fvm.buffer";
pub const FVM_MESH_VIEW_TYPE: &str = "fvm.mesh_view";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FvmBufferElement {
    I32,
    F64,
}

impl FvmBufferElement {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::I32 => "i32",
            Self::F64 => "f64",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FvmBufferType {
    element: FvmBufferElement,
}

impl FvmBufferType {
    pub const fn new(element: FvmBufferElement) -> Self {
        Self { element }
    }

    pub fn ir_type(self) -> Type {
        Type::dialect(
            FVM_BUFFER_TYPE,
            vec![TypeParameter::String(Arc::from(self.element.as_str()))],
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FvmMeshViewType {
    dimension: u32,
}

impl FvmMeshViewType {
    pub const fn new(dimension: u32) -> Self {
        Self { dimension }
    }

    pub fn ir_type(self) -> Type {
        Type::dialect(
            FVM_MESH_VIEW_TYPE,
            vec![TypeParameter::Integer(i64::from(self.dimension))],
        )
    }
}
