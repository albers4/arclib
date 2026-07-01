// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KernelValueKind {
    Buffer,
    Scalar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KernelAccess {
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelParameter {
    name: Arc<str>,
    kind: KernelValueKind,
    access: KernelAccess,
}

impl KernelParameter {
    pub fn new(name: impl AsRef<str>, kind: KernelValueKind, access: KernelAccess) -> Self {
        Self {
            name: Arc::from(name.as_ref()),

            kind,
            access,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> KernelValueKind {
        self.kind
    }

    pub fn access(&self) -> KernelAccess {
        self.access
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KernelAbi {
    parameters: Vec<KernelParameter>,
    result_aliases: Vec<usize>,
}

impl KernelAbi {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parameter(mut self, parameter: KernelParameter) -> Self {
        self.parameters.push(parameter);

        self
    }

    /// Declares that one SSA result of
    /// `kernel.call` aliases an ABI argument.
    ///
    /// Results are added in declaration order.
    pub fn result_alias(mut self, parameter_index: usize) -> Self {
        self.result_aliases.push(parameter_index);

        self
    }

    pub fn parameters(&self) -> &[KernelParameter] {
        &self.parameters
    }

    pub fn result_aliases(&self) -> &[usize] {
        &self.result_aliases
    }
}
