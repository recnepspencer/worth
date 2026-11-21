//! A successor round re-derives from its chain's own-output view; any other
//! read of a key an earlier round wrote is that round's effect, not C (§1.5).
use super::*;
use crate::domain_computation::primary_graph::application_attempt::fact::touch_key;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as ObservedFact;
use crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationDecisionFact as Fact;
use worth_relational::facade::mvcc::ValidatedMutationTouch as Touch;

impl super::super::WorthQueryPrimaryGraphProvider {
    /// The local create coordinate of `session`'s original idempotency record.
    pub(in crate::domain_computation::primary_graph) fn coupled_writer_original_key(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> String {
        let attempts = self
            .attempts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let attempt = attempts
            .attempt_state(session)
            .expect("actual coupled writer")
            .attempt();
        super::super::idempotency::original_client_key(attempt.idempotency())
    }

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
            let fact = std::ops::Deref::deref(&fact);
            let record = touch_key::fact_record(fact);
            if record.is_some_and(|record| own.covers(record)) {
                continue;
            }
            // Native's exclusive set holds every entity the chain wrote
            // (`IntervenedOutput` otherwise), so an entity-local touch only
            // ever changed a covered record: a read of another single record,
            // or a selection the overlay merged, cannot see it.
            let overlaid =
                record.is_some() || matches!(fact, ObservedFact::IndexedEntitySelection { .. });
            if prior.iter().any(|touch| {
                !(overlaid && entity_local(touch)) && touch_key::touch_changes(touch, fact)
            }) {
                return Some(record);
            }
        }
        None
    }
}

/// A touch that writes only the entity it creates or changes.
fn entity_local(touch: &Touch) -> bool {
    matches!(
        touch,
        Touch::CreateEntity { .. } | Touch::WriteEntityField { .. }
    )
}
