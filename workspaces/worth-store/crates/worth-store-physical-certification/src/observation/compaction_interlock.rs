use worth_store_physical_isolation::CompactionInterlockFoundationalEvidence;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompactionInterlockObservation {
    no_mixed_root: bool,
    old_reachability_deferred: bool,
    post_cutover_plan_matches_publication: bool,
    blocked_reclaim_until_release: bool,
    protected_ranges: u64,
    candidate_ranges: u64,
    range_comparisons: u64,
    overlapping_ranges: u64,
    copied_pages: u64,
    publication_plan_completions: u64,
    blocked_reclaims: u64,
}

impl CompactionInterlockObservation {
    pub fn from_store_interlock_evidence(
        evidence: CompactionInterlockFoundationalEvidence,
    ) -> Option<Self> {
        if !evidence.materialized_after_store_decision() {
            return None;
        }
        let counters = evidence.counters();
        if counters.candidate_ranges() == 0
            || counters.copied_pages() == 0
            || counters.publication_plan_completions() == 0
        {
            return None;
        }
        Some(Self {
            no_mixed_root: evidence.no_mixed_root(),
            old_reachability_deferred: evidence.old_reachability_deferred(),
            post_cutover_plan_matches_publication: evidence.post_cutover_plan_matches_publication(),
            blocked_reclaim_until_release: evidence.blocked_reclaim_until_release(),
            protected_ranges: counters.protected_ranges(),
            candidate_ranges: counters.candidate_ranges(),
            range_comparisons: counters.range_comparisons(),
            overlapping_ranges: counters.overlapping_ranges(),
            copied_pages: counters.copied_pages(),
            publication_plan_completions: counters.publication_plan_completions(),
            blocked_reclaims: counters.blocked_reclaims(),
        })
    }

    pub const fn no_mixed_root(self) -> bool {
        self.no_mixed_root
    }

    pub const fn old_reachability_deferred(self) -> bool {
        self.old_reachability_deferred
    }

    pub const fn post_cutover_plan_matches_publication(self) -> bool {
        self.post_cutover_plan_matches_publication
    }

    pub const fn blocked_reclaim_until_release(self) -> bool {
        self.blocked_reclaim_until_release
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
}
