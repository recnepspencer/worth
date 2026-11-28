//! Original performed and checkpoint catalogs each retain their own cursor.
pub use super::performed_stream::RelationalPerformedSourceWalk;
use super::*;
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
            Arc<worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>,
        )>,
        (),
    >{
        let catalog =
            worth_execution::try_retirement_guard(self.checkpoint_catalog.try_read(), ())?;
        Ok(catalog
            .next_retirement(after)
            .map(|(key, artifact)| (key, artifact.envelope_residency())))
    }
}
