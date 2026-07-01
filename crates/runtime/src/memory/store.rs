// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{collections::HashMap, ffi::c_void};

use execution::{
    BufferSpec, MemorySpace, ResourceDeclaration, ResourceId, ResourceTable, ScalarValue,
};
use kernel::{KernelBackend, KernelValueKind};

use crate::{BufferBinding, RuntimeError};

#[derive(Debug, Clone)]
pub enum RuntimeResource {
    Buffer(BufferBinding),
    Scalar(ScalarValue),
}

impl RuntimeResource {
    pub const fn kind(&self) -> KernelValueKind {
        match self {
            Self::Buffer(_) => KernelValueKind::Buffer,
            Self::Scalar(_) => KernelValueKind::Scalar,
        }
    }

    pub fn as_buffer(&self) -> Option<&BufferBinding> {
        match self {
            Self::Buffer(buffer) => Some(buffer),
            Self::Scalar(_) => None,
        }
    }

    pub fn as_scalar(&self) -> Option<&ScalarValue> {
        match self {
            Self::Scalar(value) => Some(value),
            Self::Buffer(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceStore {
    declarations: ResourceTable,
    values: HashMap<ResourceId, RuntimeResource>,
}

impl ResourceStore {
    pub fn from_declarations(declarations: ResourceTable) -> Self {
        let values = declarations
            .iter()
            .filter_map(|(id, declaration)| match declaration {
                ResourceDeclaration::Scalar(value) => {
                    Some((id, RuntimeResource::Scalar(value.clone())))
                }
                ResourceDeclaration::Buffer(_) => None,
            })
            .collect();

        Self {
            declarations,
            values,
        }
    }

    pub fn declarations(&self) -> &ResourceTable {
        &self.declarations
    }

    pub fn get(&self, id: ResourceId) -> Option<&RuntimeResource> {
        self.values.get(&id)
    }

    pub fn get_mut(&mut self, id: ResourceId) -> Option<&mut RuntimeResource> {
        self.values.get_mut(&id)
    }

    pub fn buffer(&self, id: ResourceId) -> Result<&BufferBinding, RuntimeError> {
        let declaration = self
            .declarations
            .get(id)
            .ok_or(RuntimeError::MissingResource(id))?;
        if !matches!(declaration, ResourceDeclaration::Buffer(_)) {
            return Err(RuntimeError::ResourceKindMismatch {
                resource: id,
                expected: KernelValueKind::Buffer,
                actual: KernelValueKind::Scalar,
            });
        }

        match self.values.get(&id) {
            Some(RuntimeResource::Buffer(buffer)) => Ok(buffer),
            Some(RuntimeResource::Scalar(_)) => unreachable!("declaration and value disagree"),
            None => Err(RuntimeError::UnmaterializedBuffer(id)),
        }
    }

    pub fn scalar(&self, id: ResourceId) -> Result<&ScalarValue, RuntimeError> {
        match self
            .declarations
            .get(id)
            .ok_or(RuntimeError::MissingResource(id))?
        {
            ResourceDeclaration::Scalar(_) => {}
            ResourceDeclaration::Buffer(_) => {
                return Err(RuntimeError::ResourceKindMismatch {
                    resource: id,
                    expected: KernelValueKind::Scalar,
                    actual: KernelValueKind::Buffer,
                });
            }
        }

        match self.values.get(&id) {
            Some(RuntimeResource::Scalar(value)) => Ok(value),
            Some(RuntimeResource::Buffer(_)) => unreachable!("declaration and value disagree"),
            None => unreachable!("scalar declarations are initialized"),
        }
    }

    pub fn scalar_mut(&mut self, id: ResourceId) -> Result<&mut ScalarValue, RuntimeError> {
        match self
            .declarations
            .get(id)
            .ok_or(RuntimeError::MissingResource(id))?
        {
            ResourceDeclaration::Scalar(_) => {}
            ResourceDeclaration::Buffer(_) => {
                return Err(RuntimeError::ResourceKindMismatch {
                    resource: id,
                    expected: KernelValueKind::Scalar,
                    actual: KernelValueKind::Buffer,
                });
            }
        }

        match self.values.get_mut(&id) {
            Some(RuntimeResource::Scalar(value)) => Ok(value),
            Some(RuntimeResource::Buffer(_)) => unreachable!("declaration and value disagree"),
            None => unreachable!("scalar declarations are initialized"),
        }
    }

    pub fn bind_buffer(
        &mut self,
        id: ResourceId,
        binding: BufferBinding,
    ) -> Result<(), RuntimeError> {
        if self.values.contains_key(&id) {
            return Err(RuntimeError::ResourceAlreadyBound(id));
        }
        let spec = self.buffer_spec(id)?;
        validate_binding(id, spec, &binding)?;
        self.values.insert(id, RuntimeResource::Buffer(binding));
        Ok(())
    }

    pub(crate) fn buffer_spec(&self, id: ResourceId) -> Result<&BufferSpec, RuntimeError> {
        match self
            .declarations
            .get(id)
            .ok_or(RuntimeError::MissingResource(id))?
        {
            ResourceDeclaration::Buffer(buffer) => Ok(buffer.spec()),
            ResourceDeclaration::Scalar(_) => Err(RuntimeError::ResourceKindMismatch {
                resource: id,
                expected: KernelValueKind::Buffer,
                actual: KernelValueKind::Scalar,
            }),
        }
    }

    pub(crate) fn ensure_unbound(&self, id: ResourceId) -> Result<(), RuntimeError> {
        self.buffer_spec(id)?;
        if self.values.contains_key(&id) {
            return Err(RuntimeError::ResourceAlreadyBound(id));
        }
        Ok(())
    }

    pub(crate) fn commit_buffer(&mut self, id: ResourceId, binding: BufferBinding) {
        let previous = self.values.insert(id, RuntimeResource::Buffer(binding));
        debug_assert!(previous.is_none());
    }

    pub(crate) fn packed_pointer(
        &mut self,
        id: ResourceId,
        expected: KernelValueKind,
        backend: KernelBackend,
        cuda_device: Option<u32>,
    ) -> Result<*mut c_void, RuntimeError> {
        let declaration = self
            .declarations
            .get(id)
            .ok_or(RuntimeError::MissingResource(id))?;
        let actual = match declaration {
            ResourceDeclaration::Buffer(_) => KernelValueKind::Buffer,
            ResourceDeclaration::Scalar(_) => KernelValueKind::Scalar,
        };
        if actual != expected {
            return Err(RuntimeError::ResourceKindMismatch {
                resource: id,
                expected,
                actual,
            });
        }

        match self.values.get_mut(&id) {
            Some(RuntimeResource::Scalar(value)) => Ok(scalar_pointer(value)),
            Some(RuntimeResource::Buffer(buffer)) => {
                validate_memory_space(backend, buffer.memory_space(), cuda_device)?;
                Ok(buffer.pointer().as_ptr())
            }
            None => Err(RuntimeError::UnmaterializedBuffer(id)),
        }
    }
}

pub(crate) fn validate_binding(
    resource: ResourceId,
    spec: &BufferSpec,
    binding: &BufferBinding,
) -> Result<(), RuntimeError> {
    if binding.bytes() < spec.bytes() {
        return Err(RuntimeError::BufferTooSmall {
            resource,
            required: spec.bytes(),
            actual: binding.bytes(),
        });
    }
    if binding.memory_space() != spec.memory_space() {
        return Err(RuntimeError::BufferSpaceMismatch {
            resource,
            expected: spec.memory_space(),
            actual: binding.memory_space(),
        });
    }
    if !binding.is_aligned_to(spec.alignment()) {
        return Err(RuntimeError::BufferMisaligned {
            resource,
            required: spec.alignment(),
            address: binding.pointer().as_ptr() as usize,
        });
    }
    Ok(())
}

fn scalar_pointer(value: &mut ScalarValue) -> *mut c_void {
    match value {
        ScalarValue::Bool(value) => (value as *mut bool).cast(),
        ScalarValue::I32(value) => (value as *mut i32).cast(),
        ScalarValue::I64(value) => (value as *mut i64).cast(),
        ScalarValue::F32(value) => (value as *mut f32).cast(),
        ScalarValue::F64(value) => (value as *mut f64).cast(),
    }
}

fn validate_memory_space(
    backend: KernelBackend,
    memory_space: MemorySpace,
    cuda_device: Option<u32>,
) -> Result<(), RuntimeError> {
    match backend {
        KernelBackend::Cpu => {
            if matches!(memory_space, MemorySpace::Host | MemorySpace::Unified) {
                Ok(())
            } else {
                Err(RuntimeError::UnsupportedMemorySpace {
                    backend,
                    memory_space,
                })
            }
        }
        KernelBackend::Cuda => match memory_space {
            MemorySpace::Unified => Ok(()),
            MemorySpace::CudaDevice { ordinal } => {
                let active = cuda_device.ok_or(RuntimeError::MissingCudaDevice)?;
                if active == ordinal {
                    Ok(())
                } else {
                    Err(RuntimeError::CudaDeviceMismatch {
                        active,
                        buffer: ordinal,
                    })
                }
            }
            MemorySpace::Host => Err(RuntimeError::UnsupportedMemorySpace {
                backend,
                memory_space,
            }),
        },
    }
}
