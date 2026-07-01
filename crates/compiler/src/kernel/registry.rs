// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{
    collections::HashMap,
    sync::Arc,
};

use kernel::{
    KernelDescriptor,
    KernelError,
};

use crate::{CapabilityRequirement, CustomKernelPattern, KernelBackendRequirement, TargetProfile};


impl KernelRegistration {

}

pub struct KernelRegistration {
    descriptor: Arc<KernelDescriptor>,
    requirements: Vec<CapabilityRequirement>,
    pattern: Arc<dyn CustomKernelPattern>,
}

impl KernelRegistration {

    pub fn requires(
        mut self,
        requirement:
            CapabilityRequirement,
    ) -> Self {
        self.requirements.push(
            requirement,
        );

        self
    }

    pub fn descriptor(
        &self,
    ) -> &Arc<KernelDescriptor> {
        &self.descriptor
    }

    pub fn requirements(
        &self,
    ) -> &[
        CapabilityRequirement
    ] {
        &self.requirements
    }

    pub fn supports(
        &self,
        target:
            Option<&TargetProfile>,
    ) -> bool {
        self.descriptor
            .backend()
            .implicit_requirement()
            .is_satisfied_by(target)
            && self.requirements
                .iter()
                .all(
                    |requirement| {
                        requirement
                            .is_satisfied_by(
                                target,
                            )
                    },
                )
    }

    pub fn pattern(
        &self,
    ) -> &Arc<
        dyn CustomKernelPattern,
    > {
        &self.pattern
    }
}

#[derive(Default)]
pub struct KernelRegistry {
    by_name:
        HashMap<String, usize>,

    registrations:
        Vec<KernelRegistration>,
}

impl KernelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        registration: KernelRegistration
    ) -> Result<(), KernelError>
    {
        let descriptor = registration.descriptor();
        
        descriptor
            .validate()
            .map_err(|message| {
                KernelError::
                    InvalidDescriptor {
                        kernel:
                            descriptor
                                .name()
                                .to_owned(),

                        message,
                    }
            })?;

        if self.by_name
            .contains_key(
                descriptor.name(),
            )
        {
            return Err(
                KernelError::
                    DuplicateKernel(
                        descriptor
                            .name()
                            .to_owned(),
                    ),
            );
        }

        let index =
            self.registrations.len();

        self.by_name.insert(
            descriptor
                .name()
                .to_owned(),
            index,
        );

        self.registrations.push(
            registration,
        );

        Ok(())
    }

    pub fn descriptor(
        &self,
        name: &str,
    ) -> Option<
        Arc<KernelDescriptor>,
    > {
        self.by_name
            .get(name)
            .and_then(|index| {
                self.registrations
                    .get(*index)
            })
            .map(|registration| {
                registration
                    .descriptor
                    .clone()
            })
    }

    pub fn registrations(
        &self,
    ) -> &[
        KernelRegistration
    ] {
        &self.registrations
    }
}