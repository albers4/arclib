// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use crate::{BlockId, OperationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertionPoint {
    Start(BlockId),
    End(BlockId),
    Before(OperationId),
    After(OperationId),
}
