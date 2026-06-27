// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use ir::{OperationName, PatternBenefit, RewritePattern, RewritePatternSet};

use crate::conversion::{TypeConversionPattern, typed_pattern::TypeConversionPatternStorage};

pub struct ConversionPatternSet {
    patterns: RewritePatternSet,
    typed: TypeConversionPatternStorage,
}

impl ConversionPatternSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<P>(&mut self, root: impl AsRef<str>, benefit: PatternBenefit, pattern: P)
    where
        P: RewritePattern + 'static,
    {
        self.patterns.add(root, benefit, pattern);
    }

    pub fn add_any<P>(&mut self, benefit: PatternBenefit, pattern: P)
    where
        P: RewritePattern + 'static,
    {
        self.patterns.add_any(benefit, pattern);
    }

    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    pub(crate) fn rewrite_patterns(&self) -> &RewritePatternSet {
        &self.patterns
    }

    pub fn add_typed<P>(&mut self, root: impl AsRef<str>, benefit: PatternBenefit, pattern: P)
    where
        P: TypeConversionPattern + 'static,
    {
        self.typed
            .add(Some(OperationName::new(root)), benefit, pattern);
    }

    pub fn add_any_typed<P>(&mut self, benefit: PatternBenefit, pattern: P)
    where
        P: TypeConversionPattern + 'static,
    {
        self.typed.add(None, benefit, pattern);
    }

    pub(crate) fn typed_patterns(&self) -> &TypeConversionPatternStorage {
        &self.typed
    }
}

impl Default for ConversionPatternSet {
    fn default() -> Self {
        Self {
            patterns: RewritePatternSet::new(),

            typed: TypeConversionPatternStorage::default(),
        }
    }
}
