//! Actual source images remain owned until the same Home reconciles World.
use super::*;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_proof::{
    CompletedHomeWorldSelection, HomeWorldSelectionAdmissionDenial as Denial,
    HomeWorldSelectionAuthority,
};

#[derive(Debug, Default)]
pub(super) struct WorldSelectionHandoff {
    authority: Option<HomeWorldSelectionAuthority>,
    roots: ExecutionOrderedMap<Option<CommitId>, Arc<RelationalBranchRoot>>,
}
impl RecoveredRootInventory {
    pub(in crate::durability::authority::runtime_rebuild) fn retain_world_selection_root(
        &mut self,
        commit: Option<CommitId>,
        root: Arc<RelationalBranchRoot>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::durability::data::RelationalRecoveryStop> {
        self.world_handoff
            .get_mut()
            .unwrap_or_else(|error| error.into_inner())
            .roots
            .insert_cloned_native_for_request(&commit, &root, *request)
            .map_err(super::super::world_selection::ordered)?;
        Ok(())
    }
    pub(in crate::durability::authority::runtime_rebuild) fn install_world_selection_handoff(
        &mut self,
        authority: HomeWorldSelectionAuthority,
    ) {
        self.world_handoff
            .get_mut()
            .unwrap_or_else(|error| error.into_inner())
            .authority = Some(authority);
    }
    pub(in crate::durability::authority) fn complete_world_selection_handoff(
        &self,
        completed: CompletedHomeWorldSelection,
    ) -> Result<(), Denial> {
        let mut handoff = self
            .world_handoff
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        handoff
            .authority
            .as_ref()
            .ok_or(Denial::ForeignOwner)?
            .admit_completion(&completed)?;
        let roots = std::mem::take(&mut handoff.roots);
        handoff.authority = None;
        drop(handoff);
        drop(roots);
        Ok(())
    }
}
