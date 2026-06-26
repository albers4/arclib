// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use crate::{
    OperationId, OperationName,
    rewrite::{error::RewriteError, rewriter::PatternRewriter},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PatternBenefit(u16);

impl PatternBenefit {
    pub const MINIMUM: Self = Self(0);
    pub const DEFAULT: Self = Self(1);

    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternResult {
    NoMatch,
    Rewritten,
}

pub trait RewritePattern: Send + Sync {
    fn name(&self) -> &'static str;

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<PatternResult, RewriteError>;
}

pub(crate) struct RegisteredPattern {
    root: Option<OperationName>,
    benefit: PatternBenefit,
    pattern: Arc<dyn RewritePattern>,
}

impl RegisteredPattern {
    pub(crate) fn applies_to(&self, name: &OperationName) -> bool {
        self.root.as_ref().map_or(true, |root| root == name)
    }

    pub(crate) fn benefit(&self) -> PatternBenefit {
        self.benefit
    }

    pub(crate) fn pattern(&self) -> &dyn RewritePattern {
        self.pattern.as_ref()
    }
}

#[derive(Default)]
pub struct RewritePatternSet {
    patterns: Vec<RegisteredPattern>,
}

impl RewritePatternSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<P>(&mut self, root: impl AsRef<str>, benefit: PatternBenefit, pattern: P)
    where
        P: RewritePattern + 'static,
    {
        self.patterns.push(RegisteredPattern {
            root: Some(OperationName::new(root)),
            benefit,
            pattern: Arc::new(pattern),
        });
    }

    pub fn add_any<P>(&mut self, benefit: PatternBenefit, pattern: P)
    where
        P: RewritePattern + 'static,
    {
        self.patterns.push(RegisteredPattern {
            root: None,
            benefit,
            pattern: Arc::new(pattern),
        });
    }

    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    pub(crate) fn candidates(&self, name: &OperationName) -> Vec<&RegisteredPattern> {
        let mut candidates: Vec<&RegisteredPattern> = self
            .patterns
            .iter()
            .filter(|pattern| pattern.applies_to(name))
            .collect();

        candidates.sort_by_key(|pattern| std::cmp::Reverse(pattern.benefit()));

        candidates
    }
}
