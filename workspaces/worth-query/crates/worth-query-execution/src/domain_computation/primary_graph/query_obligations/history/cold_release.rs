//! The ended incarnation's cold releases, retry-safe before service.
use super::*;
use journal::*;
use worth_execution::ExecutionRequest;

impl super::journal_publication::QueryObligationPublication<'_> {
    /// Publishes a cold release for every row the ended incarnation still
    /// holds. What remains is recomputed from the admitted ledger on every
    /// entry, so a retry after a refused, no-effect or interrupted publication
    /// finishes the rest and never repeats one. The witness is dropped only
    /// once nothing remains; until then no admission of this journal succeeds.
    pub(super) fn publish_cold_releases(
        &self,
        medium: &dyn QueryObligationsMedium,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryObligationDenial> {
        self.finish_pending(medium, request)?;
        let remaining = {
            let state = self.history().state();
            let journal = state
                .journal
                .as_ref()
                .ok_or(QueryObligationDenial::SupportUnavailable)?;
            let Some(ended) = &journal.ended_incarnation else {
                return Ok(());
            };
            ended.pending_cold_releases(&journal.ledger)?
        };
        for release in remaining {
            self.publish(medium, release, request)?;
        }
        if let Some(journal) = self.history().state().journal.as_mut() {
            journal.ended_incarnation = None;
        }
        Ok(())
    }
}
