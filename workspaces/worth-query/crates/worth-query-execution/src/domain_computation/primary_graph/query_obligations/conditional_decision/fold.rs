//! Original-key membership and cleanup association are folded by their family.
use super::super::{QueryObligationDenial, QueryObligationLedger};
use super::*;
impl QueryObligationLedger {
    pub(in crate::domain_computation::primary_graph::query_obligations) fn conditional_decision(
        &self,
        key: &[u8; 32],
    ) -> Option<&QueryConditionalDecision> {
        self.conditional.get(key)
    }
    pub fn conditional_decisions(&self) -> impl Iterator<Item = &QueryConditionalDecision> {
        self.conditional.values()
    }
    pub(in crate::domain_computation::primary_graph::query_obligations) fn validate_conditional(
        &self,
        record: &QueryConditionalDecisionRecord,
    ) -> Result<(), QueryObligationDenial> {
        let (key, intent) = match record {
            QueryConditionalDecisionRecord::Retain(row) => {
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
    pub(in crate::domain_computation::primary_graph::query_obligations) fn apply_conditional(
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
