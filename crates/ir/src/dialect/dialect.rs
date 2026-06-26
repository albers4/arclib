// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{collections::HashMap, sync::Arc};

use crate::OperationName;

use super::{DialectRegistryError, OperationDescriptor};

pub struct DialectDescriptor {
    namespace: Arc<str>,

    allow_unknown_operations: bool,

    operations: HashMap<OperationName, Arc<OperationDescriptor>>,
}

impl DialectDescriptor {
    pub fn new(namespace: impl AsRef<str>) -> Self {
        Self {
            namespace: Arc::from(namespace.as_ref()),

            allow_unknown_operations: false,

            operations: HashMap::new(),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn allow_unknown_operations(mut self) -> Self {
        self.allow_unknown_operations = true;
        self
    }

    pub fn allows_unknown_operations(&self) -> bool {
        self.allow_unknown_operations
    }

    pub fn register_operation(
        &mut self,
        descriptor: OperationDescriptor,
    ) -> Result<(), DialectRegistryError> {
        let operation_dialect = descriptor.name().dialect();

        if operation_dialect != Some(self.namespace()) {
            return Err(DialectRegistryError::OperationDialectMismatch {
                dialect: self.namespace().to_owned(),
                operation: descriptor.name().clone(),
            });
        }

        if self.operations.contains_key(descriptor.name()) {
            return Err(DialectRegistryError::DuplicateOperation(
                descriptor.name().clone(),
            ));
        }

        self.operations
            .insert(descriptor.name().clone(), Arc::new(descriptor));

        Ok(())
    }

    pub fn operation(&self, name: &OperationName) -> Option<&OperationDescriptor> {
        self.operations.get(name).map(Arc::as_ref)
    }

    pub fn operations(&self) -> impl Iterator<Item = &OperationDescriptor> {
        self.operations.values().map(Arc::as_ref)
    }
}
