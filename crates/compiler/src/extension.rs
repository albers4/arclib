// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::{CompilerRegistry, RegistryError};

pub trait CompilerExtension: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    fn register(&self, registry: &mut CompilerRegistry) -> Result<(), RegistryError>;
}
