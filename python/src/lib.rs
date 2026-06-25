// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use pyo3::prelude::*;
use pyo3::types::PyModule;

#[pymodule]
fn arclib(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
