// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{collections::HashMap, sync::Arc};

use crate::pass::Pass;

use super::RegistryError;

type PassFactory = Arc<dyn Fn() -> Box<dyn Pass> + Send + Sync>;

pub struct PassRegistry {
    factories: HashMap<Arc<str>, PassFactory>,
}

impl PassRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    pub fn register<P, F>(&mut self, name: impl AsRef<str>, factory: F) -> Result<(), RegistryError>
    where
        P: Pass + 'static,
        F: Fn() -> P + Send + Sync + 'static,
    {
        let name = name.as_ref();

        if self.factories.contains_key(name) {
            return Err(RegistryError::DuplicatePass(name.to_owned()));
        }

        let factory: PassFactory = Arc::new(move || Box::new(factory()));

        self.factories.insert(name.to_owned().into(), factory);

        Ok(())
    }

    pub fn register_boxed<F>(
        &mut self,
        name: impl AsRef<str>,
        factory: F,
    ) -> Result<(), RegistryError>
    where
        F: Fn() -> Box<dyn Pass> + Send + Sync + 'static,
    {
        let name = name.as_ref();

        if self.factories.contains_key(name) {
            return Err(RegistryError::DuplicatePass(name.to_owned()));
        }

        self.factories
            .insert(name.to_owned().into(), Arc::new(factory));

        Ok(())
    }

    pub fn create(&self, name: &str) -> Result<Box<dyn Pass>, RegistryError> {
        let factory = self
            .factories
            .get(name)
            .ok_or_else(|| RegistryError::MissingPass(name.to_owned()))?;

        Ok(factory())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.factories.contains_key(name)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.factories.keys().map(AsRef::as_ref)
    }
}

impl Default for PassRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn is_valid_registry_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    name.chars().all(|character| {
        character.is_ascii_alphanumeric()
            || character == '_'
            || character == '-'
            || character == '.'
    })
}
