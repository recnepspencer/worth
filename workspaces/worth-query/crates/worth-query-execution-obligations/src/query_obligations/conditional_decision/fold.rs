//! Original-key membership and cleanup association are folded by their family.
use super::super::{QueryObligationDenial, QueryObligationLedger};
use super::*;
impl QueryObligationLedger {
    pub(in crate::query_obligations) fn conditional_decision(
        &self,
        key: &[u8; 32],
    ) -> Option<&QueryConditionalDecision> {
        self.conditional.get(key)
    }
    pub fn conditional_decisions(&self) -> impl Iterator<Item = &QueryConditionalDecision> {
        self.conditional.values()
    }
    pub(in crate::query_obligations) fn validate_conditional(
        &self,
        record: &QueryConditionalDecisionRecord,
    ) -> Result<(), QueryObligationDenial> {
        let (key, intent) = match record {
            QueryConditionalDecisionRecord::Retain(row) => {
                if row.product.owner().home() != self.home() {
                    return Err(QueryObligationDenial::ForeignHome);
                }
                if let QueryConditionalPosture::CleanupRequired(identity) = row.posture {
                    if identity.home() != self.home() {
                        return Err(QueryObligationDenial::ForeignHome);
                    }
                    if !self
                        .managed_cleanup()
                        .any(|cleanup| cleanup.identity == identity)
                    {
                        return Err(QueryObligationDenial::UnknownObligation);
                    }
                }
                if let QueryConditionalPosture::Indeterminate(QueryConditionalOriginal::Recovered(
                    original,
                )) = row.posture
                {
                    if original.owner().home() != self.home() {
                        return Err(QueryObligationDenial::ForeignHome);
                    }
                    if original.owner() != row.product.owner() {
                        return Err(QueryObligationDenial::ObligationSubjectMismatch);
                    }
                }
                (&row.key, &row.intent)
            }
            QueryConditionalDecisionRecord::Finish { key, intent } => {
                if !self.conditional.contains_key(key) {
                    return Err(QueryObligationDenial::UnknownObligation);
                }
                (key, intent)
            }
        };
        if self
            .conditional
            .get(key)
            .is_some_and(|known| &known.intent != intent)
        {
            return Err(QueryObligationDenial::ObligationSubjectMismatch);
        }
        if let (Some(known), QueryConditionalDecisionRecord::Retain(row)) =
            (self.conditional.get(key), record)
            && (known.product != row.product
                || invalid_posture_transition(known.posture, row.posture))
        {
            return Err(QueryObligationDenial::ObligationSubjectMismatch);
        }
        if let Some(known) = self.conditional.get(key)
            && let QueryConditionalPosture::CleanupRequired(identity) = known.posture
            && self
                .managed_cleanup()
                .any(|cleanup| cleanup.identity == identity)
            && !matches!(record, QueryConditionalDecisionRecord::Retain(row) if row == known)
        {
            return Err(QueryObligationDenial::ConditionalCleanupPending {
                key: *key,
                identity,
            });
        }
        Ok(())
    }
    pub(in crate::query_obligations) fn apply_conditional(
        &mut self,
        record: QueryConditionalDecisionRecord,
    ) {
        match record {
            QueryConditionalDecisionRecord::Retain(row) => {
                self.conditional.insert(row.key, row);
            }
            QueryConditionalDecisionRecord::Finish { key, .. } => {
                self.conditional.remove(&key);
            }
        }
    }
}

/// An indeterminate row never drops back to an ordinary retry posture before
/// World resolves it; once joined to a recovered World original, it stays
/// joined to exactly that original until finished.
fn invalid_posture_transition(
    known: QueryConditionalPosture,
    next: QueryConditionalPosture,
) -> bool {
    match known {
        QueryConditionalPosture::Indeterminate(QueryConditionalOriginal::Recovered(original)) => {
            next != QueryConditionalPosture::Indeterminate(QueryConditionalOriginal::Recovered(
                original,
            ))
        }
        QueryConditionalPosture::Indeterminate(QueryConditionalOriginal::Unclassified) => {
            !matches!(next, QueryConditionalPosture::Indeterminate(_))
        }
        QueryConditionalPosture::CleanupRequired(_)
        | QueryConditionalPosture::Retryable
        | QueryConditionalPosture::CommitRetryable
        | QueryConditionalPosture::ProductStale => false,
    }
}
