// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{any::TypeId, collections::HashSet};

use crate::{OperationName, OperationRef};

use super::{InterfaceMap, VerifyInterface};

pub struct OperationDescriptor {
    name: OperationName,

    traits: HashSet<TypeId>,
    interfaces: InterfaceMap,
}

impl OperationDescriptor {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: OperationName::new(name),
            traits: HashSet::new(),
            interfaces: InterfaceMap::new(),
        }
    }

    pub fn name(&self) -> &OperationName {
        &self.name
    }

    pub fn with_trait<T>(mut self) -> Self
    where
        T: 'static,
    {
        self.traits.insert(TypeId::of::<T>());
        self
    }

    pub fn has_trait<T>(&self) -> bool
    where
        T: 'static,
    {
        self.traits.contains(&TypeId::of::<T>())
    }

    pub fn with_interface<I>(mut self, interface: I) -> Self
    where
        I: Send + Sync + 'static,
    {
        self.interfaces.insert(interface);
        self
    }

    pub fn interface<I>(&self) -> Option<&I>
    where
        I: Send + Sync + 'static,
    {
        self.interfaces.get::<I>()
    }

    pub fn has_interface<I>(&self) -> bool
    where
        I: Send + Sync + 'static,
    {
        self.interfaces.contains::<I>()
    }

    pub fn with_verifier<V>(self, verifier: V) -> Self
    where
        V: super::OperationVerifier + 'static,
    {
        self.with_interface(VerifyInterface::new(verifier))
    }

    pub fn verify(&self, operation: OperationRef<'_>) -> Result<(), String> {
        let Some(verifier) = self.interface::<VerifyInterface>() else {
            return Ok(());
        };

        verifier.verify(operation)
    }
}
