use super::{
    CompactionReadInterlockCounters, CompactionReadInterlockDenial, CompactionReadPlanCompletion,
    DrainedCompactionReclaim,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompactionInterlockFoundationalEvidence {
    counters: CompactionReadInterlockCounters,
    materialized_after_store_decision: bool,
    no_mixed_root: bool,
    old_reachability_deferred: bool,
    post_cutover_plan_matches_publication: bool,
    blocked_reclaim_until_release: bool,
}

impl CompactionInterlockFoundationalEvidence {
    pub(crate) const fn after_store_decision(counters: CompactionReadInterlockCounters) -> Self {
        Self {
            counters,
            materialized_after_store_decision: true,
            no_mixed_root: false,
            old_reachability_deferred: false,
            post_cutover_plan_matches_publication: false,
            blocked_reclaim_until_release: false,
        }
    }

    pub fn after_completed_plans_and_reclaim(
        completion: &CompactionReadPlanCompletion,
        reclaim: &DrainedCompactionReclaim,
    ) -> Result<Self, CompactionReadInterlockDenial> {
        if !reclaim.matches_publication(completion.publication()) {
            return Err(CompactionReadInterlockDenial::ReclaimPublicationMismatch);
        }
        Ok(Self {
            counters: reclaim.counters(),
            materialized_after_store_decision: true,
            no_mixed_root: completion.pre_cutover_root().epoch()
                != completion.post_cutover_root().epoch(),
            old_reachability_deferred: completion.old_reachability_deferred(),
            post_cutover_plan_matches_publication: true,
            blocked_reclaim_until_release: reclaim.counters().blocked_reclaims() > 0
                && reclaim.released().footprint_basis()
                    == completion
                        .pre_cutover_plan_completion()
                        .read_plan_release()
                        .footprint_basis(),
        })
    }

    pub const fn counters(self) -> CompactionReadInterlockCounters {
        self.counters
    }

    pub const fn materialized_after_store_decision(self) -> bool {
        self.materialized_after_store_decision
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
}
