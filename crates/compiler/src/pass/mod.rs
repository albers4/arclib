// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod error;
mod instrumentation;
mod manager;
mod pass;

pub use error::{PassError, PassFailure};

pub use instrumentation::{NoopInstrumentation, PassInstrumentation};

pub use manager::{PassManager, PassReport};

pub use pass::{Pass, PassContext};
