// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{fmt, sync::Arc};

use crate::{OperationRef, ValueId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionBranchPoint {
    Parent,
    Region(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionBranchSuccessor {
    target: RegionBranchPoint,

    operands: Vec<ValueId>,
}

impl RegionBranchSuccessor {
    pub fn new(target: RegionBranchPoint, operands: impl IntoIterator<Item = ValueId>) -> Self {
        Self {
            target,
            operands: operands.into_iter().collect(),
        }
    }

    pub fn target(&self) -> RegionBranchPoint {
        self.target
    }

    pub fn operands(&self) -> &[ValueId] {
        &self.operands
    }
}

type SuccessorQuery = Arc<
    dyn for<'a> Fn(
            OperationRef<'a>,
            RegionBranchPoint,
        ) -> Result<Vec<RegionBranchSuccessor>, String>
        + Send
        + Sync,
>;

#[derive(Clone)]
pub struct RegionBranchOpInterface {
    query: SuccessorQuery,
}

impl RegionBranchOpInterface {
    pub fn new<F>(query: F) -> Self
    where
        F: for<'a> Fn(
                OperationRef<'a>,
                RegionBranchPoint,
            ) -> Result<Vec<RegionBranchSuccessor>, String>
            + Send
            + Sync
            + 'static,
    {
        Self {
            query: Arc::new(query),
        }
    }

    pub fn successors(
        &self,
        operation: OperationRef<'_>,
        source: RegionBranchPoint,
    ) -> Result<Vec<RegionBranchSuccessor>, String> {
        (self.query)(operation, source)
    }
}

impl fmt::Debug for RegionBranchOpInterface {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RegionBranchOpInterface")
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BranchOpInterface;

#[derive(Debug, Default, Clone, Copy)]
pub struct RegionBranchTerminatorInterface;
