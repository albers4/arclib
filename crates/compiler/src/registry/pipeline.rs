// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{collections::HashMap, sync::Arc};

use crate::pass::PassManager;

use super::{PassRegistry, RegistryError, is_valid_registry_name};

#[derive(Debug, Clone)]
pub struct PipelineDescriptor {
    name: Arc<str>,
    passes: Vec<Arc<str>>,
}

impl PipelineDescriptor {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: Arc::from(name.as_ref()),
            passes: Vec::new(),
        }
    }

    pub fn pass(mut self, pass: impl AsRef<str>) -> Self {
        self.passes.push(Arc::from(pass.as_ref()));

        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn passes(&self) -> &[Arc<str>] {
        &self.passes
    }
}

pub struct PipelineRegistry {
    pipelines: HashMap<Arc<str>, PipelineDescriptor>,
}

impl PipelineRegistry {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
        }
    }

    pub fn register(&mut self, pipeline: PipelineDescriptor) -> Result<(), RegistryError> {
        if !is_valid_registry_name(pipeline.name()) {
            return Err(RegistryError::InvalidName {
                kind: "pipeline",
                name: pipeline.name().to_owned(),
            });
        }

        if self.pipelines.contains_key(pipeline.name()) {
            return Err(RegistryError::DuplicatePipeline(pipeline.name().to_owned()));
        }

        self.pipelines.insert(Arc::from(pipeline.name()), pipeline);

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&PipelineDescriptor> {
        self.pipelines.get(name)
    }

    pub fn build(&self, name: &str, passes: &PassRegistry) -> Result<PassManager, RegistryError> {
        let descriptor = self
            .get(name)
            .ok_or_else(|| RegistryError::MissingPipeline(name.to_owned()))?;

        let mut manager = PassManager::new();

        for pass_name in descriptor.passes() {
            manager.add_boxed_pass(passes.create(pass_name)?);
        }

        Ok(manager)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.pipelines.keys().map(AsRef::as_ref)
    }
}

impl Default for PipelineRegistry {
    fn default() -> Self {
        Self::new()
    }
}
