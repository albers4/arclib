// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::{
    Attribute, AttributeMap, BlockId, IrError, OperationId, OperationName, RegionId,
    SYMBOL_NAME_ATTRIBUTE, SYMBOL_TABLE_ATTRIBUTE, SYMBOL_VISIBILITY_ATTRIBUTE, SourceLocation,
    SymbolVisibility, Type, ValueId, storage::IrStorage,
};

pub struct OperationBuilder {
    name: OperationName,
    result_types: Vec<Type>,
    attributes: AttributeMap,
    region_count: usize,
    location: SourceLocation,
}

impl OperationBuilder {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: OperationName::new(name),
            result_types: Vec::new(),
            attributes: AttributeMap::new(),
            region_count: 0,
            location: SourceLocation::Unknown,
        }
    }

    pub fn result(mut self, ty: Type) -> Self {
        self.result_types.push(ty);
        self
    }

    pub fn results(mut self, types: impl IntoIterator<Item = Type>) -> Self {
        self.result_types.extend(types);
        self
    }

    pub fn attribute(mut self, name: impl AsRef<str>, value: Attribute) -> Self {
        self.attributes.insert(name, value);
        self
    }

    pub fn location(mut self, location: SourceLocation) -> Self {
        self.location = location;
        self
    }

    pub fn region(mut self) -> Self {
        self.region_count += 1;
        self
    }

    pub fn regions(mut self, count: usize) -> Self {
        self.region_count += count;
        self
    }

    pub fn symbol(mut self, name: impl AsRef<str>) -> Self {
        self.attributes
            .insert(SYMBOL_NAME_ATTRIBUTE, Attribute::string(name));

        self
    }

    pub fn visibility(mut self, visibility: SymbolVisibility) -> Self {
        self.attributes.insert(
            SYMBOL_VISIBILITY_ATTRIBUTE,
            Attribute::string(visibility.as_str()),
        );

        self
    }

    pub fn symbol_table(mut self) -> Self {
        self.attributes
            .insert(SYMBOL_TABLE_ATTRIBUTE, Attribute::Unit);

        self
    }

    pub fn into_parts(self) -> OperationParts {
        OperationParts {
            name: self.name,
            result_types: self.result_types,
            attributes: self.attributes,
            region_count: self.region_count,
            location: self.location,
        }
    }
}

pub struct OperationParts {
    pub name: OperationName,
    pub result_types: Vec<Type>,
    pub attributes: AttributeMap,
    pub region_count: usize,
    pub location: SourceLocation,
}

#[derive(Debug)]
pub struct OperationData {
    pub name: OperationName,
    pub parent_block: Option<BlockId>,

    pub operands: Vec<ValueId>,
    pub results: Vec<ValueId>,

    pub regions: Vec<RegionId>,
    pub successors: Vec<BlockId>,
    pub successor_operands: Vec<Vec<ValueId>>,

    pub attributes: AttributeMap,
    pub location: SourceLocation,
}

#[derive(Clone, Copy)]
pub struct OperationRef<'a> {
    storage: &'a IrStorage,
    id: OperationId,
}

impl<'a> OperationRef<'a> {
    pub fn new(storage: &'a IrStorage, id: OperationId) -> Self {
        Self { storage, id }
    }

    fn data(&self) -> &OperationData {
        self.storage
            .operation(self.id)
            .expect("validated OperationRef must remain valid")
    }

    pub fn id(&self) -> OperationId {
        self.id
    }

    pub fn name(&self) -> &OperationName {
        &self.data().name
    }

    pub fn parent_block(&self) -> Option<BlockId> {
        self.data().parent_block
    }

    pub fn operands(&self) -> &[ValueId] {
        &self.data().operands
    }

    pub fn operand(&self, index: usize) -> Option<ValueId> {
        self.data().operands.get(index).copied()
    }

    pub fn results(&self) -> &[ValueId] {
        &self.data().results
    }

    pub fn result(&self, index: usize) -> Option<ValueId> {
        self.data().results.get(index).copied()
    }

    pub fn result_type(&self, index: usize) -> Option<&Type> {
        let value = self.result(index)?;
        Some(&self.storage.value(value)?.ty)
    }

    pub fn regions(&self) -> &[RegionId] {
        &self.data().regions
    }

    pub fn attributes(&self) -> &AttributeMap {
        &self.data().attributes
    }

    pub fn attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes().get(name)
    }

    pub fn location(&self) -> &SourceLocation {
        &self.data().location
    }

    pub fn symbol_name(&self) -> Option<&str> {
        self.attribute(SYMBOL_NAME_ATTRIBUTE)
            .and_then(Attribute::as_str)
    }

    pub fn symbol_visibility(&self) -> Option<SymbolVisibility> {
        self.attribute(SYMBOL_VISIBILITY_ATTRIBUTE)
            .and_then(Attribute::as_str)
            .and_then(SymbolVisibility::parse)
    }

    pub fn is_symbol(&self) -> bool {
        self.symbol_name().is_some()
    }

    pub fn is_symbol_table(&self) -> bool {
        self.name().as_str() == "builtin.module"
            || matches!(
                self.attribute(SYMBOL_TABLE_ATTRIBUTE,),
                Some(Attribute::Unit)
            )
    }

    pub fn successor_count(&self) -> usize {
        self.data().successors.len()
    }

    pub fn successor(&self, index: usize) -> Option<BlockId> {
        self.data().successors.get(index).copied()
    }

    pub fn successors(&self) -> &[BlockId] {
        &self.data().successors
    }

    pub fn successor_operands(&self, index: usize) -> Option<&[ValueId]> {
        self.data().successor_operands.get(index).map(Vec::as_slice)
    }
}

pub struct OperationMut<'a> {
    storage: &'a mut IrStorage,
    id: OperationId,
}

impl<'a> OperationMut<'a> {
    pub fn new(storage: &'a mut IrStorage, id: OperationId) -> Self {
        Self { storage, id }
    }

    pub fn id(&self) -> OperationId {
        self.id
    }

    pub fn set_attribute(&mut self, name: impl AsRef<str>, value: Attribute) -> Option<Attribute> {
        self.storage
            .operation_mut(self.id)
            .expect("validated OperationMut must remain valid")
            .attributes
            .insert(name, value)
    }

    pub fn remove_attribute(&mut self, name: &str) -> Option<Attribute> {
        self.storage
            .operation_mut(self.id)
            .expect("validated OperationMut must remain valid")
            .attributes
            .remove(name)
    }

    pub fn set_location(&mut self, location: SourceLocation) {
        self.storage
            .operation_mut(self.id)
            .expect("validated OperationMut must remain valid")
            .location = location;
    }

    pub fn replace_operand(
        &mut self,
        index: usize,
        replacement: ValueId,
    ) -> Result<ValueId, IrError> {
        self.storage.replace_operand(self.id, index, replacement)
    }
}
