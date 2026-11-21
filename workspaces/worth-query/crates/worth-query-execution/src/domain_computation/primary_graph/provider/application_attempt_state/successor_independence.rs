//! A successor round re-derives from its chain's own-output view; any other
//! read of a key an earlier round wrote is that round's effect, not C (§1.5).
use super::*;
use crate::domain_computation::primary_graph::application_attempt::fact::touch_key;
use crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationDecisionFact as Fact;

impl super::super::WorthQueryPrimaryGraphProvider {
    /// The first read of `session`'s writer, outside `own`, that a touch its
    /// chain's earlier rounds certified could change, with that read's record
    /// when it has one. `None` when the round is independent.
    pub(in crate::domain_computation::primary_graph) fn coupled_writer_prior_effect(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        own: &OwnOutputOverlay,
        prior: &[worth_relational::facade::mvcc::ValidatedMutationTouch],
    ) -> Option<Option<worth_relational::facade::identity::EntityId>> {
        let attempts = self
            .attempts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let attempt = attempts
            .attempt_state(session)
            .expect("actual successor writer before Native append")
            .attempt();
        for fact in attempt.facts().values() {
            let fact = match fact {
                Fact::Application { fact, .. }
                | Fact::ObservedSource { fact }
                | Fact::ApplicationObservedSource { fact, .. } => fact,
                Fact::Principal(_) | Fact::Authorization { .. } => continue,
            };
            let record = touch_key::fact_record(fact);
            if record.is_some_and(|record| own.covers(record)) {
                continue;
            }
            if prior.iter().any(|touch| touch_key::touch_changes(touch, fact)) {
                return Some(record);
            }
        }
        None
    }
}
