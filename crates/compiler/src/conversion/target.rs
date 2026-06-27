// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{collections::HashMap, sync::Arc};

use ir::{OperationId, OperationName, OperationRef};

use super::ConversionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Legality {
    Legal,
    Illegal,
    Unknown,
}

type DynamicLegality = Arc<dyn for<'a> Fn(OperationRef<'a>) -> Result<bool, String> + Send + Sync>;

enum LegalityRule {
    Legal,
    Illegal,
    Dynamic(DynamicLegality),
}

pub struct ConversionTarget {
    dialects: HashMap<Arc<str>, LegalityRule>,

    operations: HashMap<OperationName, LegalityRule>,
}

impl ConversionTarget {
    pub fn new() -> Self {
        Self {
            dialects: HashMap::new(),
            operations: HashMap::new(),
        }
    }

    pub fn mark_dialect_legal(&mut self, dialect: impl AsRef<str>) -> &mut Self {
        self.dialects
            .insert(Arc::from(dialect.as_ref()), LegalityRule::Legal);

        self
    }

    pub fn mark_dialect_illegal(&mut self, dialect: impl AsRef<str>) -> &mut Self {
        self.dialects
            .insert(Arc::from(dialect.as_ref()), LegalityRule::Illegal);

        self
    }

    pub fn mark_dialect_dynamically_legal<F>(
        &mut self,
        dialect: impl AsRef<str>,
        legality: F,
    ) -> &mut Self
    where
        F: for<'a> Fn(OperationRef<'a>) -> Result<bool, String> + Send + Sync + 'static,
    {
        self.dialects.insert(
            Arc::from(dialect.as_ref()),
            LegalityRule::Dynamic(Arc::new(legality)),
        );

        self
    }

    pub fn mark_operation_legal(&mut self, operation: impl AsRef<str>) -> &mut Self {
        self.operations
            .insert(OperationName::new(operation), LegalityRule::Legal);

        self
    }

    pub fn mark_operation_illegal(&mut self, operation: impl AsRef<str>) -> &mut Self {
        self.operations
            .insert(OperationName::new(operation), LegalityRule::Illegal);

        self
    }

    pub fn mark_operation_dynamically_legal<F>(
        &mut self,
        operation: impl AsRef<str>,
        legality: F,
    ) -> &mut Self
    where
        F: for<'a> Fn(OperationRef<'a>) -> Result<bool, String> + Send + Sync + 'static,
    {
        self.operations.insert(
            OperationName::new(operation),
            LegalityRule::Dynamic(Arc::new(legality)),
        );

        self
    }

    pub fn classify(&self, operation: OperationRef<'_>) -> Result<Legality, ConversionError> {
        let operation_id = operation.id();
        let name = operation.name().clone();

        if let Some(rule) = self.operations.get(&name) {
            return evaluate_rule(rule, operation, operation_id, name);
        }

        let Some(dialect) = name.dialect() else {
            return Ok(Legality::Unknown);
        };

        let Some(rule) = self.dialects.get(dialect) else {
            return Ok(Legality::Unknown);
        };

        evaluate_rule(rule, operation, operation_id, name)
    }
}

impl Default for ConversionTarget {
    fn default() -> Self {
        Self::new()
    }
}

fn evaluate_rule(
    rule: &LegalityRule,
    operation: OperationRef<'_>,
    operation_id: OperationId,
    name: OperationName,
) -> Result<Legality, ConversionError> {
    match rule {
        LegalityRule::Legal => Ok(Legality::Legal),

        LegalityRule::Illegal => Ok(Legality::Illegal),

        LegalityRule::Dynamic(callback) => callback(operation)
            .map(|legal| {
                if legal {
                    Legality::Legal
                } else {
                    Legality::Illegal
                }
            })
            .map_err(|message| ConversionError::DynamicLegalityFailed {
                operation: operation_id,
                name,
                message,
            }),
    }
}
