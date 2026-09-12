use worth_store_security::StoreSecurityScopePropagationCounters;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StableReadSecurityScopePropagationCounters {
    store_counters: StoreSecurityScopePropagationCounters,
    root_observations: u64,
    logical_decode_entries: u64,
}

impl StableReadSecurityScopePropagationCounters {
    pub const fn from_store_counters(
        store_counters: StoreSecurityScopePropagationCounters,
    ) -> Self {
        Self {
            store_counters,
            root_observations: 0,
            logical_decode_entries: 0,
        }
    }

    pub const fn with_root_observation(self) -> Self {
        Self {
            root_observations: self.root_observations + 1,
            ..self
        }
    }

    pub const fn with_logical_decode_entry(self) -> Self {
        Self {
            logical_decode_entries: self.logical_decode_entries + 1,
            ..self
        }
    }

    pub const fn store_counters(self) -> StoreSecurityScopePropagationCounters {
        self.store_counters
    }

    pub const fn root_observations(self) -> u64 {
        self.root_observations
    }

    pub const fn logical_decode_entries(self) -> u64 {
        self.logical_decode_entries
    }
}
