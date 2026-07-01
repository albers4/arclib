// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum KernelBackend {
    Cpu,
    Cuda,
}

impl KernelBackend {
    pub const fn as_str(
        self,
    ) -> &'static str {
        match self {
            Self::Cpu => {
                "cpu"
            }

            Self::Cuda => {
                "cuda"
            }
        }
    }
}