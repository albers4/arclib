// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use ir::Module;

use super::Analysis;

pub struct AnalysisManager {
    cache: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl AnalysisManager {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get<A>(&mut self, module: &Module) -> &A
    where
        A: Analysis,
    {
        let id = TypeId::of::<A>();

        self.cache
            .entry(id)
            .or_insert_with(|| Box::new(A::run(module)));

        self.cache.get(&id).unwrap().downcast_ref::<A>().unwrap()
    }

    pub fn invalidate<A>(&mut self)
    where
        A: Analysis,
    {
        self.cache.remove(&TypeId::of::<A>());
    }

    pub fn invalidate_all(&mut self) {
        self.cache.clear();
    }
}

impl Default for AnalysisManager {
    fn default() -> Self {
        Self::new()
    }
}
