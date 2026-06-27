// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(Arc<str>),
    Array(Vec<PolicyValue>),
}

impl PolicyValue {
    pub fn string(value: impl AsRef<str>) -> Self {
        Self::String(Arc::from(value.as_ref()))
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::Integer(_) | Self::Float(_))
    }

    pub(crate) fn validate(&self) -> bool {
        match self {
            Self::Float(value) => value.is_finite(),

            Self::Array(values) => values.iter().all(Self::validate),

            _ => true,
        }
    }
}

impl From<bool> for PolicyValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for PolicyValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for PolicyValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<&str> for PolicyValue {
    fn from(value: &str) -> Self {
        Self::string(value)
    }
}

impl From<String> for PolicyValue {
    fn from(value: String) -> Self {
        Self::String(Arc::from(value))
    }
}

impl From<Vec<PolicyValue>> for PolicyValue {
    fn from(value: Vec<PolicyValue>) -> Self {
        Self::Array(value)
    }
}
