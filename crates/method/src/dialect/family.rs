// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MethodFamily {
    Continuum,
    Kinetic,
    Particle,
}

impl MethodFamily {
    pub const fn as_str(self) -> &'static str {
        match self {
            MethodFamily::Continuum => "continuum",
            MethodFamily::Kinetic => "kinetic",
            MethodFamily::Particle => "particle",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "continuum" => Some(Self::Continuum),
            "kinetic" => Some(Self::Kinetic),
            "particle" => Some(Self::Particle),
            _ => None,
        }
    }
}
