//! Visibility bookkeeping follows a branch owner's exact installed head.
use crate::branch::basis_observation_inputs::BranchObservationInputs;
use crate::runtime::state::subsystems::visibility::VisibilitySubsystem;
use crate::visibility::cache_state::{
    bump_visibility_ref, evict_cache_if_needed, protect_branch_head_state, VisibilityCacheInputs,
};
use crate::visibility::snapshot_states::VisibilitySnapshotBasis;

#[derive(Clone, Copy)]
pub(crate) struct BranchHeadVisibilityInputs<'a> {
    pub(crate) observations: BranchObservationInputs<'a>,
    pub(crate) directory: &'a VisibilitySubsystem,
    pub(crate) cache: VisibilityCacheInputs<'a>,
}

impl BranchHeadVisibilityInputs<'_> {
    pub(crate) fn move_residency(
        self,
        branch_id: &crate::history::data::BranchId,
        previous_head: Option<crate::identity::data::VersionId>,
        next_head: Option<crate::identity::data::VersionId>,
        next_basis: Option<&VisibilitySnapshotBasis>,
    ) {
        if !self.cache.policy.protect_branch_heads || previous_head == next_head {
            return;
        }
        if previous_head.is_some() {
            let previous_key = self.directory.untrack_branch_head_state(branch_id);
            if let Some(previous_key) = previous_key {
                bump_visibility_ref(self.cache, &previous_key, |residency| {
                    residency.branch_head_refs = residency.branch_head_refs.saturating_sub(1);
                });
            }
        }
        if let Some(version_id) = next_head {
            if let Some(basis) = next_basis
                .filter(|basis| basis.branch_id() == branch_id && basis.version_id() == version_id)
            {
                protect_branch_head_state(self.cache, basis.key());
                self.cache.instrumentation.count(|counters| {
                    counters.visibility_cache_branch_head_promotions += 1;
                });
            } else if let Some(basis) = self
                .observations
                .capture_current_visibility_basis(branch_id, version_id)
                .ok()
                .flatten()
            {
                protect_branch_head_state(self.cache, basis.key());
                self.cache.instrumentation.count(|counters| {
                    counters.visibility_cache_branch_head_promotions += 1;
                });
            }
        }
        evict_cache_if_needed(self.cache);
    }
}
