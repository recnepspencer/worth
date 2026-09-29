//! Fair bounded selection from already admitted owner custody.

use std::ops::Bound::{Excluded, Unbounded};

use super::{PublicationState, WorthQueryInboundCustody};
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;

impl WorthQueryInboundCustody {
    /// Select at most `maximum_work` pending correlations for this operation.
    /// The cursor rotates through blocked entries without scanning another
    /// operation's custody or settled terminals.
    pub(in crate::domain_computation) fn maintenance_candidates(
        &mut self,
        operation: &str,
        maximum_work: usize,
    ) -> Vec<ExternalEffectCorrelationIdentity> {
        let mut candidates = Vec::new();
        let Some(pending) = self.pending_by_operation.get(operation) else {
            return candidates;
        };
        let maximum_work = maximum_work.min(pending.len());
        for _ in 0..maximum_work {
            let next = self
                .maintenance_cursor_by_operation
                .get(operation)
                .copied()
                .and_then(|cursor| pending.range((Excluded(cursor), Unbounded)).next().copied())
                .or_else(|| pending.first().copied());
            let Some(correlation) = next else { break };
            self.maintenance_cursor_by_operation
                .insert(operation.to_owned(), correlation);
            candidates.push(correlation);
        }
        candidates
    }

    pub(in crate::domain_computation) fn pending_work_count(&self, operation: &str) -> usize {
        self.pending_by_operation
            .get(operation)
            .map_or(0, |pending| pending.len())
    }

    pub(in crate::domain_computation) fn is_retryable_correlation(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> bool {
        self.by_correlation
            .get(correlation)
            .and_then(|message| self.by_message.get(message))
            .is_some_and(|entry| entry.publication == PublicationState::Retryable)
    }

    pub(in crate::domain_computation) fn next_terminal_expiry(
        &self,
        operation: &str,
    ) -> Option<u64> {
        let settled = self
            .terminal_expiry
            .get(operation)
            .and_then(|by_expiry| by_expiry.first_key_value().map(|(expiry, _)| *expiry));
        let compact = self
            .compact_terminal_expiry
            .get(operation)
            .and_then(|by_expiry| by_expiry.first_key_value().map(|(expiry, _)| *expiry));
        match (settled, compact) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::CanonicalDigestId;

    use super::*;

    fn correlation(byte: u8) -> ExternalEffectCorrelationIdentity {
        ExternalEffectCorrelationIdentity::from_digest(CanonicalDigestId::new([byte; 32]))
    }

    #[test]
    fn one_item_batch_stays_with_its_operation_and_rotates_pending_work() {
        let mut custody = WorthQueryInboundCustody::default();
        custody
            .pending_by_operation
            .insert("unrelated".into(), [correlation(1)].into_iter().collect());
        custody.pending_by_operation.insert(
            "rail".into(),
            [correlation(2), correlation(3)].into_iter().collect(),
        );
        assert_eq!(
            custody.maintenance_candidates("rail", 1),
            vec![correlation(2)]
        );
        assert_eq!(
            custody.maintenance_candidates("rail", 1),
            vec![correlation(3)]
        );
        assert_eq!(
            custody.maintenance_candidates("rail", 1),
            vec![correlation(2)]
        );
        assert_eq!(
            custody.maintenance_candidates("unrelated", 1),
            vec![correlation(1)]
        );
        assert_eq!(custody.pending_work_count("rail"), 2);
    }
}
