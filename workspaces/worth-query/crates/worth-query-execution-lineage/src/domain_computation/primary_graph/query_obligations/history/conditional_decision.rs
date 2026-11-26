//! Conditional continuation uses the shared publication driver and original key.
use super::*;
use journal::{
    QueryConditionalDecision, QueryConditionalDecisionRecord, QueryConditionalOriginal,
    QueryConditionalPosture, QueryObligationDenial,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    DurableCompositePublicationAttemptIdentity, DurableProductBranchIncarnation,
};
use worth_query_execution_obligations::facade::ledger_inspect;

/// What an admitted conditional retry must do before any fresh execution.
pub enum QueryConditionalRetryAdmission {
    /// No joined original: the ordinary commit path resolves the key itself.
    Ordinary,
    /// The key's indeterminate row is joined to this recovered World original;
    /// the retry continues or releases it and never runs fresh first.
    JoinedOriginal(DurableCompositePublicationAttemptIdentity),
}

impl QueryObligationHistory {
    pub fn has_medium(&self) -> bool {
        self.medium.get().is_some()
    }
    fn conditional_decision(&self, key: &[u8; 32]) -> Option<QueryConditionalDecision> {
        self.state()
            .journal
            .as_ref()
            .and_then(|journal| ledger_inspect::conditional_decision(&journal.ledger, key).cloned())
    }
    pub fn conditional_decisions(&self) -> Vec<QueryConditionalDecision> {
        self.state()
            .journal
            .as_ref()
            .map(|journal| journal.ledger.conditional_decisions().cloned().collect())
            .unwrap_or_default()
    }

    /// The original a live indeterminate answer keeps: a row the cold consumer
    /// already joined stays joined; otherwise World has not classified it.
    pub fn retained_indeterminate_original(&self, key: &[u8; 32]) -> QueryConditionalOriginal {
        match self.conditional_decision(key).map(|row| row.posture) {
            Some(QueryConditionalPosture::Indeterminate(original)) => original,
            Some(
                QueryConditionalPosture::CleanupRequired(_)
                | QueryConditionalPosture::Retryable
                | QueryConditionalPosture::CommitRetryable
                | QueryConditionalPosture::ProductStale,
            )
            | None => QueryConditionalOriginal::Unclassified,
        }
    }

    /// Whether a durable continuation row still governs `key`, so a fresh
    /// Home must consume it before the key's fate is settled.
    pub fn retains_conditional_decision(&self, key: &[u8; 32]) -> bool {
        self.conditional_decision(key).is_some()
    }

    pub fn is_indeterminate(&self, key: &[u8; 32]) -> bool {
        matches!(
            self.conditional_decision(key).map(|row| row.posture),
            Some(QueryConditionalPosture::Indeterminate(_))
        )
    }

    pub fn admit_conditional_retry(
        &self,
        request: &ExecutionRequest<'_, '_>,
        key: [u8; 32],
        intent: [u8; 32],
        product: Option<DurableProductBranchIncarnation>,
    ) -> Result<QueryConditionalRetryAdmission, QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(QueryConditionalRetryAdmission::Ordinary);
        };
        self.lock_publication()?
            .finish_pending(medium.as_ref(), request)?;
        let Some(row) = self.conditional_decision(&key) else {
            return Ok(QueryConditionalRetryAdmission::Ordinary);
        };
        if row.intent != intent || product.is_some_and(|expected| row.product != expected) {
            return Err(QueryObligationDenial::ObligationSubjectMismatch);
        }
        match row.posture {
            QueryConditionalPosture::CleanupRequired(identity)
                if self
                    .managed_cleanup_obligations()
                    .iter()
                    .any(|cleanup| cleanup.identity == identity) =>
            {
                Err(QueryObligationDenial::ConditionalCleanupPending { key, identity })
            }
            QueryConditionalPosture::Indeterminate(QueryConditionalOriginal::Recovered(
                original,
            )) => Ok(QueryConditionalRetryAdmission::JoinedOriginal(original)),
            QueryConditionalPosture::CleanupRequired(_)
            | QueryConditionalPosture::Retryable
            | QueryConditionalPosture::CommitRetryable
            | QueryConditionalPosture::ProductStale
            | QueryConditionalPosture::Indeterminate(QueryConditionalOriginal::Unclassified) => {
                Ok(QueryConditionalRetryAdmission::Ordinary)
            }
        }
    }

    pub fn record_conditional_decision(
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
