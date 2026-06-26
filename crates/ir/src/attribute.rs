// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::SymbolRef;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone, PartialEq)]
pub enum Attribute {
    Unit,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(Arc<str>),
    SymbolRef(SymbolRef),
    Array(Vec<Attribute>),
    Dictionary(Box<AttributeMap>),
}

impl Attribute {
    pub fn string(value: impl AsRef<str>) -> Self {
        Self::String(Arc::from(value.as_ref()))
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn symbol_ref(reference: SymbolRef) -> Self {
        Self::SymbolRef(reference)
    }

    pub fn as_symbol_ref(&self) -> Option<&SymbolRef> {
        match self {
            Self::SymbolRef(reference) => Some(reference),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AttributeMap {
    values: BTreeMap<Arc<str>, Attribute>,
}

impl AttributeMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl AsRef<str>, attribute: Attribute) -> Option<Attribute> {
        self.values.insert(Arc::from(name.as_ref()), attribute)
    }

    pub fn get(&self, name: &str) -> Option<&Attribute> {
        self.values.get(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<Attribute> {
        self.values.remove(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Attribute)> {
        self.values
            .iter()
            .map(|(name, attribute)| (name.as_ref(), attribute))
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}
