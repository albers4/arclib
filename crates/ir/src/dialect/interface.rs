// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use crate::OperationRef;

#[derive(Default)]
pub struct InterfaceMap {
    interfaces: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl InterfaceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<I>(&mut self, interface: I)
    where
        I: Any + Send + Sync,
    {
        self.interfaces
            .insert(TypeId::of::<I>(), Arc::new(interface));
    }

    pub fn get<I>(&self) -> Option<&I>
    where
        I: Any + Send + Sync,
    {
        self.interfaces
            .get(&TypeId::of::<I>())
            .and_then(|interface| interface.as_ref().downcast_ref::<I>())
    }

    pub fn contains<I>(&self) -> bool
    where
        I: Any + Send + Sync,
    {
        self.interfaces.contains_key(&TypeId::of::<I>())
    }
}

pub trait OperationVerifier: Send + Sync {
    fn verify(&self, operation: OperationRef<'_>) -> Result<(), String>;
}

impl<F> OperationVerifier for F
where
    F: for<'a> Fn(OperationRef<'a>) -> Result<(), String> + Send + Sync,
{
    fn verify(&self, operation: OperationRef<'_>) -> Result<(), String> {
        self(operation)
    }
}

pub struct VerifyInterface {
    verifier: Arc<dyn OperationVerifier>,
}

impl VerifyInterface {
    pub fn new<V>(verifier: V) -> Self
    where
        V: OperationVerifier + 'static,
    {
        Self {
            verifier: Arc::new(verifier),
        }
    }

    pub fn verify(&self, operation: OperationRef<'_>) -> Result<(), String> {
        self.verifier.verify(operation)
    }
}
