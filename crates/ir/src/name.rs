// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{fmt, sync::Arc};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperationName(Arc<str>);

impl OperationName {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self(Arc::from(name.as_ref()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn dialect(&self) -> Option<&str> {
        self.as_str().split_once(".").map(|(dialect, _)| dialect)
    }

    pub fn operation(&self) -> &str {
        self.as_str()
            .split_once(".")
            .map_or(self.as_str(), |(_, operation)| operation)
    }
}

impl fmt::Debug for OperationName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("OperationName")
            .field(&self.as_str())
            .finish()
    }
}

impl fmt::Display for OperationName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
