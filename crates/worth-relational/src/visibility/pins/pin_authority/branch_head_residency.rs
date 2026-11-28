use crate::history::retention::VisibilitySnapshotBasis;
use crate::visibility::cache_state::{
    evict_cache_if_needed, maybe_remove_unprotected_state, protect_branch_head_state,
};

use super::*;

impl<'runtime> VisibilityPinAuthority<'runtime> {
    pub(crate) fn rebuild_branch_head_visibility_residency(
        &self,
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    ) {
        let tracked_states = self.runtime.visibility.tracked_branch_head_states();
        self.runtime
            .visibility
            .clear_branch_head_residency(&tracked_states);
        for key in &tracked_states {
            maybe_remove_unprotected_state(self.runtime, key);
        }
        if !self.runtime.protect_branch_heads() {
            evict_cache_if_needed(self.runtime);
            return;
        }
        let branch_ids = self.runtime.history.branch_ids_snapshot();
        for branch_id in branch_ids {
            let Some(version_id) = self
                .runtime
                .history()
                .branch_head(&branch_id)
                .map(|head| head.version_id)
            else {
                continue;
            };
            let Some(basis) = self
                .runtime
                .capture_optional_current_visibility_basis(&branch_id, version_id)
            else {
                continue;
            };
            protect_branch_head_state(self.runtime, basis.key());
            self.runtime.services.instrumentation.count(|counters| {
                counters.visibility_cache_branch_head_promotions += 1;
            });
        }
        evict_cache_if_needed(self.runtime);
    }

    pub(crate) fn move_branch_head_visibility_residency(
        &self,
        branch_id: &crate::history::data::BranchId,
        previous_head: Option<crate::identity::data::VersionId>,
        next_head: Option<crate::identity::data::VersionId>,
        next_basis: Option<&VisibilitySnapshotBasis>,
    ) {
        crate::branch::head_visibility_inputs::BranchHeadVisibilityInputs::from(self.runtime)
            .move_residency(branch_id, previous_head, next_head, next_basis);
    }
}
