//! Original performed and checkpoint catalogs each retain their own cursor.
pub(crate) use super::performed_stream::RelationalPerformedSourceWalk;
use super::*;
use std::ops::Bound::{Excluded, Unbounded};
impl RelationalCanonicalPublicationRoutes {
    pub(crate) fn source_retirement_walk(&self) -> RelationalPerformedSourceWalk {
        self.performed_stream.source_retirement_walk()
    }
    pub(crate) fn source_retirement_checkpoint_envelope(
        &self,
        after: Option<CommitId>,
    ) -> Result<
        Option<(
            CommitId,
            Arc<crate::history::RelationalCommitEnvelopeResidency>,
        )>,
        (),
    > {
        let catalog = match self.checkpoint_catalog.try_read() {
            Ok(catalog) => catalog,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(()),
        };
        let next = match after {
            None => catalog.first_key_value(),
            Some(key) => catalog.range((Excluded(key), Unbounded)).next(),
        };
        Ok(next.map(|(&key, artifact)| (key, artifact.envelope_residency())))
    }
}
