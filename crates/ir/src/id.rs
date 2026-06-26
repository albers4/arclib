// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::atomic::{AtomicU64, Ordering};

use slotmap::new_key_type;

new_key_type! {
    pub struct OperationKey;
    pub struct RegionKey;
    pub struct BlockKey;
    pub struct ValueKey;
}

static NEXT_STORAGE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StorageId(u64);

impl StorageId {
    pub fn new() -> Self {
        Self(NEXT_STORAGE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

macro_rules! define_ir_id {
    ($name:ident, $key:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name {
            storage: StorageId,
            key: $key,
        }

        impl $name {
            pub fn new(storage: StorageId, key: $key) -> Self {
                Self { storage, key }
            }

            pub fn belongs_to(self, storage: StorageId) -> bool {
                self.storage == storage
            }

            pub fn key(self) -> $key {
                self.key
            }
        }
    };
}

define_ir_id!(OperationId, OperationKey);
define_ir_id!(RegionId, RegionKey);
define_ir_id!(BlockId, BlockKey);
define_ir_id!(ValueId, ValueKey);
