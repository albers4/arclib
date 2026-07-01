// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use kernel::KernelBackend;

use crate::CapabilityRequirement;

pub trait KernelBackendRequirement {
    fn implicit_requirement(
        self,
    ) -> CapabilityRequirement;
}

impl KernelBackendRequirement for KernelBackend {
    fn implicit_requirement(
        self,
    ) -> CapabilityRequirement {
        match self {
            Self::Cpu => {
                CapabilityRequirement::equals(
                    "backend.openmp",
                    true,
                )
            }

            Self::Cuda => {
                CapabilityRequirement::equals(
                    "backend.cuda",
                    true,
                )
            }
        }
    }
}