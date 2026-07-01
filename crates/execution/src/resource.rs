// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::BufferSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(u32);

impl ResourceId {
    pub const fn index(self) -> u32 {
        self.0
    }

    pub(crate) fn from_index(index: usize) -> Self {
        Self(u32::try_from(index).expect("execution resource ID overflow"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemorySpace {
    Host,
    CudaDevice { ordinal: u32 },
    Unified,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScalarValue {
    Bool(bool),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl ScalarValue {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::F64(value) => Some(*value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferProvision {
    /// The runtime allocates this buffer from the memory plan.
    Runtime,

    /// The caller must bind an existing allocation before execution.
    External,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferDeclaration {
    spec: BufferSpec,
    provision: BufferProvision,
}

impl BufferDeclaration {
    pub fn runtime(spec: BufferSpec) -> Self {
        Self {
            spec,
            provision: BufferProvision::Runtime,
        }
    }

    pub fn external(spec: BufferSpec) -> Self {
        Self {
            spec,
            provision: BufferProvision::External,
        }
    }

    pub fn spec(&self) -> &BufferSpec {
        &self.spec
    }

    pub const fn provision(&self) -> BufferProvision {
        self.provision
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResourceDeclaration {
    Buffer(BufferDeclaration),
    Scalar(ScalarValue),
}

impl ResourceDeclaration {
    pub fn buffer(&self) -> Option<&BufferDeclaration> {
        match self {
            Self::Buffer(buffer) => Some(buffer),
            Self::Scalar(_) => None,
        }
    }

    pub fn scalar(&self) -> Option<&ScalarValue> {
        match self {
            Self::Scalar(value) => Some(value),
            Self::Buffer(_) => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResourceTable {
    declarations: Vec<ResourceDeclaration>,
}

impl ResourceTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declare(&mut self, declaration: ResourceDeclaration) -> ResourceId {
        let id = ResourceId::from_index(self.declarations.len());
        self.declarations.push(declaration);
        id
    }

    pub fn declare_scalar(&mut self, value: ScalarValue) -> ResourceId {
        self.declare(ResourceDeclaration::Scalar(value))
    }

    pub fn declare_buffer(&mut self, spec: BufferSpec) -> ResourceId {
        self.declare(ResourceDeclaration::Buffer(BufferDeclaration::runtime(
            spec,
        )))
    }

    pub fn declare_external_buffer(&mut self, spec: BufferSpec) -> ResourceId {
        self.declare(ResourceDeclaration::Buffer(BufferDeclaration::external(
            spec,
        )))
    }

    pub fn get(&self, id: ResourceId) -> Option<&ResourceDeclaration> {
        self.declarations.get(id.0 as usize)
    }

    pub fn buffer(&self, id: ResourceId) -> Option<&BufferDeclaration> {
        self.get(id).and_then(ResourceDeclaration::buffer)
    }

    pub fn scalar(&self, id: ResourceId) -> Option<&ScalarValue> {
        self.get(id).and_then(ResourceDeclaration::scalar)
    }

    pub fn len(&self) -> usize {
        self.declarations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (ResourceId, &ResourceDeclaration)> {
        self.declarations
            .iter()
            .enumerate()
            .map(|(index, declaration)| (ResourceId::from_index(index), declaration))
    }
}
