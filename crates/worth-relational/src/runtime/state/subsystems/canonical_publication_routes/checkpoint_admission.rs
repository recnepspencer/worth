use std::sync::{RwLockWriteGuard, TryLockError};

use crate::history::data::CommitId;

use super::{
    CanonicalCheckpointAdmissionError, PerformedCheckpointSelection,
    RelationalCanonicalPublicationRoutes,
};

#[cfg(test)]
mod tests {
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

impl RelationalCanonicalPublicationRoutes {
    pub(crate) fn with_settled_publication_snapshot<R>(
        &self,
        reclaim: impl FnOnce() -> R,
    ) -> Option<R> {
        let _admission = self.enter_checkpoint_selection().ok()?;
        self.reject_unsettled().ok()?;
        #[cfg(test)]
        self.lifecycle_order.pause_exclusive();
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

    pub(crate) fn has_unsettled_performed_publication(&self) -> Option<CommitId> {
        self.pending_settlement
            .first_unsettled_commit(&self.settlement_route_visits)
    }

    pub(crate) fn settlement_route_visit_count(&self) -> u64 {
        self.settlement_route_visits
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub(crate) fn checkpoint_selection(
        &self,
    ) -> Result<PerformedCheckpointSelection, CanonicalCheckpointAdmissionError> {
        let _admission = self.enter_checkpoint_selection()?;
        self.reject_unsettled()?;
        Ok(self
            .performed_stream
            .checkpoint_selection(self.branch_transition_head()))
    }

    pub(crate) fn validate_checkpoint_selection(
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
    ) -> Result<RwLockWriteGuard<'_, ()>, crate::runtime::RelationalRuntimeForkDenial> {
        let guard = match self.lifecycle.try_write() {
            Ok(guard) => Ok(guard),
            Err(TryLockError::WouldBlock) => {
                Err(crate::runtime::RelationalRuntimeForkDenial::PublicationInFlight)
            }
            Err(TryLockError::Poisoned(poisoned)) => Ok(poisoned.into_inner()),
        }?;
        #[cfg(test)]
        self.lifecycle_order.pause_exclusive();
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
