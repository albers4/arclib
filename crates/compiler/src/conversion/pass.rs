// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::Arc;

use ir::Module;

use crate::{AnalysisManager, Diagnostic, Pass, PassContext, PassError, TypeConverter};

use super::{
    ConversionConfig, ConversionMode, ConversionPatternSet, ConversionTarget, apply_conversion,
    apply_conversion_with_types,
};

pub struct DialectConversionPass {
    name: &'static str,

    target: ConversionTarget,
    patterns: ConversionPatternSet,
    type_converter: Option<Arc<TypeConverter>>,

    mode: ConversionMode,
    config: ConversionConfig,
}

impl DialectConversionPass {
    pub fn partial(
        name: &'static str,
        target: ConversionTarget,
        patterns: ConversionPatternSet,
    ) -> Self {
        Self {
            name,
            target,
            patterns,
            type_converter: None,

            mode: ConversionMode::Partial,

            config: ConversionConfig::default(),
        }
    }

    pub fn full(
        name: &'static str,
        target: ConversionTarget,
        patterns: ConversionPatternSet,
    ) -> Self {
        Self {
            name,
            target,
            patterns,
            type_converter: None,

            mode: ConversionMode::Full,

            config: ConversionConfig::default(),
        }
    }

    pub fn with_config(mut self, config: ConversionConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_type_converter(mut self, type_converter: Arc<TypeConverter>) -> Self {
        self.type_converter = Some(type_converter);
        self
    }
}

impl Pass for DialectConversionPass {
    fn name(&self) -> &'static str {
        self.name
    }

    fn run(
        &self,
        module: &mut Module,
        context: &mut PassContext,
        _analyses: &mut AnalysisManager,
    ) -> Result<(), PassError> {
        let result = match &self.type_converter {
            Some(type_converter) => apply_conversion_with_types(
                module,
                &self.target,
                &self.patterns,
                type_converter,
                self.mode,
                &self.config,
            ),

            None => apply_conversion(
                module,
                &self.target,
                &self.patterns,
                self.mode,
                &self.config,
            ),
        };

        match result {
            Ok(report) => {
                if report.rewrites > 0 {
                    context.mark_changed();
                }

                Ok(())
            }

            Err(error) => {
                let mut diagnostic =
                    Diagnostic::error(error.to_string()).with_code("conversion.failed");

                if let Some(operation) = error.operation() {
                    diagnostic = diagnostic.at_operation(operation);
                }

                context.emit(diagnostic);

                Err(PassError::failed(self.name(), error.to_string()))
            }
        }
    }
}
