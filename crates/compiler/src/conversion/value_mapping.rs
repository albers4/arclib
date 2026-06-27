// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::collections::HashMap;

use ir::ValueId;

#[derive(Debug, Default)]
pub struct ConversionValueMapping {
    values: HashMap<ValueId, Vec<ValueId>>,

    block_arguments_converted: usize,
}

impl ConversionValueMapping {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn map(&mut self, source: ValueId, targets: Vec<ValueId>) {
        self.values.insert(source, targets);
    }

    pub fn get(&self, source: ValueId) -> Option<&[ValueId]> {
        self.values.get(&source).map(Vec::as_slice)
    }

    pub fn contains(&self, source: ValueId) -> bool {
        self.values.contains_key(&source)
    }

    pub fn block_arguments_converted(&self) -> usize {
        self.block_arguments_converted
    }

    pub(crate) fn record_block_arguments_converted(&mut self, count: usize) {
        self.block_arguments_converted += count;
    }
}
