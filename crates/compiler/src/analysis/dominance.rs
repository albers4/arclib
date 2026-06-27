// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::collections::{HashMap, HashSet};

use ir::{BlockId, Module, OperationId, RegionId};

use super::{Analysis, ControlFlowGraphAnalysis};

#[derive(Debug, Clone)]
struct RegionDominance {
    dominators: HashMap<BlockId, HashSet<BlockId>>,

    immediate_dominators: HashMap<BlockId, Option<BlockId>>,

    reachable: HashSet<BlockId>,
}

#[derive(Debug, Clone, Default)]
pub struct DominanceAnalysis {
    regions: HashMap<RegionId, RegionDominance>,

    block_regions: HashMap<BlockId, RegionId>,

    operation_positions: HashMap<OperationId, (BlockId, usize)>,
}

impl DominanceAnalysis {
    pub fn is_reachable(&self, block: BlockId) -> bool {
        self.block_regions
            .get(&block)
            .and_then(|region| self.regions.get(region))
            .is_some_and(|dominance| dominance.reachable.contains(&block))
    }

    pub fn dominates_block(&self, dominator: BlockId, block: BlockId) -> bool {
        if dominator == block {
            return true;
        }

        let Some(region) = self.block_regions.get(&block) else {
            return false;
        };

        if self.block_regions.get(&dominator) != Some(region) {
            return false;
        }

        self.regions
            .get(region)
            .and_then(|dominance| dominance.dominators.get(&block))
            .is_some_and(|dominators| dominators.contains(&dominator))
    }

    pub fn properly_dominates_block(&self, dominator: BlockId, block: BlockId) -> bool {
        dominator != block && self.dominates_block(dominator, block)
    }

    pub fn immediate_dominator(&self, block: BlockId) -> Option<BlockId> {
        let region = self.block_regions.get(&block)?;

        self.regions
            .get(region)?
            .immediate_dominators
            .get(&block)
            .copied()
            .flatten()
    }

    pub fn dominates_operation(&self, dominator: OperationId, operation: OperationId) -> bool {
        if dominator == operation {
            return true;
        }

        let Some((dominator_block, dominator_index)) =
            self.operation_positions.get(&dominator).copied()
        else {
            return false;
        };

        let Some((operation_block, operation_index)) =
            self.operation_positions.get(&operation).copied()
        else {
            return false;
        };

        if dominator_block == operation_block {
            return dominator_index <= operation_index;
        }

        self.dominates_block(dominator_block, operation_block)
    }
}

impl Analysis for DominanceAnalysis {
    fn name() -> &'static str {
        "dominance"
    }

    fn run(module: &Module) -> Self {
        let cfg = ControlFlowGraphAnalysis::run(module);

        let mut regions = HashMap::new();

        let mut block_regions = HashMap::new();

        for (region, graph) in cfg.regions() {
            for block in graph.blocks() {
                block_regions.insert(*block, region);
            }

            regions.insert(region, compute_region_dominance(graph));
        }

        let mut operation_positions = HashMap::new();

        for block in module.blocks() {
            let Some(block_ref) = module.block(block) else {
                continue;
            };

            for (index, operation) in block_ref.operations().iter().copied().enumerate() {
                operation_positions.insert(operation, (block, index));
            }
        }

        Self {
            regions,
            block_regions,
            operation_positions,
        }
    }
}

fn compute_region_dominance(graph: &super::RegionControlFlowGraph) -> RegionDominance {
    let reachable: HashSet<BlockId> = graph.reachable_blocks().collect();

    let mut dominators: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();

    let Some(entry) = graph.entry() else {
        return RegionDominance {
            dominators,
            immediate_dominators: HashMap::new(),
            reachable,
        };
    };

    for block in graph.blocks() {
        if *block == entry {
            dominators.insert(*block, HashSet::from([entry]));
        } else if reachable.contains(block) {
            dominators.insert(*block, reachable.clone());
        } else {
            dominators.insert(*block, HashSet::from([*block]));
        }
    }

    loop {
        let mut changed = false;

        for block in graph.blocks() {
            if *block == entry || !reachable.contains(block) {
                continue;
            }

            let predecessors: Vec<BlockId> = graph
                .predecessors(*block)
                .iter()
                .copied()
                .filter(|predecessor| reachable.contains(predecessor))
                .collect();

            let mut new_dominators = if let Some(first) = predecessors.first() {
                dominators[first].clone()
            } else {
                HashSet::new()
            };

            for predecessor in predecessors.iter().skip(1) {
                new_dominators.retain(|candidate| dominators[predecessor].contains(candidate));
            }

            new_dominators.insert(*block);

            if dominators[block] != new_dominators {
                dominators.insert(*block, new_dominators);

                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    let mut immediate_dominators = HashMap::new();

    immediate_dominators.insert(entry, None);

    for block in graph.blocks() {
        if *block == entry || !reachable.contains(block) {
            continue;
        }

        let strict: Vec<BlockId> = dominators[block]
            .iter()
            .copied()
            .filter(|candidate| candidate != block)
            .collect();

        let immediate = strict.iter().copied().find(|candidate| {
            strict
                .iter()
                .all(|other| other == candidate || dominators[candidate].contains(other))
        });

        immediate_dominators.insert(*block, immediate);
    }

    RegionDominance {
        dominators,
        immediate_dominators,
        reachable,
    }
}

#[cfg(test)]
mod tests {
    use ir::{BlockBuilder, BlockSuccessor, Module, OperationBuilder};

    use super::*;

    #[test]
    fn computes_dominance_for_diamond_cfg() {
        let mut module = Module::new();

        let region = module.body_region();

        let entry = module.body_block();

        let left = module.append_block(region, BlockBuilder::new()).unwrap();

        let right = module.append_block(region, BlockBuilder::new()).unwrap();

        let merge = module.append_block(region, BlockBuilder::new()).unwrap();

        module
            .append_operation_to_block_with_successors(
                entry,
                OperationBuilder::new("cf.conditional"),
                [],
                [BlockSuccessor::new(left), BlockSuccessor::new(right)],
            )
            .unwrap();

        module
            .append_operation_to_block_with_successors(
                left,
                OperationBuilder::new("cf.branch"),
                [],
                [BlockSuccessor::new(merge)],
            )
            .unwrap();

        module
            .append_operation_to_block_with_successors(
                right,
                OperationBuilder::new("cf.branch"),
                [],
                [BlockSuccessor::new(merge)],
            )
            .unwrap();

        module.verify().unwrap();

        let dominance = DominanceAnalysis::run(&module);

        assert!(dominance.dominates_block(entry, left,));

        assert!(dominance.dominates_block(entry, right,));

        assert!(dominance.dominates_block(entry, merge,));

        assert!(!dominance.dominates_block(left, merge,));

        assert!(!dominance.dominates_block(right, merge,));

        assert_eq!(dominance.immediate_dominator(merge,), Some(entry),);
    }

    #[test]
    fn operations_dominate_later_operations_in_same_block() {
        let mut module = Module::new();

        let first = module
            .append_operation(OperationBuilder::new("test.first"), [])
            .unwrap();

        let second = module
            .append_operation(OperationBuilder::new("test.second"), [])
            .unwrap();

        let dominance = DominanceAnalysis::run(&module);

        assert!(dominance.dominates_operation(first, second,));

        assert!(!dominance.dominates_operation(second, first,));
    }
}
