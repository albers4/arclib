// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::collections::HashMap;

use ir::{Module, OperationId, OperationName, SymbolName};

use super::Analysis;

pub struct OperationListAnalysis {
    operations: Vec<OperationId>,
}

impl OperationListAnalysis {
    pub fn operations(&self) -> &[OperationId] {
        &self.operations
    }
}

impl Analysis for OperationListAnalysis {
    fn name() -> &'static str {
        "operation-list"
    }

    fn run(module: &Module) -> Self {
        Self {
            operations: module.operations(),
        }
    }
}

pub struct OperationNameAnalysis {
    by_name: HashMap<OperationName, Vec<OperationId>>,
}

impl OperationNameAnalysis {
    pub fn get(&self, name: &OperationName) -> &[OperationId] {
        self.by_name.get(name).map(Vec::as_slice).unwrap_or(&[])
    }
}

impl Analysis for OperationNameAnalysis {
    fn name() -> &'static str {
        "operation-name"
    }

    fn run(module: &Module) -> Self {
        let mut by_name = HashMap::new();

        for operation_id in module.operations() {
            if let Some(operation) = module.operation(operation_id) {
                by_name
                    .entry(operation.name().clone())
                    .or_insert_with(Vec::new)
                    .push(operation_id);
            }
        }

        Self { by_name }
    }
}

pub struct SymbolDefinitionAnalysis {
    by_name: HashMap<SymbolName, OperationId>,
}

impl SymbolDefinitionAnalysis {
    pub fn get(&self, name: &SymbolName) -> Option<OperationId> {
        self.by_name.get(name).copied()
    }
}

impl Analysis for SymbolDefinitionAnalysis {
    fn name() -> &'static str {
        "symbol-definitions"
    }

    fn run(module: &Module) -> Self {
        let mut by_name = HashMap::new();

        if let Ok(table) = module.symbol_table(module.root_operation()) {
            if let Ok(symbols) = table.symbols() {
                for (name, operation) in symbols {
                    by_name.insert(name, operation);
                }
            }
        }

        Self { by_name }
    }
}
