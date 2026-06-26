// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{collections::HashMap, sync::Arc};

use crate::{OperationName, OperationRef};

use super::{DialectDescriptor, DialectRegistryError, OperationDescriptor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownOperationPolicy {
    Reject,
    Allow,
}

pub struct DialectRegistry {
    dialects: HashMap<Arc<str>, DialectDescriptor>,
}

impl DialectRegistry {
    pub fn new() -> Self {
        Self {
            dialects: HashMap::new(),
        }
    }

    pub fn register_dialect(
        &mut self,
        dialect: DialectDescriptor,
    ) -> Result<(), DialectRegistryError> {
        let namespace = dialect.namespace().to_owned();

        if !is_valid_namespace(&namespace) {
            return Err(DialectRegistryError::InvalidDialectName(namespace));
        }

        if self.dialects.contains_key(namespace.as_str()) {
            return Err(DialectRegistryError::DuplicateDialect(namespace));
        }

        self.dialects.insert(Arc::from(namespace.as_str()), dialect);

        Ok(())
    }

    pub fn register_operation(
        &mut self,
        descriptor: OperationDescriptor,
    ) -> Result<(), DialectRegistryError> {
        if !is_valid_operation_name(descriptor.name()) {
            return Err(DialectRegistryError::InvalidOperationName(
                descriptor.name().clone(),
            ));
        }

        let namespace = descriptor
            .name()
            .dialect()
            .expect("validated operation has dialect")
            .to_owned();

        let dialect = self
            .dialects
            .get_mut(namespace.as_str())
            .ok_or(DialectRegistryError::MissingDialect(namespace))?;

        dialect.register_operation(descriptor)
    }

    pub fn dialect(&self, namespace: &str) -> Option<&DialectDescriptor> {
        self.dialects.get(namespace)
    }

    pub fn operation(&self, name: &OperationName) -> Option<&OperationDescriptor> {
        let namespace = name.dialect()?;

        self.dialect(namespace)?.operation(name)
    }

    pub fn verify_operation(
        &self,
        operation: OperationRef<'_>,
        unknown_policy: UnknownOperationPolicy,
    ) -> Result<(), DialectRegistryError> {
        let operation_id = operation.id();
        let operation_name = operation.name().clone();

        if !is_valid_operation_name(&operation_name) {
            return Err(DialectRegistryError::InvalidOperationName(operation_name));
        }

        let namespace = operation_name
            .dialect()
            .expect("operation name was validated");

        let Some(dialect) = self.dialect(namespace) else {
            return match unknown_policy {
                UnknownOperationPolicy::Allow => Ok(()),

                UnknownOperationPolicy::Reject => Err(DialectRegistryError::UnknownDialect {
                    operation: operation_id,
                    dialect: namespace.to_owned(),
                }),
            };
        };

        let Some(descriptor) = dialect.operation(&operation_name) else {
            if unknown_policy == UnknownOperationPolicy::Allow
                || dialect.allows_unknown_operations()
            {
                return Ok(());
            }

            return Err(DialectRegistryError::UnknownOperation {
                operation: operation_id,
                name: operation_name,
            });
        };

        descriptor.verify(operation).map_err(|message| {
            DialectRegistryError::OperationVerificationFailed {
                operation: operation_id,
                name: operation_name,
                message,
            }
        })
    }
}

impl Default for DialectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn is_valid_namespace(namespace: &str) -> bool {
    is_valid_component(namespace)
}

fn is_valid_operation_name(name: &OperationName) -> bool {
    let mut components = name.as_str().split('.');

    let Some(namespace) = components.next() else {
        return false;
    };

    if !is_valid_namespace(namespace) {
        return false;
    }

    let mut has_operation = false;

    for component in components {
        has_operation = true;

        if !is_valid_component(component) {
            return false;
        }
    }

    has_operation
}

fn is_valid_component(component: &str) -> bool {
    let mut characters = component.chars();

    let Some(first) = characters.next() else {
        return false;
    };

    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }

    characters
        .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '-')
}
