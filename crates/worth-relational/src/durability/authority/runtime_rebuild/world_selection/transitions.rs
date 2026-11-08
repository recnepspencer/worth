//! Restore a Released predecessor only when the next exact transition needs it.
use super::*;
use crate::branch::{
    PositionedBranchTransition, RelationalBranchBasisPosture as Posture,
    RelationalBranchTransition as Transition,
};
impl WorldSelectionReplay {
    pub(in crate::durability::authority::runtime_rebuild) fn before_transition(
        &self,
        request: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        plan: &RecoveryPlan,
        roots: &mut RecoveredRootInventory,
        positioned: &PositionedBranchTransition,
    ) -> Result<(), Stop> {
        let transition = positioned.transition();
        let branch = match transition {
            Transition::Fork { source, .. } => source,
            Transition::Archive { .. }
            | Transition::DeletionRequested { .. }
            | Transition::Delete { .. } => transition.branch(),
            Transition::RetainedFork { .. } => return Ok(()),
        };
        let Some(cell) = runtime.history.branch_cell(branch) else {
            return Ok(());
        };
        let Some(root) = cell.root() else {
            return Ok(());
        };
        let current = crate::branch::descriptor_for_cell(&cell, &root)
            .map_err(|_| corrupt("Native transition source cannot be described"))?;
        if matches_transition(&current, transition) {
            return Ok(());
        }
        let Some(index) = root
            .commit_id()
            .and_then(|commit| self.effects.get(&commit))
        else {
            return Ok(());
        };
        let original = &self.admitted.source().originals()[*index];
        match original.fate() {
            WorldOriginalFate::Released => {}
            WorldOriginalFate::Open
            | WorldOriginalFate::Continued
            | WorldOriginalFate::Published => return Ok(()),
        }
        if original.receipt().map(|receipt| &receipt.branch_id) != Some(branch) {
            return Ok(());
        }
        let source_matches_branch = match original.source() {
            Source::Recorded(source) => source.descriptor().branch_id() == branch,
            Source::Published(coordinates) => {
                &self
                    .entry(coordinates.commit(), runtime, plan, request)?
                    .commit
                    .branch_id
                    == branch
            }
        };
        if !source_matches_branch {
            return Err(corrupt(
                "released transition predecessor belongs to another branch",
            ));
        }
        // The same owner validates the actual original receipt/body and exact
        // pre/post axes. Core transition replay still requires its full input.
        self.select(request, runtime, plan, roots, original.source(), false)
    }
}
fn matches_transition(current: &RelationalBranchBasisDescriptor, transition: &Transition) -> bool {
    let generation = current.reference().generation();
    let truth = current.truth_version();
    match transition {
        Transition::Fork {
            source_generation,
            source_truth_version,
            source_head,
            ..
        } => {
            current.posture() == Posture::Live
                && generation == *source_generation
                && truth == *source_truth_version
                && current.reference().target().as_basis().is_some_and(|head| {
                    head.selected_commit_id() == source_head.selected_commit_id()
                        && head.version_id() == source_head.version_id()
                        && head.parent_commit_ids() == source_head.parent_commit_ids()
                        && head.roots() == source_head.roots()
                })
        }
        Transition::Archive {
            prior_generation,
            prior_truth_version,
            ..
        } => {
            current.posture() == Posture::Live
                && generation == *prior_generation
                && truth == *prior_truth_version
        }
        Transition::DeletionRequested {
            prior_generation,
            prior_truth_version,
            ..
        } => generation == *prior_generation && truth == *prior_truth_version,
        Transition::Delete {
            prior_generation,
            prior_truth_version,
            head_commit,
            ..
        } => {
            generation == *prior_generation
                && truth == *prior_truth_version
                && current
                    .reference()
                    .target()
                    .as_basis()
                    .map(|head| crate::history::data::CommitId(head.selected_commit_id()))
                    == *head_commit
        }
        Transition::RetainedFork { .. } => false,
    }
}
