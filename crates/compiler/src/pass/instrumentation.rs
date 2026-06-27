// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use ir::Module;

use crate::pass::PassError;

pub trait PassInstrumentation: Send + Sync {
    fn before_pass(&self, _pass: &'static str, _module: &Module) {}

    fn after_pass(&self, _pass: &'static str, _module: &Module, _result: Result<(), &PassError>) {}
}

pub struct NoopInstrumentation;

impl PassInstrumentation for NoopInstrumentation {}
