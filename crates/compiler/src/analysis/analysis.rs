// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::any::Any;

use ir::Module;

pub trait Analysis: Any + Send + Sync {
    fn run(module: &Module) -> Self
    where
        Self: Sized;

    fn name() -> &'static str
    where
        Self: Sized;
}
