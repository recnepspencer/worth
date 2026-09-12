#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CompactionReadInterlockCounters {
    protected_ranges: u64,
    candidate_ranges: u64,
    range_comparisons: u64,
    overlapping_ranges: u64,
    copied_pages: u64,
    publication_plan_completions: u64,
    blocked_reclaims: u64,
    epoch_retries: u64,
    denied_in_place_overwrites: u64,
    denied_early_reclaims: u64,
    denied_stale_epoch_reuse: u64,
    denied_backend_residue: u64,
}

impl CompactionReadInterlockCounters {
    pub(crate) const fn from_range_intersection(
        protected_ranges: u64,
        candidate_ranges: u64,
        range_comparisons: u64,
        overlapping_ranges: u64,
        copied_pages: u64,
    ) -> Self {
        Self {
            protected_ranges,
            candidate_ranges,
            range_comparisons,
            overlapping_ranges,
            copied_pages,
            publication_plan_completions: 0,
            blocked_reclaims: 0,
            epoch_retries: 0,
            denied_in_place_overwrites: 0,
            denied_early_reclaims: 0,
            denied_stale_epoch_reuse: 0,
            denied_backend_residue: 0,
        }
    }

    pub(crate) const fn with_publication_plan_completion(mut self) -> Self {
        self.publication_plan_completions += 1;
        self
    }

    pub(crate) const fn with_blocked_reclaim(mut self) -> Self {
        self.blocked_reclaims += 1;
        self
    }

    pub(crate) const fn with_denied_in_place_overwrite(mut self) -> Self {
        self.denied_in_place_overwrites += 1;
        self
    }

    pub(crate) const fn with_denied_early_reclaim(mut self) -> Self {
        self.denied_early_reclaims += 1;
        self
    }

    pub const fn protected_ranges(self) -> u64 {
        self.protected_ranges
    }

    pub const fn candidate_ranges(self) -> u64 {
        self.candidate_ranges
    }

    pub const fn range_comparisons(self) -> u64 {
        self.range_comparisons
    }

    pub const fn overlapping_ranges(self) -> u64 {
        self.overlapping_ranges
    }

    pub const fn copied_pages(self) -> u64 {
        self.copied_pages
    }

    pub const fn publication_plan_completions(self) -> u64 {
        self.publication_plan_completions
    }

    pub const fn blocked_reclaims(self) -> u64 {
        self.blocked_reclaims
    }

    pub const fn epoch_retries(self) -> u64 {
        self.epoch_retries
    }

    pub const fn denied_in_place_overwrites(self) -> u64 {
        self.denied_in_place_overwrites
    }

    pub const fn denied_early_reclaims(self) -> u64 {
        self.denied_early_reclaims
    }

    pub const fn denied_stale_epoch_reuse(self) -> u64 {
        self.denied_stale_epoch_reuse
    }

    pub const fn denied_backend_residue(self) -> u64 {
        self.denied_backend_residue
    }
}
