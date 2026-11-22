//! Pure workflow compilation reuse and instance progress counter snapshots.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryWorkflowCompilationReuseCounters {
    warm_hits: usize,
    cold_misses: usize,
    cold_retains: usize,
    semantic_reuse_hits: usize,
    evictions: usize,
    denials: usize,
    retained_bytes: usize,
    peak_retained_bytes: usize,
    maximum_retained_bytes: usize,
}

impl WorthQueryWorkflowCompilationReuseCounters {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        warm_hits: usize,
        cold_misses: usize,
        cold_retains: usize,
        semantic_reuse_hits: usize,
        evictions: usize,
        denials: usize,
        retained_bytes: usize,
        peak_retained_bytes: usize,
        maximum_retained_bytes: usize,
    ) -> Self {
        Self {
            warm_hits,
            cold_misses,
            cold_retains,
            semantic_reuse_hits,
            evictions,
            denials,
            retained_bytes,
            peak_retained_bytes,
            maximum_retained_bytes,
        }
    }

    pub const fn warm_hits(self) -> usize {
        self.warm_hits
    }
    pub const fn cold_misses(self) -> usize {
        self.cold_misses
    }
    pub const fn cold_retains(self) -> usize {
        self.cold_retains
    }
    pub const fn semantic_reuse_hits(self) -> usize {
        self.semantic_reuse_hits
    }
    pub const fn evictions(self) -> usize {
        self.evictions
    }
    pub const fn denials(self) -> usize {
        self.denials
    }
    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }
    pub const fn peak_retained_bytes(self) -> usize {
        self.peak_retained_bytes
    }
    pub const fn maximum_retained_bytes(self) -> usize {
        self.maximum_retained_bytes
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryWorkflowInstanceProgressCounters {
    pub warm_hits: usize,
    pub warm_core_hits: usize,
    pub warm_history_transition_visits: usize,
    pub cold_misses: usize,
    pub cold_retains: usize,
    pub cold_reconstruction_transition_visits: usize,
    pub history_reconstruction_charge_bytes: usize,
    pub peak_history_reconstruction_charge_bytes: usize,
    pub incremental_advances: usize,
    pub incremental_evidence_bytes: u64,
    pub incremental_replays: usize,
    pub incremental_misses: usize,
    pub evictions: usize,
    pub denials: usize,
    pub releases: usize,
    pub retained_charge_bytes: usize,
    pub maximum_retained_charge_bytes: usize,
}

impl WorthQueryWorkflowInstanceProgressCounters {
    pub const fn warm_hits(self) -> usize {
        self.warm_hits
    }
    pub const fn warm_core_hits(self) -> usize {
        self.warm_core_hits
    }
    pub const fn warm_history_transition_visits(self) -> usize {
        self.warm_history_transition_visits
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
    pub const fn history_reconstruction_charge_bytes(self) -> usize {
        self.history_reconstruction_charge_bytes
    }
    pub const fn peak_history_reconstruction_charge_bytes(self) -> usize {
        self.peak_history_reconstruction_charge_bytes
    }
    pub const fn incremental_advances(self) -> usize {
        self.incremental_advances
    }
    pub const fn incremental_evidence_bytes(self) -> u64 {
        self.incremental_evidence_bytes
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
    pub const fn retained_charge_bytes(self) -> usize {
        self.retained_charge_bytes
    }
    pub const fn maximum_retained_charge_bytes(self) -> usize {
        self.maximum_retained_charge_bytes
    }

    pub fn absorb(&mut self, shard: Self) {
        self.warm_hits = self.warm_hits.saturating_add(shard.warm_hits);
        self.warm_core_hits = self.warm_core_hits.saturating_add(shard.warm_core_hits);
        self.warm_history_transition_visits = self
            .warm_history_transition_visits
            .saturating_add(shard.warm_history_transition_visits);
        self.cold_misses = self.cold_misses.saturating_add(shard.cold_misses);
        self.cold_retains = self.cold_retains.saturating_add(shard.cold_retains);
        self.history_reconstruction_charge_bytes = self
            .history_reconstruction_charge_bytes
            .saturating_add(shard.history_reconstruction_charge_bytes);
        self.peak_history_reconstruction_charge_bytes = self
            .peak_history_reconstruction_charge_bytes
            .max(shard.peak_history_reconstruction_charge_bytes);
        self.cold_reconstruction_transition_visits = self
            .cold_reconstruction_transition_visits
            .saturating_add(shard.cold_reconstruction_transition_visits);
        self.incremental_advances = self
            .incremental_advances
            .saturating_add(shard.incremental_advances);
        self.incremental_evidence_bytes = self
            .incremental_evidence_bytes
            .saturating_add(shard.incremental_evidence_bytes);
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
