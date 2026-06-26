// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{collections::HashMap, error::Error, fmt, sync::Arc};

use crate::{Attribute, OperationId, storage::IrStorage};

pub const SYMBOL_NAME_ATTRIBUTE: &str = "symbol_name";
pub const SYMBOL_VISIBILITY_ATTRIBUTE: &str = "symbol_visibility";
pub const SYMBOL_TABLE_ATTRIBUTE: &str = "symbol_table";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolName(Arc<str>);

impl SymbolName {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self(Arc::from(name.as_ref()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_valid(&self) -> bool {
        !self.as_str().is_empty() && !self.as_str().contains("::")
    }
}

impl fmt::Display for SymbolName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolRef {
    absolute: bool,
    path: Vec<SymbolName>,
}

impl SymbolRef {
    pub fn relative(root: impl AsRef<str>) -> Self {
        Self {
            absolute: false,
            path: vec![SymbolName::new(root)],
        }
    }

    pub fn absolute(root: impl AsRef<str>) -> Self {
        Self {
            absolute: true,
            path: vec![SymbolName::new(root)],
        }
    }

    pub fn nested(mut self, name: impl AsRef<str>) -> Self {
        self.path.push(SymbolName::new(name));
        self
    }

    pub fn is_absolute(&self) -> bool {
        self.absolute
    }

    pub fn path(&self) -> &[SymbolName] {
        &self.path
    }

    pub fn root(&self) -> &SymbolName {
        &self.path[0]
    }
}

impl fmt::Display for SymbolRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.absolute {
            formatter.write_str("::")?;
        }

        for (index, name) in self.path.iter().enumerate() {
            if index > 0 {
                formatter.write_str("::")?;
            }

            write!(formatter, "@{name}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolVisibility {
    Public,
    Private,
    Nested,
}

impl SymbolVisibility {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::Nested => "nested",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "public" => Some(Self::Public),
            "private" => Some(Self::Private),
            "nested" => Some(Self::Nested),
            _ => None,
        }
    }
}

pub struct SymbolTableRef<'a> {
    storage: &'a IrStorage,
    operation: OperationId,
}

impl<'a> SymbolTableRef<'a> {
    pub fn new(storage: &'a IrStorage, operation: OperationId) -> Result<Self, SymbolError> {
        if !is_symbol_table_operation(storage, operation)? {
            return Err(SymbolError::NotSymbolTable(operation));
        }

        Ok(Self { storage, operation })
    }

    pub fn operation(&self) -> OperationId {
        self.operation
    }

    pub fn lookup_local(&self, name: &SymbolName) -> Result<Option<OperationId>, SymbolError> {
        let definitions = direct_symbol_definitions(self.storage, self.operation)?;

        let mut found = None;

        for (candidate_name, operation) in definitions {
            if &candidate_name != name {
                continue;
            }

            if let Some(previous) = found {
                return Err(SymbolError::DuplicateSymbol {
                    table: self.operation,
                    name: name.clone(),
                    first: previous,
                    second: operation,
                });
            }

            found = Some(operation);
        }

        Ok(found)
    }

    pub fn symbols(&self) -> Result<Vec<(SymbolName, OperationId)>, SymbolError> {
        direct_symbol_definitions(self.storage, self.operation)
    }

    pub fn verify_unique(&self) -> Result<(), SymbolError> {
        let definitions = self.symbols()?;
        let mut names = HashMap::new();

        for (name, operation) in definitions {
            if let Some(previous) = names.insert(name.clone(), operation) {
                return Err(SymbolError::DuplicateSymbol {
                    table: self.operation,
                    name,
                    first: previous,
                    second: operation,
                });
            }
        }

        Ok(())
    }
}

pub fn resolve_symbol(
    storage: &IrStorage,
    user: OperationId,
    reference: &SymbolRef,
) -> Result<OperationId, SymbolError> {
    let mut table = if reference.is_absolute() {
        root_operation(storage, user)?
    } else {
        nearest_symbol_table(storage, user)?
    };

    if !is_symbol_table_operation(storage, table)? {
        return Err(SymbolError::NotSymbolTable(table));
    }

    let mut resolved = None;

    for (index, component) in reference.path().iter().enumerate() {
        let symbol_table = SymbolTableRef::new(storage, table)?;

        let operation = symbol_table.lookup_local(component)?.ok_or_else(|| {
            SymbolError::UnresolvedReference {
                user,
                reference: reference.clone(),
            }
        })?;

        resolved = Some(operation);

        if index + 1 < reference.path().len() {
            if !is_symbol_table_operation(storage, operation)? {
                return Err(SymbolError::NestedSymbolIsNotTable {
                    operation,
                    component: component.clone(),
                });
            }

            table = operation;
        }
    }

    resolved.ok_or_else(|| SymbolError::UnresolvedReference {
        user,
        reference: reference.clone(),
    })
}

pub fn verify_symbols(storage: &IrStorage) -> Result<(), SymbolError> {
    for operation in storage.operation_ids() {
        let data = storage
            .operation(operation)
            .ok_or(SymbolError::MissingOperation(operation))?;

        let symbol_name = operation_symbol_name(storage, operation)?;

        let visibility = operation_symbol_visibility(storage, operation)?;

        if visibility.is_some() && symbol_name.is_none() {
            return Err(SymbolError::VisibilityWithoutSymbol(operation));
        }

        if is_symbol_table_operation(storage, operation)? {
            if data.regions.is_empty() {
                return Err(SymbolError::SymbolTableWithoutRegion(operation));
            }

            SymbolTableRef::new(storage, operation)?.verify_unique()?;
        }

        for (_, attribute) in data.attributes.iter() {
            let mut references = Vec::new();

            collect_symbol_references(attribute, &mut references);

            for reference in references {
                resolve_symbol(storage, operation, reference)?;
            }
        }
    }

    Ok(())
}

pub fn operation_symbol_name(
    storage: &IrStorage,
    operation: OperationId,
) -> Result<Option<SymbolName>, SymbolError> {
    let data = storage
        .operation(operation)
        .ok_or(SymbolError::MissingOperation(operation))?;

    let Some(attribute) = data.attributes.get(SYMBOL_NAME_ATTRIBUTE) else {
        return Ok(None);
    };

    let Attribute::String(value) = attribute else {
        return Err(SymbolError::InvalidNameAttribute(operation));
    };

    let name = SymbolName::new(value.as_ref());

    if !name.is_valid() {
        return Err(SymbolError::InvalidName { operation, name });
    }

    Ok(Some(name))
}

pub fn operation_symbol_visibility(
    storage: &IrStorage,
    operation: OperationId,
) -> Result<Option<SymbolVisibility>, SymbolError> {
    let data = storage
        .operation(operation)
        .ok_or(SymbolError::MissingOperation(operation))?;

    let Some(attribute) = data.attributes.get(SYMBOL_VISIBILITY_ATTRIBUTE) else {
        return Ok(None);
    };

    let Attribute::String(value) = attribute else {
        return Err(SymbolError::InvalidVisibilityAttribute(operation));
    };

    SymbolVisibility::parse(value)
        .map(Some)
        .ok_or_else(|| SymbolError::InvalidVisibility {
            operation,
            value: value.to_string(),
        })
}

pub fn is_symbol_table_operation(
    storage: &IrStorage,
    operation: OperationId,
) -> Result<bool, SymbolError> {
    let data = storage
        .operation(operation)
        .ok_or(SymbolError::MissingOperation(operation))?;

    Ok(data.name.as_str() == "builtin.module"
        || matches!(
            data.attributes.get(SYMBOL_TABLE_ATTRIBUTE,),
            Some(Attribute::Unit)
        ))
}

fn direct_symbol_definitions(
    storage: &IrStorage,
    table: OperationId,
) -> Result<Vec<(SymbolName, OperationId)>, SymbolError> {
    let table_data = storage
        .operation(table)
        .ok_or(SymbolError::MissingOperation(table))?;

    let mut result = Vec::new();

    for region in &table_data.regions {
        let region_data = storage.region(*region).ok_or(SymbolError::Corrupt(format!(
            "symbol table {table:?} contains a missing region"
        )))?;

        for block in &region_data.blocks {
            let block_data = storage.block(*block).ok_or(SymbolError::Corrupt(format!(
                "symbol table {table:?} contains a missing block"
            )))?;

            for operation in &block_data.operations {
                if let Some(name) = operation_symbol_name(storage, *operation)? {
                    result.push((name, *operation));
                }
            }
        }
    }

    Ok(result)
}

fn nearest_symbol_table(
    storage: &IrStorage,
    operation: OperationId,
) -> Result<OperationId, SymbolError> {
    let mut current = parent_operation(storage, operation)?;

    if current.is_none() && is_symbol_table_operation(storage, operation)? {
        current = Some(operation);
    }

    while let Some(candidate) = current {
        if is_symbol_table_operation(storage, candidate)? {
            return Ok(candidate);
        }

        current = parent_operation(storage, candidate)?;
    }

    Err(SymbolError::NoEnclosingSymbolTable(operation))
}

fn root_operation(storage: &IrStorage, operation: OperationId) -> Result<OperationId, SymbolError> {
    let mut current = operation;

    while let Some(parent) = parent_operation(storage, current)? {
        current = parent;
    }

    Ok(current)
}

fn parent_operation(
    storage: &IrStorage,
    operation: OperationId,
) -> Result<Option<OperationId>, SymbolError> {
    let operation_data = storage
        .operation(operation)
        .ok_or(SymbolError::MissingOperation(operation))?;

    let Some(block) = operation_data.parent_block else {
        return Ok(None);
    };

    let block_data = storage.block(block).ok_or_else(|| {
        SymbolError::Corrupt(format!(
            "operation {operation:?} has a missing parent block"
        ))
    })?;

    let region_data = storage.region(block_data.parent_region).ok_or_else(|| {
        SymbolError::Corrupt(format!(
            "operation {operation:?} has a missing parent region"
        ))
    })?;

    Ok(Some(region_data.parent_operation))
}

fn collect_symbol_references<'a>(attribute: &'a Attribute, references: &mut Vec<&'a SymbolRef>) {
    match attribute {
        Attribute::SymbolRef(reference) => {
            references.push(reference);
        }

        Attribute::Array(values) => {
            for value in values {
                collect_symbol_references(value, references);
            }
        }

        Attribute::Dictionary(dictionary) => {
            for (_, value) in dictionary.iter() {
                collect_symbol_references(value, references);
            }
        }

        _ => {}
    }
}

#[derive(Debug)]
pub enum SymbolError {
    MissingOperation(OperationId),

    NotSymbolTable(OperationId),

    NoEnclosingSymbolTable(OperationId),

    SymbolTableWithoutRegion(OperationId),

    InvalidName {
        operation: OperationId,
        name: SymbolName,
    },

    InvalidNameAttribute(OperationId),

    InvalidVisibility {
        operation: OperationId,
        value: String,
    },

    InvalidVisibilityAttribute(OperationId),

    VisibilityWithoutSymbol(OperationId),

    DuplicateSymbol {
        table: OperationId,
        name: SymbolName,
        first: OperationId,
        second: OperationId,
    },

    UnresolvedReference {
        user: OperationId,
        reference: SymbolRef,
    },

    NestedSymbolIsNotTable {
        operation: OperationId,
        component: SymbolName,
    },

    Corrupt(String),
}

impl fmt::Display for SymbolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOperation(operation) => {
                write!(formatter, "symbol operation {operation:?} does not exist")
            }

            Self::NotSymbolTable(operation) => {
                write!(formatter, "operation {operation:?} is not a symbol table")
            }

            Self::NoEnclosingSymbolTable(operation) => {
                write!(
                    formatter,
                    "operation {operation:?} has no enclosing symbol table"
                )
            }

            Self::SymbolTableWithoutRegion(operation) => {
                write!(formatter, "symbol table {operation:?} has no region")
            }

            Self::InvalidName { operation, name } => {
                write!(
                    formatter,
                    "operation {operation:?} has invalid symbol name '{name}'"
                )
            }

            Self::InvalidNameAttribute(operation) => {
                write!(
                    formatter,
                    "operation {operation:?} has a non-string symbol name"
                )
            }

            Self::InvalidVisibility { operation, value } => {
                write!(
                    formatter,
                    "operation {operation:?} has invalid symbol visibility '{value}'"
                )
            }

            Self::InvalidVisibilityAttribute(operation) => {
                write!(
                    formatter,
                    "operation {operation:?} has a non-string symbol visibility"
                )
            }

            Self::VisibilityWithoutSymbol(operation) => {
                write!(
                    formatter,
                    "operation {operation:?} has symbol visibility but no symbol name"
                )
            }

            Self::DuplicateSymbol {
                table,
                name,
                first,
                second,
            } => {
                write!(
                    formatter,
                    "symbol table {table:?} contains duplicate symbol \
                     '{name}' at {first:?} and {second:?}"
                )
            }

            Self::UnresolvedReference { user, reference } => {
                write!(
                    formatter,
                    "operation {user:?} cannot resolve symbol reference {reference}"
                )
            }

            Self::NestedSymbolIsNotTable {
                operation,
                component,
            } => {
                write!(
                    formatter,
                    "symbol '{component}' resolves to {operation:?}, \
                     which is not a symbol table"
                )
            }

            Self::Corrupt(message) => formatter.write_str(message),
        }
    }
}

impl Error for SymbolError {}

#[cfg(test)]
mod tests {
    use crate::{Attribute, BlockBuilder, Module, OperationBuilder, SymbolRef};

    #[test]
    fn symbol_table_operation_can_reference_sibling_symbol() {
        let mut module = Module::new();

        let scope = module
            .append_operation(
                OperationBuilder::new("test.scope")
                    .symbol("scope")
                    .symbol_table()
                    .region(),
                [],
            )
            .unwrap();

        let scope_region = module.operation(scope).unwrap().regions()[0];

        let scope_block = module
            .append_block(scope_region, BlockBuilder::new())
            .unwrap();

        let target = module
            .append_operation_to_block(
                scope_block,
                OperationBuilder::new("test.target").symbol("target"),
                [],
            )
            .unwrap();

        let nested_table = module
            .append_operation_to_block(
                scope_block,
                OperationBuilder::new("test.nested")
                    .symbol("nested")
                    .symbol_table()
                    .region()
                    .attribute(
                        "test.reference",
                        Attribute::SymbolRef(SymbolRef::relative("target")),
                    ),
                [],
            )
            .unwrap();

        let nested_region = module.operation(nested_table).unwrap().regions()[0];

        module
            .append_block(nested_region, BlockBuilder::new())
            .unwrap();

        assert_eq!(
            module
                .resolve_symbol(nested_table, &SymbolRef::relative("target",),)
                .unwrap(),
            target,
        );

        module.verify_symbols().unwrap();
    }
}
