use std::sync::{RwLockWriteGuard, TryLockError};

use crate::history::data::CommitId;

use super::{
    CanonicalCheckpointAdmissionError, PerformedCheckpointSelection,
    RelationalCanonicalPublicationRoutes,
};

impl RelationalCanonicalPublicationRoutes {
    pub(crate) fn with_quiescent_index_reclamation<R>(
        &self,
        reclaim: impl FnOnce() -> R,
    ) -> Option<R> {
        let _admission = self.enter_checkpoint_selection().ok()?;
        self.reject_unsettled().ok()?;
        Some(reclaim())
    }

    pub(crate) fn mark_settled(&self, commit_id: CommitId) -> bool {
        self.by_commit.get(&commit_id).is_some_and(|route| {
            route.mark_settled();
            true
        })
    }

    pub(crate) fn has_unsettled_performed_publication(&self) -> Option<CommitId> {
        self.performed_stream.first_unsettled_commit()
    }

    pub(crate) fn checkpoint_selection(
        &self,
    ) -> Result<PerformedCheckpointSelection, CanonicalCheckpointAdmissionError> {
        let _admission = self.enter_checkpoint_selection()?;
        self.reject_unsettled()?;
        Ok(self.performed_stream.checkpoint_selection())
    }

    pub(crate) fn validate_checkpoint_selection(
        &self,
        selection: &PerformedCheckpointSelection,
    ) -> Result<(), CanonicalCheckpointAdmissionError> {
        let _admission = self.enter_checkpoint_selection()?;
        self.reject_unsettled()?;
        self.performed_stream
            .still_selects(selection)
            .then_some(())
            .ok_or(CanonicalCheckpointAdmissionError::PublicationInFlight)
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
        match self.performed_stream.first_unsettled_commit() {
            Some(commit_id) => Err(
                CanonicalCheckpointAdmissionError::PerformedPublicationRequiresSettlement(
                    commit_id,
                ),
            ),
            None => Ok(()),
        }
    }
}
