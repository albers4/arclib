// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::{ConstraintPredicate, PolicyValue, Property};

use super::TargetProfile;

#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityRequirement {
    property: Property,

    predicate: ConstraintPredicate,
}

impl CapabilityRequirement {
    pub fn new(property: impl AsRef<str>, predicate: ConstraintPredicate) -> Self {
        Self {
            property: Property::new(property),

            predicate,
        }
    }

    pub fn present(property: impl AsRef<str>) -> Self {
        Self::new(property, ConstraintPredicate::Present)
    }

    pub fn equals(property: impl AsRef<str>, value: impl Into<PolicyValue>) -> Self {
        Self::new(property, ConstraintPredicate::Equals(value.into()))
    }

    pub fn at_least(property: impl AsRef<str>, value: impl Into<PolicyValue>) -> Self {
        Self::new(property, ConstraintPredicate::AtLeast(value.into()))
    }

    pub fn at_most(property: impl AsRef<str>, value: impl Into<PolicyValue>) -> Self {
        Self::new(property, ConstraintPredicate::AtMost(value.into()))
    }

    pub fn property(&self) -> &Property {
        &self.property
    }

    pub fn predicate(&self) -> &ConstraintPredicate {
        &self.predicate
    }

    pub fn is_satisfied_by(&self, target: Option<&TargetProfile>) -> bool {
        let Some(target) = target else {
            return false;
        };

        predicate_matches(&self.predicate, target.get(&self.property))
    }
}

pub fn predicate_matches(predicate: &ConstraintPredicate, selected: Option<&PolicyValue>) -> bool {
    match predicate {
        ConstraintPredicate::Equals(expected) => selected == Some(expected),

        ConstraintPredicate::NotEquals(forbidden) => {
            selected.is_some_and(|value| value != forbidden)
        }

        ConstraintPredicate::OneOf(candidates) => {
            selected.is_some_and(|value| candidates.contains(value))
        }

        ConstraintPredicate::AtLeast(minimum) => numeric(selected)
            .zip(numeric(Some(minimum)))
            .is_some_and(|(selected, minimum)| selected >= minimum),

        ConstraintPredicate::AtMost(maximum) => numeric(selected)
            .zip(numeric(Some(maximum)))
            .is_some_and(|(selected, maximum)| selected <= maximum),

        ConstraintPredicate::Present => selected.is_some(),
    }
}

fn numeric(value: Option<&PolicyValue>) -> Option<f64> {
    match value? {
        PolicyValue::Integer(value) => Some(*value as f64),

        PolicyValue::Float(value) => Some(*value),

        _ => None,
    }
}
