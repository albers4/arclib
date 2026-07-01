// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

mod interfaces;
mod operations;
mod types;

pub use interfaces::{
    DifferentialOperatorInterface, LinearOperatorInterface, OperatorExpressionInterface,
};
pub use operations::{
    ADD_OPERATION, AddOp, DIMENSION_ATTRIBUTE_PREFIX, DIVERGENCE_OPERATION, DivergenceOp,
    ELEMENT_ATTRIBUTE, FACTOR_ATTRIBUTE, FIELD_OPERATION, FieldOp, GRADIENT_OPERATION, GradientOp,
    LAPLACIAN_OPERATION, LaplacianOp, OPERATOR_DIALECT, OPERATOR_STAGE, RANK_ATTRIBUTE,
    SCALE_OPERATION, SUBTRACT_OPERATION, ScaleOp, SubtractOp, TIME_DERIVATIVE_OPERATION,
    TimeDerivativeOp, register_operator_dialect,
};
pub use types::{ExpressionType, OPERATOR_EXPRESSION_TYPE};
