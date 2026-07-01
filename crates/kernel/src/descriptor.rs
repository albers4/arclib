// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use crate::{KernelAbi, KernelAccess, KernelArtifact, KernelBackend};


#[derive(
    Debug,
    Clone,
)]
pub struct KernelDescriptor {
    name: Arc<str>,
    symbol: Arc<str>,
    backend: KernelBackend,
    artifact: KernelArtifact,
    abi: KernelAbi,
}

impl KernelDescriptor {
    pub fn new(
        name: impl AsRef<str>,
        symbol: impl AsRef<str>,
        backend: KernelBackend,
    ) -> Self {
        Self {
            name:
                Arc::from(
                    name.as_ref(),
                ),
            symbol:
                Arc::from(
                    symbol.as_ref(),
                ),
            backend,
            artifact: KernelArtifact::LinkedSymbol,
            abi: KernelAbi::new(),
        }
    }

    pub fn with_artifact(
        mut self,
        artifact:
            KernelArtifact,
    ) -> Self {
        self.artifact = artifact;
        self
    }

    pub fn with_abi(
        mut self,
        abi: KernelAbi,
    ) -> Self {
        self.abi = abi;
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn backend(
        &self,
    ) -> KernelBackend {
        self.backend
    }

    pub fn artifact(
        &self,
    ) -> &KernelArtifact {
        &self.artifact
    }

    pub fn abi(&self) -> &KernelAbi {
        &self.abi
    }

    pub fn validate(
        &self,
    ) -> Result<(), String> {
        if self.name.is_empty() {
            return Err(
                "kernel name must not be empty"
                    .into(),
            );
        }

        if self.symbol.is_empty() {
            return Err(
                "kernel symbol must not be empty"
                    .into(),
            );
        }

        for (
            result_index,
            parameter_index,
        ) in self
            .abi
            .result_aliases()
            .iter()
            .copied()
            .enumerate()
        {
            let parameter =
                self.abi
                    .parameters()
                    .get(parameter_index)
                    .ok_or_else(|| {
                        format!(
                            "kernel result {result_index} \
                            aliases missing ABI parameter \
                            {parameter_index}"
                        )
                    })?;

            if parameter.access()
                == KernelAccess::Read
            {
                return Err(
                    format!(
                        "kernel result {result_index} \
                        aliases read-only parameter \
                        '{name}'",
                        name =
                            parameter.name(),
                    ),
                );
            }
        }

        Ok(())
    }
}