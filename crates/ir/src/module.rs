// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

use crate::{
    BlockBuilder, BlockId, BlockRef, BlockSuccessor, DialectRegistry, InsertionPoint, IrError,
    OperationBuilder, OperationId, OperationMut, OperationRef, RegionId, RegionRef,
    SYMBOL_NAME_ATTRIBUTE, SymbolError, SymbolName, SymbolRef, SymbolTableRef, Type,
    UnknownOperationPolicy, ValueId, ValueRef,
    attribute::{Attribute, AttributeMap},
    storage::IrStorage,
    symbol::{resolve_symbol, verify_symbols},
};

const MODULE_OPERATION: &str = "builtin.module";

#[derive(Debug)]
pub struct Module {
    storage: IrStorage,

    root: OperationId,
    body: RegionId,
    entry: BlockId,
}

impl Module {
    pub fn new() -> Self {
        Self::with_attributes(AttributeMap::new())
    }

    pub fn storage(&self) -> &IrStorage {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut IrStorage {
        &mut self.storage
    }

    pub fn named(name: impl AsRef<str>) -> Self {
        let mut module = Self::new();
        module.set_name(name);
        module
    }

    pub fn with_attributes(attributes: AttributeMap) -> Self {
        let mut storage = IrStorage::new();

        let (root, body, entry) = storage.create_module_root(attributes);

        Self {
            storage,
            root,
            body,
            entry,
        }
    }

    pub fn root_operation(&self) -> OperationId {
        self.root
    }

    pub fn body_region(&self) -> RegionId {
        self.body
    }

    pub fn body_block(&self) -> BlockId {
        self.entry
    }

    pub fn name(&self) -> Option<&str> {
        self.attributes()
            .get(SYMBOL_NAME_ATTRIBUTE)
            .and_then(Attribute::as_str)
    }

    pub fn set_name(&mut self, name: impl AsRef<str>) {
        self.set_attribute(SYMBOL_NAME_ATTRIBUTE, Attribute::string(name));
    }

    pub fn clear_name(&mut self) {
        self.remove_attribute(SYMBOL_NAME_ATTRIBUTE);
    }

    pub fn attributes(&self) -> &AttributeMap {
        &self
            .storage
            .operation(self.root)
            .expect("module root must exist")
            .attributes
    }

    pub fn attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes().get(name)
    }

    pub fn set_attribute(
        &mut self,
        name: impl AsRef<str>,
        attribute: Attribute,
    ) -> Option<Attribute> {
        self.storage
            .operation_mut(self.root)
            .expect("module root must exist")
            .attributes
            .insert(name, attribute)
    }

    pub fn remove_attribute(&mut self, name: &str) -> Option<Attribute> {
        self.storage
            .operation_mut(self.root)
            .expect("module root must exist")
            .attributes
            .remove(name)
    }

    pub fn is_empty(&self) -> bool {
        self.storage
            .block(self.entry)
            .expect("module entry block must exist")
            .operations
            .is_empty()
    }

    pub fn verify_structure(&self) -> Result<(), ModuleError> {
        let root = self
            .storage
            .operation(self.root)
            .ok_or(ModuleError::MissingRootOperation)?;

        if root.name.as_str() != MODULE_OPERATION {
            return Err(ModuleError::InvalidRootOperation {
                found: root.name.to_string(),
            });
        }

        if root.parent_block.is_some() {
            return Err(ModuleError::RootHasParent);
        }

        if root.regions.as_slice() != [self.body] {
            return Err(ModuleError::InvalidRootRegions);
        }

        let body = self
            .storage
            .region(self.body)
            .ok_or(ModuleError::MissingBodyRegion)?;

        if body.parent_operation != self.root {
            return Err(ModuleError::InvalidBodyParent);
        }

        if body.blocks.first().copied() != Some(self.entry) {
            return Err(ModuleError::InvalidBodyBlocks);
        }

        let entry = self
            .storage
            .block(self.entry)
            .ok_or(ModuleError::MissingEntryBlock)?;

        if entry.parent_region != self.body {
            return Err(ModuleError::InvalidEntryParent);
        }

        Ok(())
    }

    fn verify_successors(&self, operation: OperationId) -> Result<(), IrError> {
        let operation_ref = self
            .operation(operation)
            .ok_or(IrError::MissingOperation(operation))?;

        if operation_ref.successor_count() == 0 {
            return Ok(());
        }

        let parent_block = operation_ref.parent_block().ok_or_else(|| {
            IrError::Corrupt(format!(
                "operation {operation:?} \
                            has successors but no \
                            parent block"
            ))
        })?;

        let parent_region = self
            .block(parent_block)
            .ok_or(IrError::MissingBlock(parent_block))?
            .parent_region();

        for successor_index in 0..operation_ref.successor_count() {
            let successor = operation_ref
                .successor(successor_index)
                .expect("verified successor index");

            let target_block = self
                .block(successor)
                .ok_or(IrError::MissingBlock(successor))?;

            if target_block.parent_region() != parent_region {
                return Err(IrError::SuccessorOutsideRegion {
                    operation,
                    successor,
                });
            }

            let expected_arguments = target_block.arguments();

            let actual_operands = operation_ref.successor_operands(successor_index).expect(
                "successor operands \
                        must parallel successors",
            );

            if actual_operands.len() != expected_arguments.len() {
                return Err(IrError::SuccessorOperandCountMismatch {
                    operation,
                    successor_index,
                    block: successor,
                    expected: expected_arguments.len(),
                    actual: actual_operands.len(),
                });
            }

            for (operand_index, (operand, argument)) in
                actual_operands.iter().zip(expected_arguments).enumerate()
            {
                let actual_type = self
                    .value(*operand)
                    .ok_or(IrError::MissingValue(*operand))?
                    .ty()
                    .clone();

                let expected_type = self
                    .value(*argument)
                    .ok_or(IrError::MissingValue(*argument))?
                    .ty()
                    .clone();

                if actual_type != expected_type {
                    return Err(IrError::SuccessorOperandTypeMismatch {
                        operation,
                        successor_index,
                        operand_index,
                        value: *operand,
                        expected: expected_type,
                        actual: actual_type,
                    });
                }
            }
        }

        Ok(())
    }

    pub fn append_operation<I>(
        &mut self,
        builder: OperationBuilder,
        operands: I,
    ) -> Result<OperationId, IrError>
    where
        I: IntoIterator<Item = ValueId>,
    {
        self.insert_operation(InsertionPoint::End(self.entry), builder, operands)
    }

    pub fn append_operation_with_successors<I, S>(
        &mut self,
        builder: OperationBuilder,
        operands: I,
        successors: S,
    ) -> Result<OperationId, IrError>
    where
        I: IntoIterator<Item = ValueId>,
        S: IntoIterator<Item = BlockSuccessor>,
    {
        self.insert_operation_with_successors(
            InsertionPoint::End(self.entry),
            builder,
            operands,
            successors,
        )
    }

    pub fn append_operation_to_block<I>(
        &mut self,
        block: BlockId,
        builder: OperationBuilder,
        operands: I,
    ) -> Result<OperationId, IrError>
    where
        I: IntoIterator<Item = ValueId>,
    {
        self.insert_operation(InsertionPoint::End(block), builder, operands)
    }

    pub fn append_operation_to_block_with_successors<I, S>(
        &mut self,
        block: BlockId,
        builder: OperationBuilder,
        operands: I,
        successors: S,
    ) -> Result<OperationId, IrError>
    where
        I: IntoIterator<Item = ValueId>,
        S: IntoIterator<Item = BlockSuccessor>,
    {
        self.insert_operation_with_successors(
            InsertionPoint::End(block),
            builder,
            operands,
            successors,
        )
    }

    pub fn operation(&self, id: OperationId) -> Option<OperationRef<'_>> {
        self.storage
            .operation(id)
            .map(|_| OperationRef::new(&self.storage, id))
    }

    pub fn operation_mut(&mut self, id: OperationId) -> Option<OperationMut<'_>> {
        if self.storage.operation(id).is_none() {
            return None;
        }

        Some(OperationMut::new(&mut self.storage, id))
    }

    pub fn operations(&self) -> Vec<OperationId> {
        self.storage.operation_ids().collect()
    }

    pub fn blocks(&self) -> Vec<BlockId> {
        self.storage.block_ids().collect()
    }

    pub fn erase_block_argument(
        &mut self,
        block: BlockId,
        index: usize,
    ) -> Result<ValueId, IrError> {
        self.storage.erase_block_argument(block, index)
    }

    pub fn value(&self, id: ValueId) -> Option<ValueRef<'_>> {
        self.storage
            .value(id)
            .map(|_| ValueRef::new(&self.storage, id))
    }

    pub fn verify(&self) -> Result<(), IrError> {
        self.verify_structure()
            .map_err(|error| IrError::InvalidModule(error.to_string()))?;

        self.storage.verify()?;

        for operation in self.operations() {
            self.verify_successors(operation)?;
        }

        self.verify_symbols()?;

        Ok(())
    }

    pub fn region(&self, id: RegionId) -> Option<RegionRef<'_>> {
        self.storage
            .region(id)
            .map(|_| RegionRef::new(&self.storage, id))
    }

    pub fn block(&self, id: BlockId) -> Option<BlockRef<'_>> {
        self.storage
            .block(id)
            .map(|_| BlockRef::new(&self.storage, id))
    }

    pub fn append_block(
        &mut self,
        region: RegionId,
        builder: BlockBuilder,
    ) -> Result<BlockId, IrError> {
        self.storage.append_block(region, builder)
    }

    pub fn append_block_argument(&mut self, block: BlockId, ty: Type) -> Result<ValueId, IrError> {
        self.storage.append_block_argument(block, ty)
    }

    pub fn insert_operation<I>(
        &mut self,
        insertion_point: InsertionPoint,
        builder: OperationBuilder,
        operands: I,
    ) -> Result<OperationId, IrError>
    where
        I: IntoIterator<Item = ValueId>,
    {
        self.insert_operation_with_successors(
            insertion_point,
            builder,
            operands,
            std::iter::empty(),
        )
    }

    pub fn insert_operation_with_successors<I, S>(
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
        self.storage
            .insert_operation(insertion_point, builder, operands, successors)
    }

    pub fn symbol_table(&self, operation: OperationId) -> Result<SymbolTableRef<'_>, SymbolError> {
        SymbolTableRef::new(&self.storage, operation)
    }

    pub fn lookup_symbol(
        &self,
        table: OperationId,
        name: impl AsRef<str>,
    ) -> Result<Option<OperationId>, SymbolError> {
        self.symbol_table(table)?
            .lookup_local(&SymbolName::new(name))
    }

    pub fn resolve_symbol(
        &self,
        user: OperationId,
        reference: &SymbolRef,
    ) -> Result<OperationId, SymbolError> {
        resolve_symbol(&self.storage, user, reference)
    }

    pub fn verify_symbols(&self) -> Result<(), SymbolError> {
        verify_symbols(&self.storage)
    }

    pub fn verify_with_registry(
        &self,
        registry: &DialectRegistry,
        unknown_policy: UnknownOperationPolicy,
    ) -> Result<(), IrError> {
        self.verify()?;

        for operation_id in self.storage.operation_ids() {
            let operation = OperationRef::new(&self.storage, operation_id);

            registry.verify_operation(operation, unknown_policy)?;
        }

        Ok(())
    }

    pub fn move_region_contents(
        &mut self,
        source: RegionId,
        target: RegionId,
    ) -> Result<(), IrError> {
        self.storage.move_region_contents(source, target)
    }

    pub fn replace_successor_operand(
        &mut self,
        operation: OperationId,
        successor_index: usize,
        operand_index: usize,
        replacement: ValueId,
    ) -> Result<ValueId, IrError> {
        self.storage.replace_successor_operand(
            operation,
            successor_index,
            operand_index,
            replacement,
        )
    }

    pub fn set_successor_operands(
        &mut self,
        operation: OperationId,
        successor_index: usize,
        replacements: impl IntoIterator<Item = ValueId>,
    ) -> Result<Vec<ValueId>, IrError> {
        self.storage
            .set_successor_operands(operation, successor_index, replacements)
    }
}

impl Default for Module {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum ModuleError {
    MissingRootOperation,
    InvalidRootOperation { found: String },
    RootHasParent,
    InvalidRootRegions,
    MissingBodyRegion,
    InvalidBodyParent,
    InvalidBodyBlocks,
    MissingEntryBlock,
    InvalidEntryParent,
}

impl fmt::Display for ModuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRootOperation => formatter.write_str("module root operation is missing"),
            Self::InvalidRootOperation { found } => {
                write!(formatter, "expected '{MODULE_OPERATION}', found '{found}'")
            }
            Self::RootHasParent => {
                formatter.write_str("module root operation must not have a parent block")
            }
            Self::InvalidRootRegions => {
                formatter.write_str("module root must own exactly its body region")
            }
            Self::MissingBodyRegion => formatter.write_str("module body region is missing"),
            Self::InvalidBodyParent => {
                formatter.write_str("module body region has the wrong parent operation")
            }
            Self::InvalidBodyBlocks => formatter.write_str(
                "module entry block must be the first block \
                    in the module body region",
            ),
            Self::MissingEntryBlock => formatter.write_str("module entry block is missing"),
            Self::InvalidEntryParent => {
                formatter.write_str("module entry block has the wrong parent region")
            }
        }
    }
}

impl Error for ModuleError {}
