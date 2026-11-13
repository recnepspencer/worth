//! Conditional continuation uses the shared publication driver and original key.
use super::*;
use journal::{
    QueryConditionalDecision, QueryConditionalDecisionRecord, QueryConditionalPosture,
    QueryObligationDenial,
};
use worth_execution::ExecutionRequest;

impl QueryObligationHistory {
    fn conditional_decision(&self, key: &[u8; 32]) -> Option<QueryConditionalDecision> {
        self.state()
            .journal
            .as_ref()
            .and_then(|journal| journal.ledger.conditional_decision(key).cloned())
    }
    pub(in crate::domain_computation::primary_graph) fn conditional_decisions(
        &self,
    ) -> Vec<QueryConditionalDecision> {
        self.state()
            .journal
            .as_ref()
            .map(|journal| journal.ledger.conditional_decisions().cloned().collect())
            .unwrap_or_default()
    }

    pub(in crate::domain_computation::primary_graph) fn admit_conditional_retry(
        &self,
        request: &ExecutionRequest<'_, '_>,
        key: [u8; 32],
        intent: [u8; 32],
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        self.lock_publication()?
            .finish_pending(medium.as_ref(), request)?;
        let Some(row) = self.conditional_decision(&key) else {
            return Ok(());
        };
        if row.intent != intent {
            return Err(QueryObligationDenial::ObligationSubjectMismatch);
        }
        if let QueryConditionalPosture::CleanupRequired(identity) = row.posture
            && self
                .managed_cleanup_obligations()
                .iter()
                .any(|cleanup| cleanup.identity == identity)
        {
            return Err(QueryObligationDenial::ConditionalCleanupPending { key, identity });
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn record_conditional_decision(
        &self,
        request: &ExecutionRequest<'_, '_>,
        record: QueryConditionalDecisionRecord,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        match &record {
            QueryConditionalDecisionRecord::Retain(row)
                if self.conditional_decision(&row.key).as_ref() == Some(row) =>
            {
                return Ok(())
            }
            QueryConditionalDecisionRecord::Finish { key, .. }
                if self.conditional_decision(key).is_none() =>
            {
                return Ok(())
            }
            QueryConditionalDecisionRecord::Retain(_)
            | QueryConditionalDecisionRecord::Finish { .. } => {}
        }
        driver.publish(medium.as_ref(), record, request)
    }
}
