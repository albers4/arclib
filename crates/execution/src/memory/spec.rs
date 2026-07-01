// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::MemorySpace;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BufferSpec {
    bytes: usize,
    alignment: usize,
    memory_space: MemorySpace,
}

impl BufferSpec {
    pub fn new(bytes: usize, memory_space: MemorySpace) -> Self {
        Self {
            bytes,
            alignment: 64,
            memory_space,
        }
    }

    pub fn with_alignment(mut self, alignment: usize) -> Self {
        self.alignment = alignment;
        self
    }

    pub const fn bytes(&self) -> usize {
        self.bytes
    }

    pub const fn alignment(&self) -> usize {
        self.alignment
    }

    pub const fn memory_space(&self) -> MemorySpace {
        self.memory_space
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.bytes == 0 {
            return Err("buffer size must be greater than zero".into());
        }
        if self.alignment == 0 || !self.alignment.is_power_of_two() {
            return Err("buffer alignment must be a nonzero power of two".into());
        }
        Ok(())
    }
}
