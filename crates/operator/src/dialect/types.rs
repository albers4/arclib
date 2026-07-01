// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use ir::{Type, TypeParameter};

pub const OPERATOR_EXPRESSION_TYPE: &str = "operator.expression";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionType {
    element: Arc<str>,
    dimensions: Vec<u32>,
}

impl ExpressionType {
    pub fn new(element: impl AsRef<str>, dimensions: impl IntoIterator<Item = u32>) -> Self {
        Self {
            element: Arc::from(element.as_ref()),
            dimensions: dimensions.into_iter().collect(),
        }
    }

    pub fn scalar_f64() -> Self {
        Self::new("f64", [])
    }

    pub fn vector_f64(components: u32) -> Self {
        Self::new("f64", [components])
    }

    pub fn tensor_f64(dimensions: impl IntoIterator<Item = u32>) -> Self {
        Self::new("f64", dimensions)
    }

    pub fn element(&self) -> &str {
        &self.element
    }

    pub fn dimensions(&self) -> &[u32] {
        &self.dimensions
    }

    pub fn rank(&self) -> usize {
        self.dimensions.len()
    }

    pub fn ir_type(&self) -> Type {
        let mut parameters = Vec::with_capacity(self.dimensions.len() + 2);
        parameters.push(TypeParameter::String(self.element.clone()));
        parameters.push(TypeParameter::Integer(
            i64::try_from(self.dimensions.len()).expect("operator rank exceeds i64"),
        ));
        parameters.extend(
            self.dimensions
                .iter()
                .copied()
                .map(|dimension| TypeParameter::Integer(i64::from(dimension))),
        );
        Type::dialect(OPERATOR_EXPRESSION_TYPE, parameters)
    }
}
