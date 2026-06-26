// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::collections::HashSet;

use slotmap::SlotMap;

use crate::{
    BlockBuilder, BlockId, BlockKey, BlockSuccessor, InsertionPoint, IrError, OperationBuilder,
    OperationId, OperationKey, OperationName, RegionId, RegionKey, SYMBOL_TABLE_ATTRIBUTE,
    SourceLocation, StorageId, Type, Use, ValueId, ValueKey, ValueProducer,
    attribute::{Attribute, AttributeMap},
    block::BlockData,
    operation::OperationData,
    region::RegionData,
    value::{UsePosition, ValueData},
};

#[derive(Debug)]
pub struct IrStorage {
    id: StorageId,

    operations: SlotMap<OperationKey, OperationData>,
    regions: SlotMap<RegionKey, RegionData>,
    blocks: SlotMap<BlockKey, BlockData>,
    values: SlotMap<ValueKey, ValueData>,
}

impl IrStorage {
    pub fn new() -> Self {
        Self {
            id: StorageId::new(),
            operations: SlotMap::with_key(),
            regions: SlotMap::with_key(),
            blocks: SlotMap::with_key(),
            values: SlotMap::with_key(),
        }
    }

    pub fn create_module_root(
        &mut self,
        mut attributes: AttributeMap,
    ) -> (OperationId, RegionId, BlockId) {
        attributes.insert(SYMBOL_TABLE_ATTRIBUTE, Attribute::Unit);

        let root_key = self.operations.insert(OperationData {
            name: OperationName::new("builtin.module"),
            parent_block: None,
            operands: Vec::new(),
            results: Vec::new(),
            regions: Vec::new(),
            successors: Vec::new(),
            successor_operands: Vec::new(),
            attributes,
            location: SourceLocation::Unknown,
        });

        let root = OperationId::new(self.id, root_key);

        let body_key = self.regions.insert(RegionData {
            parent_operation: root,
            blocks: Vec::new(),
        });

        let body = RegionId::new(self.id, body_key);

        let entry_key = self.blocks.insert(BlockData {
            parent_region: body,
            arguments: Vec::new(),
            operations: Vec::new(),
        });

        let entry = BlockId::new(self.id, entry_key);

        self.operations[root_key].regions.push(body);
        self.regions[body_key].blocks.push(entry);

        (root, body, entry)
    }

    fn resolve_insertion_point(
        &self,
        insertion_point: InsertionPoint,
    ) -> Result<(BlockId, BlockKey, usize), IrError> {
        match insertion_point {
            InsertionPoint::Start(block) => {
                let block_key = self.require_block(block)?;

                Ok((block, block_key, 0))
            }

            InsertionPoint::End(block) => {
                let block_key = self.require_block(block)?;

                let index = self.blocks[block_key].operations.len();

                Ok((block, block_key, index))
            }

            InsertionPoint::Before(operation) => self.relative_insertion_point(operation, false),

            InsertionPoint::After(operation) => self.relative_insertion_point(operation, true),
        }
    }

    fn relative_insertion_point(
        &self,
        operation: OperationId,
        after: bool,
    ) -> Result<(BlockId, BlockKey, usize), IrError> {
        let operation_key = self.require_operation(operation)?;

        let block = self.operations[operation_key]
            .parent_block
            .ok_or(IrError::OperationHasNoParentBlock(operation))?;

        let block_key = self.require_block(block)?;

        let index = self.blocks[block_key]
            .operations
            .iter()
            .position(|candidate| *candidate == operation)
            .ok_or(IrError::OperationNotInParentBlock { operation, block })?;

        Ok((block, block_key, if after { index + 1 } else { index }))
    }

    pub fn insert_operation<I, S>(
        &mut self,
        insertion_point: InsertionPoint,
        builder: OperationBuilder,
        operands: I,
        successors: S,
    ) -> Result<OperationId, IrError>
    where
        I: IntoIterator<Item = ValueId>,
        S: IntoIterator<Item = BlockSuccessor>,
    {
        let (parent_block, parent_block_key, insertion_index) =
            self.resolve_insertion_point(insertion_point)?;

        let parent_region = self.blocks[parent_block_key].parent_region;

        let operands: Vec<ValueId> = operands.into_iter().collect();

        let successors: Vec<BlockSuccessor> = successors.into_iter().collect();

        // validate everything before mutating storage
        for operand in &operands {
            self.require_value(*operand)?;
        }

        for successor in &successors {
            let successor_key = self.require_block(successor.block())?;

            if self.blocks[successor_key].parent_region != parent_region {
                return Err(IrError::SuccessorOutsideParentRegion {
                    parent_block,
                    successor: successor.block(),
                });
            }

            for operand in successor.operands() {
                self.require_value(*operand)?;
            }
        }

        let parts = builder.into_parts();

        let mut successor_blocks = Vec::with_capacity(successors.len());

        let mut successor_operands = Vec::with_capacity(successors.len());

        for successor in successors {
            let (block, operands) = successor.into_parts();

            successor_blocks.push(block);
            successor_operands.push(operands);
        }

        let operation_key = self.operations.insert(OperationData {
            name: parts.name,

            parent_block: Some(parent_block),

            operands: operands.clone(),

            results: Vec::new(),

            regions: Vec::new(),

            successors: successor_blocks,

            successor_operands: successor_operands.clone(),

            attributes: parts.attributes,

            location: parts.location,
        });

        let operation = OperationId::new(self.id, operation_key);

        let mut results = Vec::with_capacity(parts.result_types.len());

        for (index, ty) in parts.result_types.into_iter().enumerate() {
            let value_key = self.values.insert(ValueData {
                ty,

                producer: ValueProducer::OperationResult { operation, index },

                uses: Vec::new(),
            });

            results.push(ValueId::new(self.id, value_key));
        }

        let mut regions = Vec::with_capacity(parts.region_count);

        for _ in 0..parts.region_count {
            let region_key = self.regions.insert(RegionData {
                parent_operation: operation,

                blocks: Vec::new(),
            });

            regions.push(RegionId::new(self.id, region_key));
        }

        self.operations[operation_key].results = results;

        self.operations[operation_key].regions = regions;

        self.blocks[parent_block_key]
            .operations
            .insert(insertion_index, operation);

        for (operand_index, operand) in operands.into_iter().enumerate() {
            let value_key = self.require_value(operand)?;

            self.values[value_key]
                .uses
                .push(Use::operand(operation, operand_index));
        }

        for (successor_index, operands) in successor_operands.into_iter().enumerate() {
            for (operand_index, operand) in operands.into_iter().enumerate() {
                let value_key = self.require_value(operand)?;

                self.values[value_key].uses.push(Use::successor_operand(
                    operation,
                    successor_index,
                    operand_index,
                ));
            }
        }

        Ok(operation)
    }

    pub fn operation_ids(&self) -> impl Iterator<Item = OperationId> + '_ {
        let storage = self.id;

        self.operations
            .keys()
            .map(move |key| OperationId::new(storage, key))
    }

    pub fn append_block(
        &mut self,
        region: RegionId,
        builder: BlockBuilder,
    ) -> Result<BlockId, IrError> {
        let region_key = self.require_region(region)?;

        let block_key = self.blocks.insert(BlockData {
            parent_region: region,
            arguments: Vec::new(),
            operations: Vec::new(),
        });

        let block = BlockId::new(self.id, block_key);

        let argument_types = builder.into_argument_types();

        let mut arguments = Vec::with_capacity(argument_types.len());

        for (index, ty) in argument_types.into_iter().enumerate() {
            let value_key = self.values.insert(ValueData {
                ty,
                producer: ValueProducer::BlockArgument { block, index },
                uses: Vec::new(),
            });

            arguments.push(ValueId::new(self.id, value_key));
        }

        self.blocks[block_key].arguments = arguments;
        self.regions[region_key].blocks.push(block);

        Ok(block)
    }

    pub fn append_block_argument(&mut self, block: BlockId, ty: Type) -> Result<ValueId, IrError> {
        let block_key = self.require_block(block)?;

        let index = self.blocks[block_key].arguments.len();

        let value_key = self.values.insert(ValueData {
            ty,
            producer: ValueProducer::BlockArgument { block, index },
            uses: Vec::new(),
        });

        let value = ValueId::new(self.id, value_key);

        self.blocks[block_key].arguments.push(value);

        Ok(value)
    }

    pub fn block_ids(&self) -> impl Iterator<Item = BlockId> + '_ {
        let storage = self.id;

        self.blocks
            .keys()
            .map(move |key| BlockId::new(storage, key))
    }

    pub fn erase_block_argument(
        &mut self,
        block: BlockId,
        index: usize,
    ) -> Result<ValueId, IrError> {
        let block_key = self.require_block(block)?;

        let argument_count = self.blocks[block_key].arguments.len();

        let value = self.blocks[block_key].arguments.get(index).copied().ok_or(
            IrError::BlockArgumentIndexOutOfBounds {
                block,
                index,
                argument_count,
            },
        )?;

        let value_key = self.require_value(value)?;

        if !self.values[value_key].uses.is_empty() {
            return Err(IrError::BlockArgumentHasUses {
                block,
                index,
                value,
            });
        }

        self.blocks[block_key].arguments.remove(index);

        self.values.remove(value_key);

        let remaining = self.blocks[block_key].arguments.clone();

        for (new_index, argument) in remaining.into_iter().enumerate() {
            let argument_key = self.require_value(argument)?;

            self.values[argument_key].producer = ValueProducer::BlockArgument {
                block,
                index: new_index,
            };
        }

        Ok(value)
    }

    pub fn replace_operand(
        &mut self,
        operation: OperationId,
        index: usize,
        replacement: ValueId,
    ) -> Result<ValueId, IrError> {
        let operation_key = self.require_operation(operation)?;

        self.require_value(replacement)?;

        let operand_count = self.operations[operation_key].operands.len();

        let previous = self.operations[operation_key]
            .operands
            .get(index)
            .copied()
            .ok_or(IrError::OperandIndexOutOfBounds {
                operation,
                index,
                operand_count,
            })?;

        if previous == replacement {
            return Ok(previous);
        }

        let position = UsePosition::Operand { index };

        self.remove_use(previous, operation, position)?;

        self.operations[operation_key].operands[index] = replacement;

        let replacement_key = self.require_value(replacement)?;

        self.values[replacement_key]
            .uses
            .push(Use::operand(operation, index));

        Ok(previous)
    }

    pub fn operation(&self, id: OperationId) -> Option<&OperationData> {
        if !id.belongs_to(self.id) {
            return None;
        }

        self.operations.get(id.key())
    }

    pub fn operation_mut(&mut self, id: OperationId) -> Option<&mut OperationData> {
        if !id.belongs_to(self.id) {
            return None;
        }

        self.operations.get_mut(id.key())
    }

    pub fn region(&self, id: RegionId) -> Option<&RegionData> {
        if !id.belongs_to(self.id) {
            return None;
        }

        self.regions.get(id.key())
    }

    pub fn block(&self, id: BlockId) -> Option<&BlockData> {
        if !id.belongs_to(self.id) {
            return None;
        }

        self.blocks.get(id.key())
    }

    pub fn value(&self, id: ValueId) -> Option<&ValueData> {
        if !id.belongs_to(self.id) {
            return None;
        }

        self.values.get(id.key())
    }

    fn require_operation(&self, id: OperationId) -> Result<OperationKey, IrError> {
        if !id.belongs_to(self.id) {
            return Err(IrError::ForeignHandle { kind: "operation" });
        }

        self.operations
            .contains_key(id.key())
            .then_some(id.key())
            .ok_or(IrError::MissingOperation(id))
    }

    fn require_block(&self, id: BlockId) -> Result<BlockKey, IrError> {
        if !id.belongs_to(self.id) {
            return Err(IrError::ForeignHandle { kind: "block" });
        }

        self.blocks
            .contains_key(id.key())
            .then_some(id.key())
            .ok_or(IrError::MissingBlock(id))
    }

    fn require_value(&self, id: ValueId) -> Result<ValueKey, IrError> {
        if !id.belongs_to(self.id) {
            return Err(IrError::ForeignHandle { kind: "value" });
        }

        self.values
            .contains_key(id.key())
            .then_some(id.key())
            .ok_or(IrError::MissingValue(id))
    }

    fn require_region(&self, id: RegionId) -> Result<RegionKey, IrError> {
        if !id.belongs_to(self.id) {
            return Err(IrError::ForeignHandle { kind: "region" });
        }

        self.regions
            .contains_key(id.key())
            .then_some(id.key())
            .ok_or(IrError::MissingRegion(id))
    }

    pub fn verify(&self) -> Result<(), IrError> {
        for (region_key, region) in &self.regions {
            let region_id = RegionId::new(self.id, region_key);

            let parent_key = self.require_operation(region.parent_operation)?;

            let occurrence_count = self.operations[parent_key]
                .regions
                .iter()
                .filter(|candidate| **candidate == region_id)
                .count();

            if occurrence_count != 1 {
                return Err(IrError::Corrupt(format!(
                    "region {region_id:?} occurs \
                            {occurrence_count} times in \
                            its parent operation"
                )));
            }

            for block in &region.blocks {
                let block_key = self.require_block(*block)?;

                if self.blocks[block_key].parent_region != region_id {
                    return Err(IrError::Corrupt(format!(
                        "block {block:?} has the \
                                wrong parent region"
                    )));
                }
            }
        }

        for (block_key, block) in &self.blocks {
            let block_id = BlockId::new(self.id, block_key);

            let region_key = self.require_region(block.parent_region)?;

            let occurrence_count = self.regions[region_key]
                .blocks
                .iter()
                .filter(|candidate| **candidate == block_id)
                .count();

            if occurrence_count != 1 {
                return Err(IrError::Corrupt(format!(
                    "block {block_id:?} occurs \
                            {occurrence_count} times in \
                            its parent region"
                )));
            }

            for (index, argument) in block.arguments.iter().enumerate() {
                let argument_key = self.require_value(*argument)?;

                let expected = ValueProducer::BlockArgument {
                    block: block_id,
                    index,
                };

                if self.values[argument_key].producer != expected {
                    return Err(IrError::Corrupt(format!(
                        "argument {index} of \
                                block {block_id:?} has \
                                an invalid producer"
                    )));
                }
            }

            for operation in &block.operations {
                let operation_key = self.require_operation(*operation)?;

                let data = &self.operations[operation_key];

                if data.parent_block != Some(block_id) {
                    return Err(IrError::Corrupt(format!(
                        "operation {operation:?} \
                                has the wrong parent block"
                    )));
                }

                if data.successors.len() != data.successor_operands.len() {
                    return Err(IrError::Corrupt(format!(
                        "operation {operation:?} \
                                has {} successors but {} \
                                successor operand lists",
                        data.successors.len(),
                        data.successor_operands.len(),
                    )));
                }

                for (operand_index, operand) in data.operands.iter().copied().enumerate() {
                    let value_key = self.require_value(operand)?;

                    let position = UsePosition::Operand {
                        index: operand_index,
                    };

                    let use_count = self.values[value_key]
                        .uses
                        .iter()
                        .filter(|usage| usage.user() == *operation && usage.position() == position)
                        .count();

                    if use_count != 1 {
                        return Err(IrError::Corrupt(format!(
                            "operand \
                                    {operand_index} of \
                                    operation \
                                    {operation:?} has \
                                    {use_count} matching \
                                    use records"
                        )));
                    }
                }

                for (successor_index, successor) in data.successors.iter().copied().enumerate() {
                    let successor_key = self.require_block(successor)?;

                    if self.blocks[successor_key].parent_region != block.parent_region {
                        return Err(IrError::SuccessorOutsideRegion {
                            operation: *operation,

                            successor,
                        });
                    }

                    for (operand_index, operand) in data.successor_operands[successor_index]
                        .iter()
                        .copied()
                        .enumerate()
                    {
                        let value_key = self.require_value(operand)?;

                        let position = UsePosition::SuccessorOperand {
                            successor_index,
                            operand_index,
                        };

                        let use_count = self.values[value_key]
                            .uses
                            .iter()
                            .filter(|usage| {
                                usage.user() == *operation && usage.position() == position
                            })
                            .count();

                        if use_count != 1 {
                            return Err(IrError::Corrupt(format!(
                                "successor operand \
                                        {operand_index} on \
                                        edge \
                                        {successor_index} of \
                                        operation \
                                        {operation:?} has \
                                        {use_count} matching \
                                        use records"
                            )));
                        }
                    }
                }
            }
        }

        for (operation_key, operation) in &self.operations {
            let operation_id = OperationId::new(self.id, operation_key);

            if let Some(parent_block) = operation.parent_block {
                let block_key = self.require_block(parent_block)?;

                let occurrence_count = self.blocks[block_key]
                    .operations
                    .iter()
                    .filter(|candidate| **candidate == operation_id)
                    .count();

                if occurrence_count != 1 {
                    return Err(IrError::Corrupt(format!(
                        "operation \
                                {operation_id:?} occurs \
                                {occurrence_count} times \
                                in its parent block"
                    )));
                }
            }

            for (index, result) in operation.results.iter().copied().enumerate() {
                let value_key = self.require_value(result)?;

                let expected = ValueProducer::OperationResult {
                    operation: operation_id,
                    index,
                };

                if self.values[value_key].producer != expected {
                    return Err(IrError::Corrupt(format!(
                        "result {index} of \
                                operation \
                                {operation_id:?} has \
                                an invalid producer"
                    )));
                }
            }

            for region in &operation.regions {
                let region_key = self.require_region(*region)?;

                if self.regions[region_key].parent_operation != operation_id {
                    return Err(IrError::Corrupt(format!(
                        "region {region:?} has \
                                the wrong parent \
                                operation"
                    )));
                }
            }
        }

        for (value_key, value) in &self.values {
            let value_id = ValueId::new(self.id, value_key);

            match value.producer {
                ValueProducer::OperationResult { operation, index } => {
                    let operation_key = self.require_operation(operation)?;

                    if self.operations[operation_key].results.get(index).copied() != Some(value_id)
                    {
                        return Err(IrError::Corrupt(format!(
                            "value {value_id:?} \
                                    has an invalid \
                                    operation-result \
                                    producer"
                        )));
                    }
                }

                ValueProducer::BlockArgument { block, index } => {
                    let block_key = self.require_block(block)?;

                    if self.blocks[block_key].arguments.get(index).copied() != Some(value_id) {
                        return Err(IrError::Corrupt(format!(
                            "value {value_id:?} \
                                    has an invalid \
                                    block-argument \
                                    producer"
                        )));
                    }
                }
            }

            for usage in &value.uses {
                let operation_key = self.require_operation(usage.user())?;

                let actual = match usage.position() {
                    UsePosition::Operand { index } => {
                        self.operations[operation_key].operands.get(index).copied()
                    }

                    UsePosition::SuccessorOperand {
                        successor_index,
                        operand_index,
                    } => self.operations[operation_key]
                        .successor_operands
                        .get(successor_index)
                        .and_then(|operands| operands.get(operand_index))
                        .copied(),
                };

                if actual != Some(value_id) {
                    return Err(IrError::Corrupt(format!(
                        "use entry for value \
                                {value_id:?} does not \
                                match its user \
                                operation"
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn replace_all_uses(&mut self, from: ValueId, to: ValueId) -> Result<usize, IrError> {
        if from == to {
            return Ok(0);
        }

        let from_key = self.require_value(from)?;

        let to_key = self.require_value(to)?;

        let from_type = self.values[from_key].ty.clone();

        let to_type = self.values[to_key].ty.clone();

        if from_type != to_type {
            return Err(IrError::ValueTypeMismatch {
                from,
                to,
                from_type,
                to_type,
            });
        }

        let uses = self.values[from_key].uses.clone();

        for usage in &uses {
            match usage.position() {
                UsePosition::Operand { index } => {
                    self.replace_operand(usage.user(), index, to)?;
                }

                UsePosition::SuccessorOperand {
                    successor_index,
                    operand_index,
                } => {
                    self.replace_successor_operand(
                        usage.user(),
                        successor_index,
                        operand_index,
                        to,
                    )?;
                }
            }
        }

        Ok(uses.len())
    }

    pub fn erase_operation(&mut self, root: OperationId) -> Result<Vec<OperationId>, IrError> {
        let root_key = self.require_operation(root)?;

        if self.operations[root_key].parent_block.is_none() {
            return Err(IrError::CannotEraseRootOperation(root));
        }

        let mut operations = Vec::new();

        let mut regions = Vec::new();

        let mut blocks = Vec::new();

        self.collect_operation_subtree(root, &mut operations, &mut regions, &mut blocks)?;

        let operation_set: HashSet<OperationId> = operations.iter().copied().collect();

        let block_set: HashSet<BlockId> = blocks.iter().copied().collect();

        for (operation_key, operation) in &self.operations {
            let predecessor = OperationId::new(self.id, operation_key);

            if operation_set.contains(&predecessor) {
                continue;
            }

            for successor in &operation.successors {
                if block_set.contains(successor) {
                    return Err(IrError::BlockHasExternalPredecessor {
                        block: *successor,

                        predecessor,
                    });
                }
            }
        }

        let mut owned_values = Vec::new();

        for operation in &operations {
            let operation_key = self.require_operation(*operation)?;

            owned_values.extend(self.operations[operation_key].results.iter().copied());
        }

        for block in &blocks {
            let block_key = self.require_block(*block)?;

            owned_values.extend(self.blocks[block_key].arguments.iter().copied());
        }

        for value in &owned_values {
            let value_key = self.require_value(*value)?;

            for usage in &self.values[value_key].uses {
                if !operation_set.contains(&usage.user()) {
                    return Err(IrError::ValueEscapesErasedOperation {
                        value: *value,
                        user: usage.user(),
                    });
                }
            }
        }

        for operation in &operations {
            let operation_key = self.require_operation(*operation)?;

            let operands = self.operations[operation_key].operands.clone();

            for (operand_index, operand) in operands.into_iter().enumerate() {
                self.remove_use(
                    operand,
                    *operation,
                    UsePosition::Operand {
                        index: operand_index,
                    },
                )?;
            }

            let successor_operands = self.operations[operation_key].successor_operands.clone();

            for (successor_index, operands) in successor_operands.into_iter().enumerate() {
                for (operand_index, operand) in operands.into_iter().enumerate() {
                    self.remove_use(
                        operand,
                        *operation,
                        UsePosition::SuccessorOperand {
                            successor_index,
                            operand_index,
                        },
                    )?;
                }
            }
        }

        for operation in &operations {
            let operation_key = self.require_operation(*operation)?;

            if let Some(parent) = self.operations[operation_key].parent_block {
                let block_key = self.require_block(parent)?;

                self.blocks[block_key]
                    .operations
                    .retain(|candidate| candidate != operation);
            }
        }

        for value in owned_values {
            let value_key = self.require_value(value)?;

            if !self.values[value_key].uses.is_empty() {
                return Err(IrError::Corrupt(format!(
                    "value {value:?} still \
                            has uses while erasing \
                            its defining subtree"
                )));
            }

            self.values.remove(value_key);
        }

        for operation in operations.iter().rev() {
            let operation_key = self.require_operation(*operation)?;

            self.operations.remove(operation_key);
        }

        for block in blocks.iter().rev() {
            let block_key = self.require_block(*block)?;

            self.blocks.remove(block_key);
        }

        for region in regions.iter().rev() {
            let region_key = self.require_region(*region)?;

            self.regions.remove(region_key);
        }

        Ok(operations)
    }

    fn collect_operation_subtree(
        &self,
        operation: OperationId,
        operations: &mut Vec<OperationId>,
        regions: &mut Vec<RegionId>,
        blocks: &mut Vec<BlockId>,
    ) -> Result<(), IrError> {
        let operation_key = self.require_operation(operation)?;

        operations.push(operation);

        let operation_regions = self.operations[operation_key].regions.clone();

        for region in operation_regions {
            regions.push(region);

            let region_key = self.require_region(region)?;

            let region_blocks = self.regions[region_key].blocks.clone();

            for block in region_blocks {
                blocks.push(block);

                let block_key = self.require_block(block)?;

                let block_operations = self.blocks[block_key].operations.clone();

                for child in block_operations {
                    self.collect_operation_subtree(child, operations, regions, blocks)?;
                }
            }
        }

        Ok(())
    }

    pub fn move_region_contents(
        &mut self,
        source: RegionId,
        target: RegionId,
    ) -> Result<(), IrError> {
        if source == target {
            return Err(IrError::CannotMoveRegionIntoItself(source));
        }

        let source_key = self.require_region(source)?;

        let target_key = self.require_region(target)?;

        if !self.regions[target_key].blocks.is_empty() {
            return Err(IrError::TargetRegionNotEmpty(target));
        }

        let blocks = std::mem::take(&mut self.regions[source_key].blocks);

        for block in &blocks {
            let block_key = self.require_block(*block)?;

            self.blocks[block_key].parent_region = target;
        }

        self.regions[target_key].blocks = blocks;

        Ok(())
    }

    fn remove_use(
        &mut self,
        value: ValueId,
        user: OperationId,
        position: UsePosition,
    ) -> Result<(), IrError> {
        let value_key = self.require_value(value)?;

        let use_index = self.values[value_key]
            .uses
            .iter()
            .position(|usage| usage.user() == user && usage.position() == position)
            .ok_or_else(|| {
                IrError::Corrupt(format!(
                    "missing use record for \
                        value {value:?}, user \
                        {user:?}, position \
                        {position:?}"
                ))
            })?;

        self.values[value_key].uses.swap_remove(use_index);

        Ok(())
    }

    pub fn replace_successor_operand(
        &mut self,
        operation: OperationId,
        successor_index: usize,
        operand_index: usize,
        replacement: ValueId,
    ) -> Result<ValueId, IrError> {
        let operation_key = self.require_operation(operation)?;

        self.require_value(replacement)?;

        let operand_count = self.operations[operation_key]
            .successor_operands
            .get(successor_index)
            .map(Vec::len)
            .ok_or(IrError::SuccessorIndexOutOfBounds {
                operation,
                index: successor_index,
                successor_count: self.operations[operation_key].successors.len(),
            })?;

        let previous = *self.operations[operation_key].successor_operands[successor_index]
            .get(operand_index)
            .ok_or(IrError::SuccessorOperandIndexOutOfBounds {
                operation,
                successor_index,
                operand_index,
                operand_count,
            })?;

        if previous == replacement {
            return Ok(previous);
        }

        let position = UsePosition::SuccessorOperand {
            successor_index,
            operand_index,
        };

        self.remove_use(previous, operation, position)?;

        self.operations[operation_key].successor_operands[successor_index][operand_index] =
            replacement;

        let replacement_key = self.require_value(replacement)?;

        self.values[replacement_key].uses.push(Use {
            user: operation,
            position,
        });

        Ok(previous)
    }

    pub fn set_successor_operands(
        &mut self,
        operation: OperationId,
        successor_index: usize,
        replacements: impl IntoIterator<Item = ValueId>,
    ) -> Result<Vec<ValueId>, IrError> {
        let operation_key = self.require_operation(operation)?;

        let successor_count = self.operations[operation_key].successors.len();

        if successor_index >= successor_count {
            return Err(IrError::SuccessorIndexOutOfBounds {
                operation,
                index: successor_index,
                successor_count,
            });
        }

        let replacements: Vec<ValueId> = replacements.into_iter().collect();

        for replacement in &replacements {
            self.require_value(*replacement)?;
        }

        let previous = self.operations[operation_key].successor_operands[successor_index].clone();

        for (operand_index, operand) in previous.iter().copied().enumerate() {
            self.remove_use(
                operand,
                operation,
                UsePosition::SuccessorOperand {
                    successor_index,
                    operand_index,
                },
            )?;
        }

        for (operand_index, operand) in replacements.iter().copied().enumerate() {
            let value_key = self.require_value(operand)?;

            self.values[value_key].uses.push(Use::successor_operand(
                operation,
                successor_index,
                operand_index,
            ));
        }

        self.operations[operation_key].successor_operands[successor_index] = replacements;

        Ok(previous)
    }
}
