// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::{
    Attribute, BlockId, BlockSuccessor, InsertionPoint, IrError, Module, OperationBuilder,
    OperationId, OperationRef, RegionId, Type, ValueId, ValueRef,
};

use super::RewriteError;

pub struct PatternRewriter<'a> {
    module: &'a mut Module,

    insertion_point: InsertionPoint,

    changed: bool,

    created_operations: Vec<OperationId>,
    erased_operations: Vec<OperationId>,
}

impl<'a> PatternRewriter<'a> {
    pub fn new(module: &'a mut Module, root: OperationId) -> Result<Self, RewriteError> {
        let insertion_point = {
            let operation = module
                .operation(root)
                .ok_or(IrError::MissingOperation(root))?;

            if operation.parent_block().is_some() {
                InsertionPoint::Before(root)
            } else {
                InsertionPoint::End(module.body_block())
            }
        };

        Ok(Self {
            module,
            insertion_point,
            changed: false,
            created_operations: Vec::new(),
            erased_operations: Vec::new(),
        })
    }

    pub fn module(&self) -> &Module {
        self.module
    }

    pub fn operation(&self, operation: OperationId) -> Option<OperationRef<'_>> {
        self.module.operation(operation)
    }

    pub fn value(&self, value: ValueId) -> Option<ValueRef<'_>> {
        self.module.value(value)
    }

    pub fn set_insertion_point(&mut self, insertion_point: InsertionPoint) {
        self.insertion_point = insertion_point;
    }

    pub fn set_insertion_point_before(&mut self, operation: OperationId) {
        self.insertion_point = InsertionPoint::Before(operation);
    }

    pub fn set_insertion_point_after(&mut self, operation: OperationId) {
        self.insertion_point = InsertionPoint::After(operation);
    }

    pub fn set_insertion_point_to_end(&mut self, block: BlockId) {
        self.insertion_point = InsertionPoint::End(block);
    }

    pub fn create_operation<I>(
        &mut self,
        builder: OperationBuilder,
        operands: I,
    ) -> Result<OperationId, RewriteError>
    where
        I: IntoIterator<Item = ValueId>,
    {
        let operation = self
            .module
            .insert_operation(self.insertion_point, builder, operands)?;

        // Preserve insertion order for subsequent creations.
        self.insertion_point = InsertionPoint::After(operation);

        self.created_operations.push(operation);

        self.changed = true;

        Ok(operation)
    }

    pub fn create_operation_with_successors<I, S>(
        &mut self,
        builder: OperationBuilder,
        operands: I,
        successors: S,
    ) -> Result<OperationId, RewriteError>
    where
        I: IntoIterator<Item = ValueId>,
        S: IntoIterator<Item = BlockSuccessor>,
    {
        let operation = self.module.insert_operation_with_successors(
            self.insertion_point,
            builder,
            operands,
            successors,
        )?;

        self.insertion_point = InsertionPoint::After(operation);

        self.created_operations.push(operation);

        self.changed = true;

        Ok(operation)
    }

    pub fn set_successor_operands(
        &mut self,
        operation: OperationId,
        successor_index: usize,
        operands: impl IntoIterator<Item = ValueId>,
    ) -> Result<Vec<ValueId>, RewriteError> {
        let previous = self
            .module
            .set_successor_operands(operation, successor_index, operands)?;

        self.changed = true;

        Ok(previous)
    }

    pub fn replace_all_uses(&mut self, from: ValueId, to: ValueId) -> Result<usize, RewriteError> {
        let replaced = self.module.storage_mut().replace_all_uses(from, to)?;

        if replaced > 0 {
            self.changed = true;
        }

        Ok(replaced)
    }

    pub fn replace_operand(
        &mut self,
        operation: OperationId,
        operand_index: usize,
        replacement: ValueId,
    ) -> Result<ValueId, RewriteError> {
        let previous =
            self.module
                .storage_mut()
                .replace_operand(operation, operand_index, replacement)?;

        if previous != replacement {
            self.changed = true;
        }

        Ok(previous)
    }

    pub fn replace_operation(
        &mut self,
        operation: OperationId,
        replacements: &[ValueId],
    ) -> Result<(), RewriteError> {
        let results = {
            let operation_ref = self
                .module
                .operation(operation)
                .ok_or(IrError::MissingOperation(operation))?;

            let nonempty_region = operation_ref.regions().iter().copied().find(|region| {
                self.module
                    .region(*region)
                    .is_none_or(|region| !region.is_empty())
            });

            if let Some(region) = nonempty_region {
                return Err(RewriteError::RegionNotEmptyDuringReplacement { operation, region });
            }

            operation_ref.results().to_vec()
        };

        if results.len() != replacements.len() {
            return Err(RewriteError::ResultCountMismatch {
                operation,
                expected: results.len(),
                provided: replacements.len(),
            });
        }

        // Type compatibility is checked by
        // replace_all_uses before mutation.
        for (result, replacement) in results.iter().zip(replacements.iter()) {
            let result_type = self
                .module
                .value(*result)
                .ok_or(IrError::MissingValue(*result))?
                .ty()
                .clone();

            let replacement_type = self
                .module
                .value(*replacement)
                .ok_or(IrError::MissingValue(*replacement))?
                .ty()
                .clone();

            if result_type != replacement_type {
                return Err(IrError::ValueTypeMismatch {
                    from: *result,
                    to: *replacement,
                    from_type: result_type,
                    to_type: replacement_type,
                }
                .into());
            }
        }

        for (result, replacement) in results.into_iter().zip(replacements.iter().copied()) {
            self.module
                .storage_mut()
                .replace_all_uses(result, replacement)?;
        }

        self.erase_operation(operation)?;

        Ok(())
    }

    pub fn erase_operation(&mut self, operation: OperationId) -> Result<(), RewriteError> {
        let erased = self.module.storage_mut().erase_operation(operation)?;

        self.erased_operations.extend(erased);

        self.changed = true;

        Ok(())
    }

    pub fn set_attribute(
        &mut self,
        operation: OperationId,
        name: impl AsRef<str>,
        value: Attribute,
    ) -> Result<Option<Attribute>, RewriteError> {
        let name = name.as_ref();

        if self
            .module
            .operation(operation)
            .ok_or(IrError::MissingOperation(operation))?
            .attribute(name)
            == Some(&value)
        {
            return Ok(None);
        }

        let previous = self
            .module
            .operation_mut(operation)
            .ok_or(IrError::MissingOperation(operation))?
            .set_attribute(name, value);

        self.changed = true;

        Ok(previous)
    }

    pub fn remove_attribute(
        &mut self,
        operation: OperationId,
        name: &str,
    ) -> Result<Option<Attribute>, RewriteError> {
        let previous = self
            .module
            .operation_mut(operation)
            .ok_or(IrError::MissingOperation(operation))?
            .remove_attribute(name);

        if previous.is_some() {
            self.changed = true;
        }

        Ok(previous)
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn created_operations(&self) -> &[OperationId] {
        &self.created_operations
    }

    pub fn erased_operations(&self) -> &[OperationId] {
        &self.erased_operations
    }

    pub fn at_insertion_point(module: &'a mut Module, insertion_point: InsertionPoint) -> Self {
        Self {
            module,
            insertion_point,
            changed: false,
            created_operations: Vec::new(),
            erased_operations: Vec::new(),
        }
    }

    pub fn insertion_point(&self) -> InsertionPoint {
        self.insertion_point
    }

    pub fn with_insertion_point<R, E>(
        &mut self,
        insertion_point: InsertionPoint,
        callback: impl FnOnce(&mut Self) -> Result<R, E>,
    ) -> Result<R, E> {
        let previous = self.insertion_point;

        self.insertion_point = insertion_point;

        let result = callback(self);

        self.insertion_point = previous;

        result
    }

    pub fn move_region_contents(
        &mut self,
        source: RegionId,
        target: RegionId,
    ) -> Result<(), RewriteError> {
        self.module.move_region_contents(source, target)?;

        self.changed = true;

        Ok(())
    }

    pub fn append_block_argument(
        &mut self,
        block: BlockId,
        ty: Type,
    ) -> Result<ValueId, RewriteError> {
        let value = self.module.append_block_argument(block, ty)?;

        self.changed = true;

        Ok(value)
    }

    pub fn erase_block_argument(
        &mut self,
        block: BlockId,
        index: usize,
    ) -> Result<ValueId, RewriteError> {
        let value = self.module.erase_block_argument(block, index)?;

        self.changed = true;

        Ok(value)
    }
}
