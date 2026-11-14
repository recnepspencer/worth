//! Stopped branch-set DATA uses the same serialized Query publisher.
use super::*;
impl QueryObligationHistory {
    pub(in crate::domain_computation::primary_graph) fn retain_stopped_adoption(
        &self,
        request: &ExecutionRequest<'_, '_>,
        row: QueryStoppedBranchSetObligation,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        if let Some(prior) = self
            .state()
            .journal
            .as_ref()
            .and_then(|j| j.ledger.adoption.stopped.get(&row.attempt))
        {
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
    pub(in crate::domain_computation::primary_graph) fn release_stopped_adoption(
        &self,
        request: &ExecutionRequest<'_, '_>,
        attempt: DurableCompositePublicationAttemptIdentity,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        if !self
            .state()
            .journal
            .as_ref()
            .is_some_and(|j| j.ledger.adoption.stopped.contains_key(&attempt))
        {
            return Err(QueryObligationDenial::UnknownObligation);
        }
        driver.publish(
            medium.as_ref(),
            QueryAdoptionRecord::ReleaseStopped { attempt },
            request,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn stopped_adoptions(
        &self,
    ) -> Vec<QueryStoppedBranchSetObligation> {
        self.state()
            .journal
            .as_ref()
            .map(|j| j.ledger.adoption.stopped.values().cloned().collect())
            .unwrap_or_default()
    }
    pub(in crate::domain_computation::primary_graph) fn rostered_revision(
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
