// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod profile;
mod requirement;

pub use profile::TargetProfile;

pub use requirement::{CapabilityRequirement, predicate_matches};
