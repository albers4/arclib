// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{fmt, sync::Arc};

use super::{ConstraintScope, PolicyError, PolicyValue};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Property(Arc<str>);

impl Property {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self(Arc::from(name.as_ref()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn is_valid(&self) -> bool {
        is_valid_key(self.as_str())
    }
}

impl fmt::Display for Property {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstraintStrength {
    Required,

    Preferred { weight: f64 },

    Hint,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintPredicate {
    Equals(PolicyValue),

    NotEquals(PolicyValue),

    OneOf(Vec<PolicyValue>),

    AtLeast(PolicyValue),

    AtMost(PolicyValue),

    Present,
}

impl ConstraintPredicate {
    fn exact_value(&self) -> Option<&PolicyValue> {
        match self {
            Self::Equals(value) => Some(value),

            _ => None,
        }
    }

    fn validate(&self, property: &Property) -> Result<(), PolicyError> {
        match self {
            Self::Equals(value) | Self::NotEquals(value) => {
                if !value.validate() {
                    return Err(PolicyError::InvalidValue);
                }
            }

            Self::OneOf(values) => {
                if values.is_empty() {
                    return Err(PolicyError::EmptyOneOf {
                        property: property.clone(),
                    });
                }

                if !values.iter().all(PolicyValue::validate) {
                    return Err(PolicyError::InvalidValue);
                }
            }

            Self::AtLeast(value) | Self::AtMost(value) => {
                if !value.validate() {
                    return Err(PolicyError::InvalidValue);
                }

                if !value.is_numeric() {
                    return Err(PolicyError::NonNumericBound {
                        property: property.clone(),
                    });
                }
            }

            Self::Present => {}
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompilationConstraint {
    scope: ConstraintScope,
    property: Property,
    predicate: ConstraintPredicate,
    strength: ConstraintStrength,
}

impl CompilationConstraint {
    pub fn new(
        scope: ConstraintScope,
        property: impl AsRef<str>,
        predicate: ConstraintPredicate,
        strength: ConstraintStrength,
    ) -> Self {
        Self {
            scope,
            property: Property::new(property),
            predicate,
            strength,
        }
    }

    pub fn required(
        scope: ConstraintScope,
        property: impl AsRef<str>,
        value: impl Into<PolicyValue>,
    ) -> Self {
        Self::new(
            scope,
            property,
            ConstraintPredicate::Equals(value.into()),
            ConstraintStrength::Required,
        )
    }

    pub fn preferred(
        scope: ConstraintScope,
        property: impl AsRef<str>,
        value: impl Into<PolicyValue>,
        weight: f64,
    ) -> Self {
        Self::new(
            scope,
            property,
            ConstraintPredicate::Equals(value.into()),
            ConstraintStrength::Preferred { weight },
        )
    }

    pub fn hint(
        scope: ConstraintScope,
        property: impl AsRef<str>,
        value: impl Into<PolicyValue>,
    ) -> Self {
        Self::new(
            scope,
            property,
            ConstraintPredicate::Equals(value.into()),
            ConstraintStrength::Hint,
        )
    }

    pub fn scope(&self) -> &ConstraintScope {
        &self.scope
    }

    pub fn property(&self) -> &Property {
        &self.property
    }

    pub fn predicate(&self) -> &ConstraintPredicate {
        &self.predicate
    }

    pub fn strength(&self) -> ConstraintStrength {
        self.strength
    }

    fn validate(&self) -> Result<(), PolicyError> {
        if !self.property.is_valid() {
            return Err(PolicyError::InvalidProperty(
                self.property.as_str().to_owned(),
            ));
        }

        if let ConstraintStrength::Preferred { weight } = self.strength {
            if !weight.is_finite() || weight <= 0.0 {
                return Err(PolicyError::InvalidWeight {
                    owner: self.property.as_str().to_owned(),
                    weight,
                });
            }
        }

        self.predicate.validate(&self.property)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConstraintSet {
    constraints: Vec<CompilationConstraint>,
}

impl ConstraintSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, constraint: CompilationConstraint) {
        self.constraints.push(constraint);
    }

    pub fn iter(&self) -> impl Iterator<Item = &CompilationConstraint> {
        self.constraints.iter()
    }

    pub fn for_property<'a>(
        &'a self,
        property: &'a Property,
    ) -> impl Iterator<Item = &'a CompilationConstraint> {
        self.constraints
            .iter()
            .filter(move |constraint| constraint.property() == property)
    }

    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty()
    }

    pub fn len(&self) -> usize {
        self.constraints.len()
    }

    pub fn validate(&self) -> Result<(), PolicyError> {
        for constraint in &self.constraints {
            constraint.validate()?;
        }

        for (index, lhs) in self.constraints.iter().enumerate() {
            if !matches!(lhs.strength(), ConstraintStrength::Required) {
                continue;
            }

            let Some(lhs_value) = lhs.predicate().exact_value() else {
                continue;
            };

            for rhs in self.constraints.iter().skip(index + 1) {
                if !matches!(rhs.strength(), ConstraintStrength::Required) {
                    continue;
                }

                if lhs.scope() != rhs.scope() || lhs.property() != rhs.property() {
                    continue;
                }

                let Some(rhs_value) = rhs.predicate().exact_value() else {
                    continue;
                };

                if lhs_value != rhs_value {
                    return Err(PolicyError::ConflictingRequiredConstraints {
                        scope: lhs.scope().clone(),
                        property: lhs.property().clone(),
                    });
                }
            }
        }

        Ok(())
    }
}

pub(crate) fn is_valid_key(name: &str) -> bool {
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
