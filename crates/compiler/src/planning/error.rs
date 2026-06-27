// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{error::Error, fmt};

use super::ConversionStage;

#[derive(Debug)]
pub enum PlanningError {
    NoRoute {
        source: ConversionStage,

        target: ConversionStage,
    },

    MissingPipeline {
        edge: String,
        pipeline: String,
    },
}

impl fmt::Display for PlanningError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoRoute { source, target } => {
                write!(
                    formatter,
                    "no legal conversion route \
                     exists from '{source}' \
                     to '{target}'"
                )
            }

            Self::MissingPipeline { edge, pipeline } => {
                write!(
                    formatter,
                    "conversion edge '{edge}' \
                     references missing \
                     pipeline '{pipeline}'"
                )
            }
        }
    }
}

impl Error for PlanningError {}
