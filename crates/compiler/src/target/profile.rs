// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{collections::HashMap, fmt, sync::Arc};

use crate::{PolicyValue, Property};

#[derive(Debug, Clone, PartialEq)]
pub struct TargetProfile {
    name: Arc<str>,

    capabilities: HashMap<Property, PolicyValue>,
}

impl TargetProfile {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: Arc::from(name.as_ref()),

            capabilities: HashMap::new(),
        }
    }

    pub fn capability(mut self, property: impl AsRef<str>, value: impl Into<PolicyValue>) -> Self {
        self.capabilities
            .insert(Property::new(property), value.into());

        self
    }

    pub fn set_capability(&mut self, property: impl AsRef<str>, value: impl Into<PolicyValue>) {
        self.capabilities
            .insert(Property::new(property), value.into());
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get(&self, property: &Property) -> Option<&PolicyValue> {
        self.capabilities.get(property)
    }

    pub fn capability_value(&self, property: &str) -> Option<&PolicyValue> {
        self.capabilities.get(&Property::new(property))
    }

    pub fn capabilities(&self) -> &HashMap<Property, PolicyValue> {
        &self.capabilities
    }
}

impl fmt::Display for TargetProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}
