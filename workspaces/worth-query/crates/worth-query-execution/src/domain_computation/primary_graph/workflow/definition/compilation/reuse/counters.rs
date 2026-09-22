#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    pub(super) const fn new(
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
