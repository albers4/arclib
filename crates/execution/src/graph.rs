// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::collections::{HashMap, HashSet, VecDeque};

use kernel::{KernelAccess, KernelValueKind};

use crate::{
    BufferCopy, BufferFill, ExecutionCommand, ExecutionError, KernelInvocation,
    ResourceDeclaration, ResourceId, ResourceTable,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExecutionNodeId(u32);

impl ExecutionNodeId {
    pub const fn index(self) -> u32 {
        self.0
    }

    fn from_index(index: usize) -> Self {
        Self(u32::try_from(index).expect("execution node ID overflow"))
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionNode {
    command: ExecutionCommand,
    dependencies: Vec<ExecutionNodeId>,
}

impl ExecutionNode {
    pub fn command(&self) -> &ExecutionCommand {
        &self.command
    }

    pub fn dependencies(&self) -> &[ExecutionNodeId] {
        &self.dependencies
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExecutionGraph {
    nodes: Vec<ExecutionNode>,
}

impl ExecutionGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn node(&self, id: ExecutionNodeId) -> Option<&ExecutionNode> {
        self.nodes.get(id.0 as usize)
    }

    pub fn command(&self, id: ExecutionNodeId) -> Option<&ExecutionCommand> {
        self.node(id).map(ExecutionNode::command)
    }

    pub fn dependencies(&self, id: ExecutionNodeId) -> Option<&[ExecutionNodeId]> {
        self.node(id).map(ExecutionNode::dependencies)
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = (ExecutionNodeId, &ExecutionNode)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (ExecutionNodeId::from_index(index), node))
    }

    pub fn schedule(&self) -> Result<ExecutionSchedule, ExecutionError> {
        let mut indegree = vec![0_usize; self.nodes.len()];
        let mut outgoing = vec![Vec::<usize>::new(); self.nodes.len()];

        for (node_index, node) in self.nodes.iter().enumerate() {
            for dependency in &node.dependencies {
                let dependency_index = dependency.0 as usize;
                if dependency_index >= self.nodes.len() {
                    return Err(ExecutionError::MissingDependency {
                        node: ExecutionNodeId::from_index(node_index),
                        dependency: *dependency,
                    });
                }

                indegree[node_index] += 1;
                outgoing[dependency_index].push(node_index);
            }
        }

        let mut ready = VecDeque::new();
        for (index, degree) in indegree.iter().copied().enumerate() {
            if degree == 0 {
                ready.push_back(index);
            }
        }

        let mut order = Vec::with_capacity(self.nodes.len());
        while let Some(index) = ready.pop_front() {
            order.push(ExecutionNodeId::from_index(index));
            for dependent in &outgoing[index] {
                indegree[*dependent] -= 1;
                if indegree[*dependent] == 0 {
                    ready.push_back(*dependent);
                }
            }
        }

        if order.len() != self.nodes.len() {
            return Err(ExecutionError::CycleDetected);
        }

        Ok(ExecutionSchedule::new(order))
    }

    pub fn validate_resources(&self, resources: &ResourceTable) -> Result<(), ExecutionError> {
        for (_, node) in self.nodes() {
            match node.command() {
                ExecutionCommand::Kernel(invocation) => {
                    for (parameter, resource) in invocation
                        .descriptor()
                        .abi()
                        .parameters()
                        .iter()
                        .zip(invocation.arguments().iter().copied())
                    {
                        let declaration = resources
                            .get(resource)
                            .ok_or(ExecutionError::MissingResource(resource))?;
                        let actual = resource_kind(declaration);
                        if actual != parameter.kind() {
                            return Err(ExecutionError::ResourceKindMismatch {
                                resource,
                                expected: parameter.kind(),
                                actual,
                            });
                        }
                    }
                }
                ExecutionCommand::Copy(copy) => {
                    validate_buffer_size(resources, copy.source(), copy.bytes())?;
                    validate_buffer_size(resources, copy.destination(), copy.bytes())?;
                }
                ExecutionCommand::Fill(fill) => {
                    validate_buffer_size(resources, fill.destination(), fill.bytes())?;
                }
                ExecutionCommand::Barrier => {}
            }
        }

        Ok(())
    }
}

fn resource_kind(declaration: &ResourceDeclaration) -> KernelValueKind {
    match declaration {
        ResourceDeclaration::Buffer(_) => KernelValueKind::Buffer,
        ResourceDeclaration::Scalar(_) => KernelValueKind::Scalar,
    }
}

fn validate_buffer_size(
    resources: &ResourceTable,
    resource: ResourceId,
    required: usize,
) -> Result<(), ExecutionError> {
    let declaration = resources
        .get(resource)
        .ok_or(ExecutionError::MissingResource(resource))?;

    let ResourceDeclaration::Buffer(buffer) = declaration else {
        return Err(ExecutionError::ResourceKindMismatch {
            resource,
            expected: KernelValueKind::Buffer,
            actual: KernelValueKind::Scalar,
        });
    };

    if buffer.spec().bytes() < required {
        return Err(ExecutionError::BufferTooSmall {
            resource,
            required,
            available: buffer.spec().bytes(),
        });
    }

    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct ExecutionSchedule {
    order: Vec<ExecutionNodeId>,
    positions: HashMap<ExecutionNodeId, usize>,
}

impl ExecutionSchedule {
    fn new(order: Vec<ExecutionNodeId>) -> Self {
        let positions = order
            .iter()
            .copied()
            .enumerate()
            .map(|(position, node)| (node, position))
            .collect();

        Self { order, positions }
    }

    pub fn order(&self) -> &[ExecutionNodeId] {
        &self.order
    }

    pub fn position(&self, node: ExecutionNodeId) -> Option<usize> {
        self.positions.get(&node).copied()
    }
}

#[derive(Debug, Default)]
pub struct ExecutionGraphBuilder {
    graph: ExecutionGraph,
    last_writer: HashMap<ResourceId, ExecutionNodeId>,
    readers: HashMap<ResourceId, Vec<ExecutionNodeId>>,
    last_barrier: Option<ExecutionNodeId>,
}

impl ExecutionGraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_command(
        &mut self,
        command: ExecutionCommand,
    ) -> Result<ExecutionNodeId, ExecutionError> {
        self.add_command_with_dependencies(command, [])
    }

    pub fn add_command_with_dependencies(
        &mut self,
        command: ExecutionCommand,
        explicit_dependencies: impl IntoIterator<Item = ExecutionNodeId>,
    ) -> Result<ExecutionNodeId, ExecutionError> {
        let explicit_dependencies: Vec<_> = explicit_dependencies.into_iter().collect();
        for dependency in &explicit_dependencies {
            if dependency.0 as usize >= self.graph.nodes.len() {
                return Err(ExecutionError::InvalidExplicitDependency {
                    dependency: *dependency,
                    node_count: self.graph.nodes.len(),
                });
            }
        }

        let id = ExecutionNodeId::from_index(self.graph.nodes.len());
        let mut dependencies: HashSet<_> = explicit_dependencies.into_iter().collect();

        if matches!(&command, ExecutionCommand::Barrier) {
            dependencies.extend(
                (0..self.graph.nodes.len()).map(ExecutionNodeId::from_index),
            );
        } else {
            if let Some(barrier) = self.last_barrier {
                dependencies.insert(barrier);
            }

            for (resource, access) in merged_accesses(&command) {
                match access {
                    KernelAccess::Read => {
                        if let Some(writer) = self.last_writer.get(&resource) {
                            dependencies.insert(*writer);
                        }
                    }
                    KernelAccess::Write | KernelAccess::ReadWrite => {
                        if let Some(writer) = self.last_writer.get(&resource) {
                            dependencies.insert(*writer);
                        }
                        if let Some(readers) = self.readers.get(&resource) {
                            dependencies.extend(readers.iter().copied());
                        }
                    }
                }
            }
        }

        let mut dependencies: Vec<_> = dependencies.into_iter().collect();
        dependencies.sort_unstable();

        self.graph.nodes.push(ExecutionNode {
            command: command.clone(),
            dependencies,
        });

        if matches!(&command, ExecutionCommand::Barrier) {
            self.last_writer.clear();
            self.readers.clear();
            self.last_barrier = Some(id);
        } else {
            for (resource, access) in merged_accesses(&command) {
                match access {
                    KernelAccess::Read => self.readers.entry(resource).or_default().push(id),
                    KernelAccess::Write | KernelAccess::ReadWrite => {
                        self.last_writer.insert(resource, id);
                        self.readers.remove(&resource);
                    }
                }
            }
        }

        Ok(id)
    }

    pub fn add_kernel(
        &mut self,
        invocation: KernelInvocation,
    ) -> Result<ExecutionNodeId, ExecutionError> {
        self.add_command(ExecutionCommand::Kernel(invocation))
    }

    pub fn add_copy(&mut self, copy: BufferCopy) -> Result<ExecutionNodeId, ExecutionError> {
        self.add_command(ExecutionCommand::Copy(copy))
    }

    pub fn add_fill(&mut self, fill: BufferFill) -> Result<ExecutionNodeId, ExecutionError> {
        self.add_command(ExecutionCommand::Fill(fill))
    }

    pub fn add_barrier(&mut self) -> Result<ExecutionNodeId, ExecutionError> {
        self.add_command(ExecutionCommand::Barrier)
    }

    pub fn finish(self) -> ExecutionGraph {
        self.graph
    }
}

fn merged_accesses(command: &ExecutionCommand) -> HashMap<ResourceId, KernelAccess> {
    let mut accesses = HashMap::new();
    for (resource, access) in command.accesses() {
        accesses
            .entry(resource)
            .and_modify(|current| *current = merge_access(*current, access))
            .or_insert(access);
    }
    accesses
}

fn merge_access(lhs: KernelAccess, rhs: KernelAccess) -> KernelAccess {
    match (lhs, rhs) {
        (KernelAccess::Read, KernelAccess::Read) => KernelAccess::Read,
        (KernelAccess::Write, KernelAccess::Write) => KernelAccess::Write,
        _ => KernelAccess::ReadWrite,
    }
}

#[cfg(test)]
mod tests {
    use crate::{BufferCopy, BufferFill, ExecutionCommand, ExecutionGraphBuilder, ResourceTable};
    use crate::{BufferSpec, MemorySpace};

    #[test]
    fn derives_dependencies_for_general_commands() {
        let mut resources = ResourceTable::new();
        let first = resources.declare_buffer(BufferSpec::new(16, MemorySpace::Host));
        let second = resources.declare_buffer(BufferSpec::new(16, MemorySpace::Host));

        let mut builder = ExecutionGraphBuilder::new();
        let fill = builder
            .add_fill(BufferFill::new(first, 0, 16).unwrap())
            .unwrap();
        let copy = builder
            .add_copy(BufferCopy::new(first, second, 16).unwrap())
            .unwrap();
        let barrier = builder.add_barrier().unwrap();
        let refill = builder
            .add_command(ExecutionCommand::fill(
                BufferFill::new(second, 1, 16).unwrap(),
            ))
            .unwrap();

        let graph = builder.finish();
        assert_eq!(graph.dependencies(copy).unwrap(), &[fill]);
        assert!(graph.dependencies(barrier).unwrap().contains(&copy));
        assert_eq!(graph.dependencies(refill).unwrap(), &[barrier]);
        graph.validate_resources(&resources).unwrap();
        assert_eq!(graph.schedule().unwrap().order().len(), 4);
    }
}
