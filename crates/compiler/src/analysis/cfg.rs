// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::collections::{HashMap, HashSet, VecDeque};

use ir::{BlockId, Module, RegionId};

use super::Analysis;

#[derive(Debug, Clone)]
pub struct RegionControlFlowGraph {
    region: RegionId,

    entry: Option<BlockId>,

    blocks: Vec<BlockId>,

    successors: HashMap<BlockId, Vec<BlockId>>,

    predecessors: HashMap<BlockId, Vec<BlockId>>,

    reachable: HashSet<BlockId>,
}

impl RegionControlFlowGraph {
    pub fn region(&self) -> RegionId {
        self.region
    }

    pub fn entry(&self) -> Option<BlockId> {
        self.entry
    }

    pub fn blocks(&self) -> &[BlockId] {
        &self.blocks
    }

    pub fn successors(&self, block: BlockId) -> &[BlockId] {
        self.successors
            .get(&block)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn predecessors(&self, block: BlockId) -> &[BlockId] {
        self.predecessors
            .get(&block)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn is_reachable(&self, block: BlockId) -> bool {
        self.reachable.contains(&block)
    }

    pub fn reachable_blocks(&self) -> impl Iterator<Item = BlockId> + '_ {
        self.blocks
            .iter()
            .copied()
            .filter(|block| self.is_reachable(*block))
    }
}

#[derive(Debug, Clone, Default)]
pub struct ControlFlowGraphAnalysis {
    regions: HashMap<RegionId, RegionControlFlowGraph>,

    block_regions: HashMap<BlockId, RegionId>,
}

impl ControlFlowGraphAnalysis {
    pub fn region(&self, region: RegionId) -> Option<&RegionControlFlowGraph> {
        self.regions.get(&region)
    }

    pub fn region_for_block(&self, block: BlockId) -> Option<RegionId> {
        self.block_regions.get(&block).copied()
    }

    pub fn successors(&self, block: BlockId) -> &[BlockId] {
        self.region_for_block(block)
            .and_then(|region| self.region(region))
            .map(|graph| graph.successors(block))
            .unwrap_or(&[])
    }

    pub fn predecessors(&self, block: BlockId) -> &[BlockId] {
        self.region_for_block(block)
            .and_then(|region| self.region(region))
            .map(|graph| graph.predecessors(block))
            .unwrap_or(&[])
    }

    pub fn regions(&self) -> impl Iterator<Item = (RegionId, &RegionControlFlowGraph)> {
        self.regions.iter().map(|(region, graph)| (*region, graph))
    }
}

impl Analysis for ControlFlowGraphAnalysis {
    fn name() -> &'static str {
        "control-flow-graph"
    }

    fn run(module: &Module) -> Self {
        let mut blocks_by_region: HashMap<RegionId, Vec<BlockId>> = HashMap::new();

        let mut block_regions = HashMap::new();

        for block in module.blocks() {
            let Some(block_ref) = module.block(block) else {
                continue;
            };

            let region = block_ref.parent_region();

            block_regions.insert(block, region);

            blocks_by_region.entry(region).or_default().push(block);
        }

        let mut regions = HashMap::new();

        for (region, blocks) in blocks_by_region {
            let entry = module
                .region(region)
                .and_then(|region| region.entry_block());

            let mut successors: HashMap<BlockId, Vec<BlockId>> = blocks
                .iter()
                .copied()
                .map(|block| (block, Vec::new()))
                .collect();

            let mut predecessors: HashMap<BlockId, Vec<BlockId>> = blocks
                .iter()
                .copied()
                .map(|block| (block, Vec::new()))
                .collect();

            for block in &blocks {
                let Some(block_ref) = module.block(*block) else {
                    continue;
                };

                let mut block_successors = Vec::new();

                for operation in block_ref.operations() {
                    let Some(operation_ref) = module.operation(*operation) else {
                        continue;
                    };

                    for successor in operation_ref.successors() {
                        if !block_successors.contains(successor) {
                            block_successors.push(*successor);
                        }
                    }
                }

                successors.insert(*block, block_successors.clone());

                for successor in block_successors {
                    let predecessor_list = predecessors.entry(successor).or_default();

                    if !predecessor_list.contains(block) {
                        predecessor_list.push(*block);
                    }
                }
            }

            let reachable = compute_reachable(entry, &successors);

            regions.insert(
                region,
                RegionControlFlowGraph {
                    region,
                    entry,
                    blocks,
                    successors,
                    predecessors,
                    reachable,
                },
            );
        }

        Self {
            regions,
            block_regions,
        }
    }
}

fn compute_reachable(
    entry: Option<BlockId>,

    successors: &HashMap<BlockId, Vec<BlockId>>,
) -> HashSet<BlockId> {
    let Some(entry) = entry else {
        return HashSet::new();
    };

    let mut reachable = HashSet::new();

    let mut worklist = VecDeque::from([entry]);

    while let Some(block) = worklist.pop_front() {
        if !reachable.insert(block) {
            continue;
        }

        for successor in successors.get(&block).into_iter().flatten() {
            worklist.push_back(*successor);
        }
    }

    reachable
}
