// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Signedness {
    Signless,
    Signed,
    Unsigned,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeParameter {
    Type(Box<Type>),
    Integer(i64),
    String(Arc<str>),
    Shape(Vec<Option<usize>>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Unit,
    Index,

    Integer {
        width: u16,
        signedness: Signedness,
    },

    Float {
        width: u16,
    },

    Dialect {
        name: Arc<str>,
        parameters: Vec<TypeParameter>,
    },
}

impl Type {
    pub fn f4() -> Self {
        Self::Float { width: 4 }
    }

    pub fn f8() -> Self {
        Self::Float { width: 8 }
    }

    pub fn f16() -> Self {
        Self::Float { width: 16 }
    }

    pub fn f32() -> Self {
        Self::Float { width: 32 }
    }

    pub fn f64() -> Self {
        Self::Float { width: 64 }
    }

    pub fn integer(width: u16, signedness: Signedness) -> Self {
        Self::Integer { width, signedness }
    }

    pub fn dialect(name: impl AsRef<str>, parameters: impl Into<Vec<TypeParameter>>) -> Self {
        Self::Dialect {
            name: Arc::from(name.as_ref()),
            parameters: parameters.into(),
        }
    }

    pub fn vector(length: usize, element: Type) -> Self {
        Self::dialect(
            "builtin.vector",
            vec![
                TypeParameter::Shape(vec![Some(length)]),
                TypeParameter::Type(Box::new(element)),
            ],
        )
    }

    pub fn tensor(shape: Vec<Option<usize>>, element: Type) -> Self {
        Self::dialect(
            "builtin.tensor",
            vec![
                TypeParameter::Shape(shape),
                TypeParameter::Type(Box::new(element)),
            ],
        )
    }
}
