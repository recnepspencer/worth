#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryWorkflowInstanceProgressCounters {
    pub(super) warm_hits: usize,
    pub(super) cold_misses: usize,
    pub(super) cold_retains: usize,
    pub(super) cold_reconstruction_transition_visits: usize,
    pub(super) incremental_advances: usize,
    pub(super) incremental_replays: usize,
    pub(super) incremental_misses: usize,
    pub(super) evictions: usize,
    pub(super) denials: usize,
    pub(super) releases: usize,
    pub(super) retained_charge_bytes: usize,
    pub(super) maximum_retained_charge_bytes: usize,
}

impl WorthQueryWorkflowInstanceProgressCounters {
    pub const fn warm_hits(self) -> usize {
        self.warm_hits
    }
    pub const fn cold_misses(self) -> usize {
        self.cold_misses
    }
    pub const fn cold_retains(self) -> usize {
        self.cold_retains
    }
    pub const fn cold_reconstruction_transition_visits(self) -> usize {
        self.cold_reconstruction_transition_visits
    }
    pub const fn incremental_advances(self) -> usize {
        self.incremental_advances
    }
    pub const fn incremental_replays(self) -> usize {
        self.incremental_replays
    }
    pub const fn incremental_misses(self) -> usize {
        self.incremental_misses
    }
    pub const fn evictions(self) -> usize {
        self.evictions
    }
    pub const fn denials(self) -> usize {
        self.denials
    }
    pub const fn releases(self) -> usize {
        self.releases
    }
    /// Logical charge used by the bounded retention policy. It includes owned
    /// values and a conservative per-entry/tree-node allowance; it is not a
    /// process allocator or resident-set measurement.
    pub const fn retained_charge_bytes(self) -> usize {
        self.retained_charge_bytes
    }
    pub const fn maximum_retained_charge_bytes(self) -> usize {
        self.maximum_retained_charge_bytes
    }

    pub(in crate::domain_computation::primary_graph) fn absorb(&mut self, shard: Self) {
        self.warm_hits = self.warm_hits.saturating_add(shard.warm_hits);
        self.cold_misses = self.cold_misses.saturating_add(shard.cold_misses);
        self.cold_retains = self.cold_retains.saturating_add(shard.cold_retains);
        self.cold_reconstruction_transition_visits = self
            .cold_reconstruction_transition_visits
            .saturating_add(shard.cold_reconstruction_transition_visits);
        self.incremental_advances = self
            .incremental_advances
            .saturating_add(shard.incremental_advances);
        self.incremental_replays = self
            .incremental_replays
            .saturating_add(shard.incremental_replays);
        self.incremental_misses = self
            .incremental_misses
            .saturating_add(shard.incremental_misses);
        self.evictions = self.evictions.saturating_add(shard.evictions);
        self.denials = self.denials.saturating_add(shard.denials);
        self.releases = self.releases.saturating_add(shard.releases);
        self.retained_charge_bytes = self
            .retained_charge_bytes
            .saturating_add(shard.retained_charge_bytes);
        self.maximum_retained_charge_bytes = self
            .maximum_retained_charge_bytes
            .saturating_add(shard.maximum_retained_charge_bytes);
    }
}
