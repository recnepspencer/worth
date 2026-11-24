use std::sync::{RwLockWriteGuard, TryLockError};
use worth_relational_contracts::internal_port as contracts;

use contracts::history::data::coordinates::CommitId;

use super::PerformedCheckpointSelection;
use super::RelationalCanonicalPublicationRoutes;
use worth_relational_durability::internal_port::durability::checkpoint_selection_denial::CanonicalCheckpointAdmissionError;

impl RelationalCanonicalPublicationRoutes {
    pub fn with_settled_publication_snapshot<R>(&self, reclaim: impl FnOnce() -> R) -> Option<R> {
        let _admission = self.enter_checkpoint_selection().ok()?;
        self.reject_unsettled().ok()?;
        if let Some(control) = self.lifecycle_order.get() {
            control.pause_exclusive();
        }
        Some(reclaim())
    }

    pub(crate) fn mark_settled(&self, commit_id: CommitId) -> bool {
        let Some(route) = self
            .by_commit
            .get(&commit_id)
            .map(|route| std::sync::Arc::clone(route.value()))
        else {
            return false;
        };
        route.mark_settled();
        self.pending_settlement.withdraw(&route);
        true
    }

    pub fn has_unsettled_performed_publication(&self) -> Option<CommitId> {
        self.pending_settlement
            .first_unsettled_commit(&self.settlement_route_visits)
    }

    pub fn settlement_route_visit_count(&self) -> u64 {
        self.settlement_route_visits
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn checkpoint_selection(
        &self,
    ) -> Result<PerformedCheckpointSelection, CanonicalCheckpointAdmissionError> {
        self.with_checkpoint_selection(|selection| selection)
    }

    /// Snapshot all Complete producer inputs under the SAME existing epoch.
    /// The callback must not reacquire this publication gate.
    pub fn with_checkpoint_selection<R>(
        &self,
        capture: impl FnOnce(PerformedCheckpointSelection) -> R,
    ) -> Result<R, CanonicalCheckpointAdmissionError> {
        let _admission = self.enter_checkpoint_selection()?;
        self.reject_unsettled()?;
        // The admitted checkpoint catalog owns receipt/position facts even
        // when no covered original envelope has ever become resident. This
        // does not create a performed route or settle a recovered effect.
        let indexed_head = self
            .checkpoint_catalog
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .head
            .clone();
        Ok(capture(self.performed_stream.checkpoint_selection(
            self.branch_transition_head(),
            indexed_head,
        )))
    }

    pub fn validate_checkpoint_selection(
        &self,
        selection: &PerformedCheckpointSelection,
    ) -> Result<(), CanonicalCheckpointAdmissionError> {
        let _admission = self.enter_checkpoint_selection()?;
        self.reject_unsettled()?;
        (self.performed_stream.still_selects(selection)
            && self.branch_transition_head() == selection.covered_transitions())
        .then_some(())
        .ok_or(CanonicalCheckpointAdmissionError::PublicationInFlight)
    }

    /// Exclude publication for a runtime fork's snapshot, as a checkpoint
    /// selection does.
    pub(crate) fn enter_fork(
        &self,
    ) -> Result<
        RwLockWriteGuard<'_, ()>,
        contracts::runtime::fork_denial::RelationalRuntimeForkDenial,
    > {
        let guard = match self.lifecycle.try_write() {
            Ok(guard) => Ok(guard),
            Err(TryLockError::WouldBlock) => Err(
                contracts::runtime::fork_denial::RelationalRuntimeForkDenial::PublicationInFlight,
            ),
            Err(TryLockError::Poisoned(poisoned)) => Ok(poisoned.into_inner()),
        }?;
        if let Some(control) = self.lifecycle_order.get() {
            control.pause_exclusive();
        }
        Ok(guard)
    }

    fn enter_checkpoint_selection(
        &self,
    ) -> Result<RwLockWriteGuard<'_, ()>, CanonicalCheckpointAdmissionError> {
        match self.lifecycle.try_write() {
            Ok(guard) => Ok(guard),
            Err(TryLockError::WouldBlock) => {
                Err(CanonicalCheckpointAdmissionError::PublicationInFlight)
            }
            Err(TryLockError::Poisoned(poisoned)) => Ok(poisoned.into_inner()),
        }
    }

    fn reject_unsettled(&self) -> Result<(), CanonicalCheckpointAdmissionError> {
        match self
            .pending_settlement
            .first_unsettled_commit(&self.settlement_route_visits)
        {
            Some(commit_id) => Err(
                CanonicalCheckpointAdmissionError::PerformedPublicationRequiresSettlement(
                    commit_id,
                ),
            ),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::RelationalCanonicalPublicationRoutes;

    #[test]
    fn branch_transition_excludes_settled_owner_inventory_until_release() {
        let routes = RelationalCanonicalPublicationRoutes::default();
        let transition = routes.begin_branch_transition();
        assert_eq!(routes.with_settled_publication_snapshot(|| ()), None);
        drop(transition);
        assert_eq!(routes.with_settled_publication_snapshot(|| ()), Some(()));
    }
}
