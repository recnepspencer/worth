use super::super::StoreSecurityScopePropagationCounters;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoverySecurityScopePropagationCounters {
    wal_checkpoint_store_counters: StoreSecurityScopePropagationCounters,
    root_store_counters: StoreSecurityScopePropagationCounters,
    wal_checkpoint_comparisons: u64,
    root_scope_comparisons: u64,
}

impl RecoverySecurityScopePropagationCounters {
    pub const fn from_wal_checkpoint_counters(
        wal_checkpoint_store_counters: StoreSecurityScopePropagationCounters,
    ) -> Self {
        Self {
            wal_checkpoint_store_counters,
            root_store_counters: StoreSecurityScopePropagationCounters::empty(),
            wal_checkpoint_comparisons: 1,
            root_scope_comparisons: 0,
        }
    }

    pub const fn with_root_scope_comparison(
        self,
        root_store_counters: StoreSecurityScopePropagationCounters,
    ) -> Self {
        Self {
            root_store_counters,
            root_scope_comparisons: self.root_scope_comparisons + 1,
            ..self
        }
    }

    pub const fn store_counters(self) -> StoreSecurityScopePropagationCounters {
        self.root_store_counters
    }

    pub const fn wal_checkpoint_store_counters(self) -> StoreSecurityScopePropagationCounters {
        self.wal_checkpoint_store_counters
    }

    pub const fn root_store_counters(self) -> StoreSecurityScopePropagationCounters {
        self.root_store_counters
    }

    pub const fn wal_checkpoint_comparisons(self) -> u64 {
        self.wal_checkpoint_comparisons
    }

    pub const fn root_scope_comparisons(self) -> u64 {
        self.root_scope_comparisons
    }
}
