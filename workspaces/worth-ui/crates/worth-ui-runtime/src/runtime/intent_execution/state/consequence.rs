use std::sync::Arc;

use super::{
    UiFrameworkIntentConsequenceHandoffMarker, UiIntentConsequenceBasis,
    UiIntentConsequenceHandoff, UiIntentExecutionSlotPhase, UiIntentExecutionState,
    UiPreparedFrameworkIntentConsequence, UiPreparedIntentConsequenceBatch,
    UiSettledFrameworkIntentAttempt,
};
use crate::runtime::intent_execution::{
    UiIntentConsequenceHandle, UiIntentConsequenceRecovery, UiIntentConsequenceStop,
    UiIntentConsequenceStopReason,
};

pub(crate) struct UiIntentConsequenceCurrentnessContext<'state> {
    pub(crate) catalog: &'state crate::declaration::UiIntentCatalog,
    pub(crate) generation: &'state crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    pub(crate) mounted: &'state crate::mounting::WorthUiMountedSessionState,
}

pub(crate) enum UiIntentConsequenceBeginOutcome {
    Handoff(UiIntentConsequenceHandoff),
    Stopped(UiIntentConsequenceStop),
}

impl UiIntentExecutionState {
    pub(crate) fn begin_consequence(
        &mut self,
        handle: UiIntentConsequenceHandle,
        current: UiIntentConsequenceCurrentnessContext<'_>,
    ) -> UiIntentConsequenceBeginOutcome {
        if !self.exact_consequence(&handle) {
            return stopped(UiIntentConsequenceStopReason::StaleOrForeign, handle);
        }
        let (attempt, idempotency, handle_lease) = handle.into_parts();
        let slot = attempt.slot() as usize;
        let phase = self.slots[slot]
            .phase
            .take()
            .expect("exact consequence identity has a retained phase");
        let prepared = match phase {
            UiIntentExecutionSlotPhase::ConsequencePending(settled) => {
                if let Err(reason) = require_current(&settled.basis, current) {
                    self.slots[slot].phase =
                        Some(UiIntentExecutionSlotPhase::ConsequencePending(settled));
                    return stopped_from_parts(reason, attempt, idempotency, handle_lease);
                }
                prepare(settled)
            }
            UiIntentExecutionSlotPhase::ConsequenceReady(prepared) => {
                if let Err(reason) = require_current(&prepared.basis, current) {
                    self.slots[slot].phase =
                        Some(UiIntentExecutionSlotPhase::ConsequenceReady(prepared));
                    return stopped_from_parts(reason, attempt, idempotency, handle_lease);
                }
                prepared
            }
            _ => unreachable!("exact consequence identity excludes other phases"),
        };
        let mismatch = batch_mismatch(&prepared);
        if let Some(reason) = mismatch {
            self.slots[slot].phase = Some(UiIntentExecutionSlotPhase::ConsequenceReady(prepared));
            return stopped_from_parts(reason, attempt, idempotency, handle_lease);
        }
        let handoff = UiIntentConsequenceHandoff {
            slot,
            attempt,
            idempotency,
            consequence_lease: Arc::clone(&prepared.consequence_lease),
            basis: prepared.basis,
            batch: prepared.batch,
        };
        self.slots[slot].phase = Some(UiIntentExecutionSlotPhase::ConsequenceHandoff(
            UiFrameworkIntentConsequenceHandoffMarker {
                attempt,
                idempotency,
                consequence_lease: prepared.consequence_lease,
            },
        ));
        UiIntentConsequenceBeginOutcome::Handoff(handoff)
    }

    pub(crate) fn retry_consequence(
        &mut self,
        recovery: UiIntentConsequenceRecovery,
        current: UiIntentConsequenceCurrentnessContext<'_>,
    ) -> UiIntentConsequenceBeginOutcome {
        self.begin_consequence(recovery.into_handle(), current)
    }

    pub(crate) fn retain_consequence_handoff(
        &mut self,
        handoff: UiIntentConsequenceHandoff,
        reason: UiIntentConsequenceStopReason,
    ) -> UiIntentConsequenceStop {
        let slot = handoff.slot;
        self.require_exact_handoff(&handoff);
        self.slots[slot].phase = Some(UiIntentExecutionSlotPhase::ConsequenceReady(
            UiPreparedFrameworkIntentConsequence {
                attempt: handoff.attempt,
                idempotency: handoff.idempotency,
                consequence_lease: Arc::clone(&handoff.consequence_lease),
                basis: handoff.basis,
                batch: handoff.batch,
            },
        ));
        UiIntentConsequenceStop::new(
            reason,
            UiIntentConsequenceRecovery::from_parts(
                handoff.attempt,
                handoff.idempotency,
                handoff.consequence_lease,
            ),
        )
    }

    pub(crate) fn finish_consequence_handoff(&mut self, handoff: UiIntentConsequenceHandoff) {
        self.require_exact_handoff(&handoff);
        self.slots[handoff.slot].phase = None;
    }

    pub(crate) fn dispose_consequence_handoff(&mut self, handoff: UiIntentConsequenceHandoff) {
        self.finish_consequence_handoff(handoff);
    }

    fn exact_consequence(&self, handle: &UiIntentConsequenceHandle) -> bool {
        let Some(slot) = self.slots.get(handle.attempt().slot() as usize) else {
            return false;
        };
        slot.generation == handle.attempt().generation()
            && match slot.phase.as_ref() {
                Some(UiIntentExecutionSlotPhase::ConsequencePending(settled)) => {
                    settled.attempt == handle.attempt()
                        && settled.idempotency == handle.idempotency()
                        && Arc::ptr_eq(&settled.consequence_lease, handle.lease())
                }
                Some(UiIntentExecutionSlotPhase::ConsequenceReady(prepared)) => {
                    prepared.attempt == handle.attempt()
                        && prepared.idempotency == handle.idempotency()
                        && Arc::ptr_eq(&prepared.consequence_lease, handle.lease())
                }
                _ => false,
            }
    }

    fn require_exact_handoff(&self, handoff: &UiIntentConsequenceHandoff) {
        assert!(matches!(
            self.slots[handoff.slot].phase.as_ref(),
            Some(UiIntentExecutionSlotPhase::ConsequenceHandoff(marker))
                if marker.attempt == handoff.attempt
                    && marker.idempotency == handoff.idempotency
                    && Arc::ptr_eq(&marker.consequence_lease, &handoff.consequence_lease)
        ));
    }
}

fn prepare(settled: UiSettledFrameworkIntentAttempt) -> UiPreparedFrameworkIntentConsequence {
    let runtime_service = settled.outcome.runtime_service_destination();
    let consequences = settled.outcome.into_consequences();
    let (query_collection_change, query_projection) = consequences.into_parts();
    UiPreparedFrameworkIntentConsequence {
        attempt: settled.attempt,
        idempotency: settled.idempotency,
        consequence_lease: settled.consequence_lease,
        batch: UiPreparedIntentConsequenceBatch {
            runtime_service,
            mounted_posture: settled
                .basis
                .declaration
                .consequences()
                .includes_mounted_posture(),
            query_collection_change,
            query_projection,
        },
        basis: settled.basis,
    }
}

fn batch_mismatch(
    prepared: &UiPreparedFrameworkIntentConsequence,
) -> Option<UiIntentConsequenceStopReason> {
    let expected = prepared
        .basis
        .declaration
        .consequences()
        .query_collection_change();
    let observed_collection = prepared
        .batch
        .query_collection_change
        .as_ref()
        .map(|consequence| consequence.query_view_identity());
    let observed_projection = prepared
        .batch
        .query_projection
        .as_ref()
        .map(worth_ui_query_binding::UiProjectionObservation::projection_identity);
    if observed_collection.is_some() && observed_projection.is_some() {
        return Some(UiIntentConsequenceStopReason::MultipleQueryConsequences);
    }
    let observed = observed_collection.or(observed_projection);
    match (expected, observed) {
        (None, None) => None,
        (None, Some(observed)) => Some(UiIntentConsequenceStopReason::UndeclaredQueryConsequence {
            observed: observed.clone(),
        }),
        (Some(expected), None) => Some(
            UiIntentConsequenceStopReason::MissingDeclaredQueryConsequence {
                expected: expected.clone(),
            },
        ),
        (Some(expected), Some(observed)) if expected != observed => Some(
            UiIntentConsequenceStopReason::QueryConsequenceIdentityMismatch {
                expected: expected.clone(),
                observed: observed.clone(),
            },
        ),
        (Some(_), Some(_)) => None,
    }
}

fn require_current(
    basis: &UiIntentConsequenceBasis,
    current: UiIntentConsequenceCurrentnessContext<'_>,
) -> Result<(), UiIntentConsequenceStopReason> {
    if &basis.generation != current.generation {
        return Err(UiIntentConsequenceStopReason::ApplicationGenerationChanged);
    }
    basis
        .target_affinity
        .require_current(current.mounted)
        .map_err(UiIntentConsequenceStopReason::TargetChanged)?;
    if let Some(command_route) = basis.command_route.as_ref() {
        return match current
            .catalog
            .lookup_command(command_route.destination().intent())
        {
            Some((
                crate::declaration::UiIntentCatalogCommandRoute::Resolved { declaration },
                _,
            )) if declaration.as_ref() == basis.declaration.as_ref() => Ok(()),
            _ => Err(UiIntentConsequenceStopReason::ProductRouteChanged),
        };
    }
    match current
        .catalog
        .lookup(basis.graph_node, basis.declaration.interaction())
    {
        Some((
            crate::declaration::UiIntentCatalogResolvedRoute::Product { declaration, .. },
            _,
        )) if declaration.as_ref() == basis.declaration.as_ref() => Ok(()),
        _ => Err(UiIntentConsequenceStopReason::ProductRouteChanged),
    }
}

fn stopped(
    reason: UiIntentConsequenceStopReason,
    handle: UiIntentConsequenceHandle,
) -> UiIntentConsequenceBeginOutcome {
    let (attempt, idempotency, lease) = handle.into_parts();
    stopped_from_parts(reason, attempt, idempotency, lease)
}

fn stopped_from_parts(
    reason: UiIntentConsequenceStopReason,
    attempt: crate::runtime::intent_execution::UiIntentExecutionAttemptIdentity,
    idempotency: crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity,
    lease: Arc<crate::runtime::intent_execution::UiIntentConsequenceLease>,
) -> UiIntentConsequenceBeginOutcome {
    UiIntentConsequenceBeginOutcome::Stopped(UiIntentConsequenceStop::new(
        reason,
        UiIntentConsequenceRecovery::from_parts(attempt, idempotency, lease),
    ))
}

impl UiIntentConsequenceHandoff {
    pub(crate) const fn attempt(
        &self,
    ) -> crate::runtime::intent_execution::UiIntentExecutionAttemptIdentity {
        self.attempt
    }

    pub(crate) const fn idempotency(
        &self,
    ) -> crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity {
        self.idempotency
    }

    pub(crate) const fn includes_mounted_posture(&self) -> bool {
        self.batch.mounted_posture
    }

    pub(crate) const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.basis.graph_node
    }

    pub(crate) const fn target(
        &self,
    ) -> crate::runtime::interaction::UiPresentedInteractionTargetView {
        self.basis.target
    }

    pub(crate) fn interaction_family(&self) -> crate::capability::UiSemanticInteractionFamily {
        self.basis.declaration.interaction()
    }

    pub(crate) const fn authored_portal_declaration(
        &self,
    ) -> Option<worth_ui_dsl::UiPortalDeclarationId> {
        self.basis.portal_declaration
    }

    pub(crate) fn selection_option(
        &self,
    ) -> Option<&worth_ui_query_binding::UiProjectionOptionReference> {
        self.basis.selection_option.as_ref()
    }

    pub(crate) fn take_query_consequence(
        &mut self,
    ) -> Option<worth_ui_query_binding::WorthUiCollectionChangeConsequence> {
        self.batch.query_collection_change.take()
    }

    pub(crate) fn take_query_projection(
        &mut self,
    ) -> Option<worth_ui_query_binding::UiProjectionObservation> {
        self.batch.query_projection.take()
    }

    pub(crate) fn query_operation_live_reference(
        &self,
    ) -> Option<worth_ui_query_binding::WorthUiInstalledQueryBindingReference> {
        self.batch
            .query_collection_change
            .as_ref()
            .map(|consequence| consequence.operation_live_reference().clone())
    }

    pub(crate) fn consequence_count(&self) -> usize {
        usize::from(self.batch.mounted_posture)
            + usize::from(self.batch.query_collection_change.is_some())
            + usize::from(self.batch.query_projection.is_some())
    }

    pub(crate) const fn runtime_service_destination(
        &self,
    ) -> Option<crate::capability::UiIntentRuntimeServiceDestination> {
        self.batch.runtime_service
    }

    pub(crate) const fn command_route(
        &self,
    ) -> Option<&crate::runtime::command_routing::UiCommandRouteEvidence> {
        self.basis.command_route.as_ref()
    }

    pub(crate) fn restore_query_from_facts(
        &mut self,
        facts: Box<[crate::fact_contract::UiProducedFact]>,
    ) {
        for fact in facts {
            let fact = match fact.into_query_owner_consequence() {
                Ok(query) => {
                    self.restore_query_consequence(query);
                    continue;
                }
                Err(fact) => fact,
            };
            match fact.into_query_projection_observation() {
                Ok(projection) => self.restore_query_projection(projection),
                Err(fact) => debug_assert!(fact.intent_posture().is_some()),
            }
        }
    }

    pub(crate) fn restore_query_consequence(
        &mut self,
        consequence: worth_ui_query_binding::WorthUiCollectionChangeConsequence,
    ) {
        assert!(self
            .batch
            .query_collection_change
            .replace(consequence)
            .is_none());
    }

    pub(crate) fn restore_query_projection(
        &mut self,
        observation: worth_ui_query_binding::UiProjectionObservation,
    ) {
        assert!(self.batch.query_projection.replace(observation).is_none());
    }
}
