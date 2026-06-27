// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod compiler;
mod error;
mod pass;
mod pipeline;

pub use compiler::CompilerRegistry;
pub use error::RegistryError;
pub use pass::PassRegistry;

pub use pipeline::{PipelineDescriptor, PipelineRegistry};

pub(crate) use pass::is_valid_registry_name;
