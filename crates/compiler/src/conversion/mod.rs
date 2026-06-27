// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod driver;
mod error;
mod pass;
mod pattern;
mod reconcile;
mod signature;
mod target;
mod type_converter;
mod type_error;
mod typed_pattern;
mod value_mapping;

pub use driver::{
    ConversionConfig, ConversionMode, ConversionReport, apply_conversion,
    apply_conversion_with_types, apply_full_conversion, apply_partial_conversion,
};

pub use error::ConversionError;

pub use pass::DialectConversionPass;

pub use pattern::ConversionPatternSet;

pub use target::{ConversionTarget, Legality};

pub use type_converter::{TypeConversionRuleResult, TypeConverter};

pub use type_error::TypeConversionError;

pub use typed_pattern::{ConversionPatternRewriter, TypeConversionPattern};

pub use value_mapping::ConversionValueMapping;

pub use reconcile::{
    ReconcileUnrealizedCastsPass, populate_reconcile_unrealized_cast_patterns,
    reconcile_unrealized_casts,
};

pub use signature::{
    SignatureConversion, SignatureConversionReport, apply_signature_conversion,
    convert_region_entry_signature,
};
