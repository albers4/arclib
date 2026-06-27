// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{collections::HashSet, sync::Arc};

use ir::DialectRegistry;

use crate::{CompilerExtension, ConversionRegistry};

use super::{PassRegistry, PipelineRegistry, RegistryError};

pub struct CompilerRegistry {
    dialects: DialectRegistry,
    passes: PassRegistry,
    pipelines: PipelineRegistry,

    conversions: ConversionRegistry,
    extensions: HashSet<Arc<str>>,
}

impl CompilerRegistry {
    pub fn new() -> Self {
        Self {
            dialects: DialectRegistry::with_builtin(),

            passes: PassRegistry::new(),

            pipelines: PipelineRegistry::new(),

            conversions: ConversionRegistry::new(),

            extensions: HashSet::new(),
        }
    }

    pub fn dialects(&self) -> &DialectRegistry {
        &self.dialects
    }

    pub fn dialects_mut(&mut self) -> &mut DialectRegistry {
        &mut self.dialects
    }

    pub fn passes(&self) -> &PassRegistry {
        &self.passes
    }

    pub fn passes_mut(&mut self) -> &mut PassRegistry {
        &mut self.passes
    }

    pub fn pipelines(&self) -> &PipelineRegistry {
        &self.pipelines
    }

    pub fn pipelines_mut(&mut self) -> &mut PipelineRegistry {
        &mut self.pipelines
    }

    pub fn conversions(&self) -> &ConversionRegistry {
        &self.conversions
    }

    pub fn conversions_mut(&mut self) -> &mut ConversionRegistry {
        &mut self.conversions
    }

    pub fn register_extension<E>(&mut self, extension: E) -> Result<(), RegistryError>
    where
        E: CompilerExtension,
    {
        let name = extension.name();

        if self.extensions.contains(name) {
            return Err(RegistryError::DuplicateExtension(name.to_owned()));
        }

        extension.register(self)?;

        self.extensions.insert(Arc::from(name));

        Ok(())
    }

    pub fn has_extension(&self, name: &str) -> bool {
        self.extensions.contains(name)
    }

    pub fn extensions(&self) -> impl Iterator<Item = &str> {
        self.extensions.iter().map(AsRef::as_ref)
    }
}

impl Default for CompilerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
