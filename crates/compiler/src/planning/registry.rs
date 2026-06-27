// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::collections::{HashMap, HashSet};

use crate::{Metric, Property, RegistryError};

use super::{ConversionEdgeDescriptor, ConversionStage};

pub struct ConversionRegistry {
    edges: Vec<ConversionEdgeDescriptor>,

    by_name: HashMap<String, usize>,
}

impl ConversionRegistry {
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            by_name: HashMap::new(),
        }
    }

    pub fn register(&mut self, edge: ConversionEdgeDescriptor) -> Result<(), RegistryError> {
        edge.validate()
            .map_err(|message| RegistryError::InvalidConversion {
                name: edge.name().to_owned(),

                message,
            })?;

        if self.by_name.contains_key(edge.name()) {
            return Err(RegistryError::DuplicateConversion(edge.name().to_owned()));
        }

        let index = self.edges.len();

        self.by_name.insert(edge.name().to_owned(), index);

        self.edges.push(edge);

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&ConversionEdgeDescriptor> {
        self.by_name
            .get(name)
            .and_then(|index| self.edges.get(*index))
    }

    pub fn edges(&self) -> &[ConversionEdgeDescriptor] {
        &self.edges
    }

    pub(crate) fn outgoing_indices(&self, stage: &ConversionStage) -> Vec<usize> {
        self.edges
            .iter()
            .enumerate()
            .filter_map(|(index, edge)| (edge.source() == stage).then_some(index))
            .collect()
    }

    pub(crate) fn edge(&self, index: usize) -> Option<&ConversionEdgeDescriptor> {
        self.edges.get(index)
    }

    pub fn properties(&self) -> HashSet<Property> {
        self.edges
            .iter()
            .flat_map(|edge| edge.properties().keys().cloned())
            .collect()
    }

    pub fn metrics(&self) -> HashSet<Metric> {
        self.edges
            .iter()
            .flat_map(|edge| edge.metrics().keys().cloned())
            .collect()
    }
}

impl Default for ConversionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
