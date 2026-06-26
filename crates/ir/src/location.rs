// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum SourceLocation {
    #[default]
    Unknown,

    File {
        file: Arc<str>,
        line: u32,
        column: u32,
    },

    Named {
        name: Arc<str>,
        child: Box<SourceLocation>,
    },

    Fused(Vec<SourceLocation>),
}

impl SourceLocation {
    pub fn file(file: impl AsRef<str>, line: u32, column: u32) -> Self {
        Self::File {
            file: Arc::from(file.as_ref()),
            line,
            column,
        }
    }

    pub fn named(name: impl AsRef<str>, child: SourceLocation) -> Self {
        Self::Named {
            name: Arc::from(name.as_ref()),
            child: Box::new(child),
        }
    }
}
