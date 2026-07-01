// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelArtifact {
    /// The symbol is linked into the current
    /// executable or shared runtime library.
    LinkedSymbol,

    DynamicLibrary {
        path: PathBuf,
    },

    CudaPtx {
        path: PathBuf,
    },

    CudaFatbin {
        path: PathBuf,
    },
}
