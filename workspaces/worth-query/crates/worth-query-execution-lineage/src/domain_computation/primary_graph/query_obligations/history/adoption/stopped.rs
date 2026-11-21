//! Stopped branch-set DATA uses the same serialized Query publisher.
use super::*;
use worth_foundational::facade::CanonicalDigestId;
use worth_query_execution_obligations::facade::ledger_inspect;
impl QueryObligationHistory {
    pub fn retain_stopped_adoption(
        &self,
        request: &ExecutionRequest<'_, '_>,
        row: QueryStoppedBranchSetObligation,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        if let Some(prior) = self.state().journal.as_ref().and_then(|j| {
            ledger_inspect::adoption(&j.ledger)
                .stopped
                .get(&row.attempt)
        }) {
            return if prior == &row {
                Ok(())
            } else {
                Err(QueryObligationDenial::ObligationSubjectMismatch)
            };
        }
        driver.publish(
            medium.as_ref(),
            QueryAdoptionRecord::RetainStopped(row),
            request,
        )
    }
    pub fn release_stopped_adoption(
        &self,
        request: &ExecutionRequest<'_, '_>,
        attempt: DurableCompositePublicationAttemptIdentity,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        if !self.state().journal.as_ref().is_some_and(|j| {
            ledger_inspect::adoption(&j.ledger)
                .stopped
                .contains_key(&attempt)
        }) {
            return Err(QueryObligationDenial::UnknownObligation);
        }
        driver.publish(
            medium.as_ref(),
            QueryAdoptionRecord::ReleaseStopped { attempt },
            request,
        )
    }
    pub fn stopped_adoptions(&self) -> Vec<QueryStoppedBranchSetObligation> {
        self.state()
            .journal
            .as_ref()
            .map(|j| {
                ledger_inspect::adoption(&j.ledger)
                    .stopped
                    .values()
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn rostered_revision(
        &self,
        revision: CanonicalDigestId,
    ) -> Result<ApplicationProgramRevision, QueryObligationDenial> {
        self.state()
            .entries
            .keys()
            .find(|known| known.as_bytes() == revision.bytes())
            .copied()
            .ok_or(QueryObligationDenial::UnrosteredProgram(revision))
    }
}
