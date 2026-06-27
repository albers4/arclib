// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use ir::{OperationName, SymbolRef};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstraintScope {
    Global,

    Dialect(Arc<str>),

    Operation(OperationName),

    Symbol(SymbolRef),

    Tag(Arc<str>),
}

impl ConstraintScope {
    pub fn dialect(name: impl AsRef<str>) -> Self {
        Self::Dialect(Arc::from(name.as_ref()))
    }

    pub fn operation(name: impl AsRef<str>) -> Self {
        Self::Operation(OperationName::new(name))
    }

    pub fn symbol(reference: SymbolRef) -> Self {
        Self::Symbol(reference)
    }

    pub fn tag(name: impl AsRef<str>) -> Self {
        Self::Tag(Arc::from(name.as_ref()))
    }
}
