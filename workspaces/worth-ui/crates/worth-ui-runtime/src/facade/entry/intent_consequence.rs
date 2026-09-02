use super::{
    intent_consequence_observation::prepare_intent_consequence_observation,
    intent_consequence_publication::UiIntentConsequencePublicationOutcome,
    intent_consequence_rebind::WorthUiIntentConsequenceRebindTransfer,
    WorthUiActiveApplicationSession,
};
use crate::runtime::intent_execution::{
    UiIntentConsequenceBeginOutcome, UiIntentConsequenceCurrentnessContext,
    UiIntentConsequenceHandoff, UiIntentConsequenceStopReason,
};

#[path = "intent_consequence/portal_service_request.rs"]
mod portal_service_request;
use portal_service_request::{portal_placement_stop_reason, portal_service_request};

impl WorthUiActiveApplicationSession {
    pub fn publish_intent_consequences(
        &mut self,
        handle: crate::facade::intent::UiIntentConsequenceHandle,
        policy: crate::runtime::rebind::UiRebindExecutionPolicy,
        execution: crate::runtime::rebind::UiRebindExecutionRequest,
    ) -> UiIntentConsequencePublicationOutcome<'_> {
        let generation = self.active_generation_identity();
        let prepared = self.application.prepared_authority();
        let begin = self.intent_execution.begin_consequence(
            handle,
            UiIntentConsequenceCurrentnessContext {
                catalog: prepared.intent_catalog(),
                generation: &generation,
                mounted: &self.mounted,
            },
        );
        self.finish_intent_consequence_begin(begin, policy, execution)
    }

    pub fn retry_intent_consequences(
        &mut self,
        recovery: crate::facade::intent::UiIntentConsequenceRecovery,
        policy: crate::runtime::rebind::UiRebindExecutionPolicy,
        execution: crate::runtime::rebind::UiRebindExecutionRequest,
    ) -> UiIntentConsequencePublicationOutcome<'_> {
        let generation = self.active_generation_identity();
        let prepared = self.application.prepared_authority();
        let begin = self.intent_execution.retry_consequence(
            recovery,
            UiIntentConsequenceCurrentnessContext {
                catalog: prepared.intent_catalog(),
                generation: &generation,
                mounted: &self.mounted,
            },
        );
        self.finish_intent_consequence_begin(begin, policy, execution)
    }

    fn finish_intent_consequence_begin(
        &mut self,
        begin: UiIntentConsequenceBeginOutcome,
        policy: crate::runtime::rebind::UiRebindExecutionPolicy,
        execution: crate::runtime::rebind::UiRebindExecutionRequest,
    ) -> UiIntentConsequencePublicationOutcome<'_> {
        match begin {
            UiIntentConsequenceBeginOutcome::Stopped(stop) => {
                UiIntentConsequencePublicationOutcome::Stopped(stop)
            }
            UiIntentConsequenceBeginOutcome::Handoff(handoff) => {
                self.publish_intent_consequence_handoff(handoff, policy, execution)
            }
        }
    }

    fn publish_intent_consequence_handoff(
        &mut self,
        mut handoff: UiIntentConsequenceHandoff,
        policy: crate::runtime::rebind::UiRebindExecutionPolicy,
        execution: crate::runtime::rebind::UiRebindExecutionRequest,
    ) -> UiIntentConsequencePublicationOutcome<'_> {
        let explicit_portal_transition = match handoff.runtime_service_destination() {
            Some(crate::capability::UiIntentRuntimeServiceDestination::InvokeCommand) => {
                if handoff.command_route().is_none() {
                    return self.stop_intent_consequence(
                        handoff,
                        UiIntentConsequenceStopReason::RuntimeServiceCommandRouteMissing,
                    );
                }
                None
            }
            Some(
                destination @ (crate::capability::UiIntentRuntimeServiceDestination::OpenPortal
                | crate::capability::UiIntentRuntimeServiceDestination::ClosePortal),
            ) => {
                if !handoff.includes_mounted_posture() {
                    return self.stop_intent_consequence(
                        handoff,
                        UiIntentConsequenceStopReason::RuntimeServiceRequiresMountedPosture,
                    );
                }
                let viewport = match self
                    .application
                    .mounted_viewport_bounds_for(handoff.graph_node())
                {
                    Ok(viewport) => viewport,
                    Err(_) => {
                        return self.stop_intent_consequence(
                            handoff,
                            UiIntentConsequenceStopReason::RuntimeServicePortalPlacement(
                                crate::runtime::intent_execution::UiIntentPortalPlacementStopReason::IncompatibleCoordinateSpace,
                            ),
                        )
                    }
                };
                let presented_viewport = viewport.and_then(|viewport| {
                    crate::runtime::interaction::UiPresentedViewportGeometry::from_current_interaction(
                        viewport,
                        handoff.target().geometry(),
                    )
                });
                let resolved_owner = self
                    .mounted
                    .current_portal_owner_for_child(handoff.target().mounted_instance());
                let request = portal_service_request(
                    &handoff,
                    destination,
                    presented_viewport,
                    resolved_owner,
                );
                let Some(portal) = self.portal.as_ref() else {
                    return self.stop_intent_consequence(
                        handoff,
                        UiIntentConsequenceStopReason::RuntimeServiceOwnerUnavailable(
                            crate::capability::UiRuntimeServiceFamily::Portal.into(),
                        ),
                    );
                };
                match portal.prepare(request) {
                    Ok(transition) => Some(transition),
                    Err(
                        crate::runtime::portal::UiPortalServiceTransitionDenial::RevisionExhausted
                        | crate::runtime::portal::UiPortalServiceTransitionDenial::StackOrdinalExhausted
                        | crate::runtime::portal::UiPortalServiceTransitionDenial::LiveRowCapacityExceeded { .. }
                        | crate::runtime::portal::UiPortalServiceTransitionDenial::StackOrdinalConflict
                        | crate::runtime::portal::UiPortalServiceTransitionDenial::PortalNotLive
                        | crate::runtime::portal::UiPortalServiceTransitionDenial::PortalSurfaceMismatch
                        | crate::runtime::portal::UiPortalServiceTransitionDenial::ReplacementNotTopmost,
                    ) => {
                        return self.stop_intent_consequence(
                            handoff,
                            UiIntentConsequenceStopReason::RuntimeServiceTransitionExhausted,
                        )
                    }
                    Err(crate::runtime::portal::UiPortalServiceTransitionDenial::StalePlan) => {
                        unreachable!("portal preparation does not mutate its revision")
                    }
                    Err(crate::runtime::portal::UiPortalServiceTransitionDenial::Placement(
                        denial,
                    )) => {
                        return self.stop_intent_consequence(
                            handoff,
                            UiIntentConsequenceStopReason::RuntimeServicePortalPlacement(
                                portal_placement_stop_reason(denial),
                            ),
                        )
                    }
                }
            }
            None => None,
        };
        let portal_transition = if explicit_portal_transition.is_none()
            && handoff.interaction_family()
                == crate::capability::UiSemanticInteractionFamily::SelectionCommit
        {
            match self.portal.as_ref().map(|portal| {
                portal.prepare_dismissal(
                    crate::runtime::portal::UiPortalDismissalTrigger::AcceptedSelection,
                    None,
                    handoff.idempotency(),
                )
            }) {
                None
                | Some(Ok(crate::runtime::portal::UiPortalDismissalPreparation::Ignored(_))) => {
                    None
                }
                Some(Ok(crate::runtime::portal::UiPortalDismissalPreparation::Prepared(
                    dismissal,
                ))) => Some(dismissal.into_transition()),
                Some(Err(
                    crate::runtime::portal::UiPortalServiceTransitionDenial::RevisionExhausted,
                )) => {
                    return self.stop_intent_consequence(
                        handoff,
                        UiIntentConsequenceStopReason::RuntimeServiceTransitionExhausted,
                    )
                }
                Some(Err(crate::runtime::portal::UiPortalServiceTransitionDenial::StalePlan)) => {
                    unreachable!("portal dismissal preparation does not mutate its revision")
                }
                Some(Err(
                    crate::runtime::portal::UiPortalServiceTransitionDenial::StackOrdinalExhausted,
                )) => unreachable!("portal dismissal does not reserve a stack ordinal"),
                Some(Err(
                    crate::runtime::portal::UiPortalServiceTransitionDenial::LiveRowCapacityExceeded { .. }
                    | crate::runtime::portal::UiPortalServiceTransitionDenial::PortalNotLive
                    | crate::runtime::portal::UiPortalServiceTransitionDenial::PortalSurfaceMismatch
                    | crate::runtime::portal::UiPortalServiceTransitionDenial::ReplacementNotTopmost
                    | crate::runtime::portal::UiPortalServiceTransitionDenial::StackOrdinalConflict,
                )) => unreachable!("portal dismissal targets one current live row"),
                Some(Err(crate::runtime::portal::UiPortalServiceTransitionDenial::Placement(
                    denial,
                ))) => {
                    return self.stop_intent_consequence(
                        handoff,
                        UiIntentConsequenceStopReason::RuntimeServicePortalPlacement(
                            portal_placement_stop_reason(denial),
                        ),
                    )
                }
            }
        } else {
            explicit_portal_transition
        };
        let observed = handoff.consequence_count();
        let limit = self.application.intent_consequence_fact_capacity();
        if observed > limit {
            return self.stop_intent_consequence(
                handoff,
                UiIntentConsequenceStopReason::ConsequenceFactCapacityExceeded { limit, observed },
            );
        }
        if observed == 0 {
            let receipt =
                crate::runtime::intent_execution::UiIntentConsequenceCompletionReceipt::new(
                    handoff.attempt(),
                    handoff.idempotency(),
                );
            self.intent_execution.finish_consequence_handoff(handoff);
            return UiIntentConsequencePublicationOutcome::NoConsequences(receipt);
        }
        let posture = if handoff.includes_mounted_posture() {
            match self.intent_postures.prepare(
                handoff.graph_node(),
                handoff.target(),
                crate::fact_contract::UiIntentPostureReference::Attempt {
                    attempt: handoff.attempt(),
                    idempotency: handoff.idempotency(),
                },
                crate::fact_contract::UiIntentPostureKind::Completed,
            ) {
                Some(posture) => Some(posture),
                None => {
                    return self.stop_intent_consequence(
                        handoff,
                        UiIntentConsequenceStopReason::IntentPostureIdentityExhausted,
                    )
                }
            }
        } else {
            None
        };
        let query_reference = handoff.query_operation_live_reference();
        let batch = crate::runtime::observation::UiIntentConsequenceObservationBatch::new(
            posture,
            handoff.take_query_consequence(),
            handoff.take_query_projection(),
        );
        debug_assert!(!batch.is_empty());
        let observation = match prepare_intent_consequence_observation(self, batch) {
            Ok(observation) => observation,
            Err(stop) => {
                restore_query_from_batch(&mut handoff, *stop.batch);
                return self.stop_intent_consequence(handoff, stop.reason);
            }
        };
        debug_assert_eq!(observation.admitted_count, observed);
        let change = self
            .application
            .classify_intent_consequence(self.identity, observation.set);
        let scope = match crate::runtime::rebind::UiAffectedScopeResolver::resolve_recoverable(
            change,
            self.identity,
            self.application.prepared_authority(),
        ) {
            Ok(scope) => scope,
            Err(stop) => {
                let (denial, change) = stop.into_parts();
                let (_, facts, _) = change.into_parts();
                return self.stop_intent_consequence_from_facts(
                    handoff,
                    UiIntentConsequenceStopReason::AffectedScope(Box::new(denial)),
                    facts,
                );
            }
        };
        let lifecycle =
            match crate::runtime::rebind::UiIdentityLifecycleResolver::resolve_recoverable(scope) {
                Ok(lifecycle) => lifecycle,
                Err(stop) => {
                    let (denial, scope) = stop.into_parts();
                    return self.stop_intent_consequence_from_facts(
                        handoff,
                        UiIntentConsequenceStopReason::IdentityLifecycle(Box::new(denial)),
                        scope.into_facts(),
                    );
                }
            };
        let plan = match self.application.compile_non_source_rebind_recoverable(
            self.identity,
            lifecycle,
            policy,
        ) {
            Ok(plan) => plan,
            Err(stop) => {
                let (denial, lifecycle) = stop.into_parts();
                let (scope, _) = lifecycle.into_parts();
                return self.stop_intent_consequence_from_facts(
                    handoff,
                    UiIntentConsequenceStopReason::Planning(Box::new(denial)),
                    scope.into_facts(),
                );
            }
        };
        let now_tick = execution.now_tick();
        let transfer = WorthUiIntentConsequenceRebindTransfer {
            observation: observation.progress,
            posture: observation.posture,
            consequence: handoff,
            portal_transition,
            portal_proposal: None,
            query_reference,
        };
        match self.prepare_intent_consequence_rebind(plan, execution, transfer) {
            Ok(prepared) => prepared.execute(now_tick),
            Err(stop) => UiIntentConsequencePublicationOutcome::Stopped(stop),
        }
    }

    fn stop_intent_consequence(
        &mut self,
        handoff: UiIntentConsequenceHandoff,
        reason: UiIntentConsequenceStopReason,
    ) -> UiIntentConsequencePublicationOutcome<'_> {
        UiIntentConsequencePublicationOutcome::Stopped(
            self.intent_execution
                .retain_consequence_handoff(handoff, reason),
        )
    }

    fn stop_intent_consequence_from_facts(
        &mut self,
        mut handoff: UiIntentConsequenceHandoff,
        reason: UiIntentConsequenceStopReason,
        facts: Box<[crate::fact_contract::UiProducedFact]>,
    ) -> UiIntentConsequencePublicationOutcome<'_> {
        restore_query_from_facts(&mut handoff, facts);
        self.stop_intent_consequence(handoff, reason)
    }
}

fn restore_query_from_batch(
    handoff: &mut UiIntentConsequenceHandoff,
    batch: crate::runtime::observation::UiIntentConsequenceObservationBatch,
) {
    let (_, query, projection) = batch.into_parts();
    if let Some(query) = query {
        handoff.restore_query_consequence(query);
    }
    if let Some(projection) = projection {
        handoff.restore_query_projection(projection);
    }
}

fn restore_query_from_facts(
    handoff: &mut UiIntentConsequenceHandoff,
    facts: Box<[crate::fact_contract::UiProducedFact]>,
) {
    handoff.restore_query_from_facts(facts);
}
