use std::sync::Arc;

use super::super::conditional_owner_delivery_admission::{
    WorthQueryStagedOwnerDeliveryAdmission, WorthQueryStagedOwnerDeliveryAdmissionError,
};
use super::super::{WorthQueryLiveArtifactTarget, WorthQueryRuntime, WorthQueryRuntimeError};
use super::owner_delivery_queue::WorthQueryQueuedOwnerDeliveryAdmission;

pub(crate) struct WorthQueryAdmittedStagedOwnerDelivery {
    pub(super) target: WorthQueryLiveArtifactTarget,
    queued: WorthQueryQueuedOwnerDeliveryAdmission,
}

impl WorthQueryAdmittedStagedOwnerDelivery {
    pub(crate) const fn work(&self) -> WorthQueryStagedOwnerDeliveryAdmission {
        self.queued.work()
    }

    pub(crate) fn retained_classification(
        &self,
    ) -> Option<crate::runtime::WorthQueryRetainedOwnerDeliveryClassification> {
        self.queued.target_continuation().retained()
    }

    pub(crate) fn retain_classification(
        &self,
        classification: crate::runtime::WorthQueryRetainedOwnerDeliveryClassification,
    ) -> crate::runtime::WorthQueryRetainedOwnerDeliveryClassification {
        self.queued.target_continuation().retain(classification)
    }

    pub(crate) fn retained_decision_seed(
        &self,
    ) -> Option<Arc<crate::domain_installation::WorthQueryRetainedConditionalDecision>> {
        self.queued.continuation().seed()
    }

    pub(crate) fn retain_decision_seed(
        &self,
        seed: crate::domain_installation::WorthQueryRetainedConditionalDecision,
    ) -> Arc<crate::domain_installation::WorthQueryRetainedConditionalDecision> {
        self.queued.continuation().retain_seed(seed)
    }
}

pub(crate) enum WorthQueryClassifiedOwnerDeliveryEmissionError {
    Impact(crate::domain_installation::WorthQueryImpactAdmissionDenial),
    Runtime(WorthQueryRuntimeError),
}

impl WorthQueryRuntime {
    pub(crate) fn stage_conditional_owner_delivery<D: 'static, O: 'static, F: 'static>(
        &mut self,
        receipt: &worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt,
    ) {
        let operation = super::operation_key::<D, O, F>();
        let Some(targets) = self.installed_live_routes.target_index.conditional_targets(
            operation,
            &crate::domain_installation::query_location_from_bridge_candidate(
                receipt.change_set().dependency(),
            ),
        ) else {
            return;
        };
        let continuation = Arc::new(
            super::super::conditional_owner_delivery_continuation::WorthQueryOwnerDeliveryContinuation::default(),
        );
        let mut shared_work = crate::runtime::WorthQueryLiveMutationRoutingWork {
            installed_relevance_index_probes: 1,
            installed_target_candidates_selected: targets.len(),
            ..Default::default()
        };
        for target in targets {
            let route = self
                .installed_live_routes
                .routes
                .get_mut(target)
                .expect("installed owner route index must retain its target");
            let mut routing_work = crate::runtime::WorthQueryLiveMutationRoutingWork {
                capability_index_lookups: 1,
                live_target_candidates_visited: 1,
                installed_route_index_probes: 1,
                ..Default::default()
            };
            routing_work.add(shared_work);
            shared_work = Default::default();
            route
                .owner_deliveries
                .stage(receipt, Arc::clone(&continuation), routing_work);
        }
    }

    pub(crate) fn admit_staged_conditional_owner_delivery<D: 'static, O: 'static, F: 'static>(
        &self,
        target: &WorthQueryLiveArtifactTarget,
        owner: &worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt,
    ) -> Result<WorthQueryAdmittedStagedOwnerDelivery, WorthQueryStagedOwnerDeliveryAdmissionError>
    {
        let operation = super::operation_key::<D, O, F>();
        let Some(route) = self.installed_live_routes.routes.get(target) else {
            return Err(admission_error(Default::default()));
        };
        if route.operation != operation {
            return Err(admission_error(Default::default()));
        }
        let queued = route.owner_deliveries.admit(owner)?;
        Ok(WorthQueryAdmittedStagedOwnerDelivery {
            target: target.clone(),
            queued,
        })
    }

    pub(crate) fn emit_classified_conditional_owner_delivery(
        &mut self,
        admitted: WorthQueryAdmittedStagedOwnerDelivery,
        closure: &crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure,
        conditional: &crate::domain_installation::WorthQueryConditionalProvenance,
        impact: &crate::domain_installation::WorthQueryImpactDecision,
    ) -> Result<(), WorthQueryClassifiedOwnerDeliveryEmissionError> {
        impact
            .readmit_owner_delivery(closure, admitted.queued.receipt(), conditional)
            .map_err(WorthQueryClassifiedOwnerDeliveryEmissionError::Impact)?;
        super::controlled_fault::deny_injected_emission(self, &admitted)?;
        if impact.class()
            != crate::domain_installation::WorthQueryImpactClass::UnaffectedOrSuppressed
        {
            self.emit_owner_mutation(&admitted, impact.class())?;
        }
        self.consume_staged_conditional_owner_delivery(&admitted);
        Ok(())
    }

    fn emit_owner_mutation(
        &mut self,
        admitted: &WorthQueryAdmittedStagedOwnerDelivery,
        impact: crate::domain_installation::WorthQueryImpactClass,
    ) -> Result<(), WorthQueryClassifiedOwnerDeliveryEmissionError> {
        let target_collection = self
            .live_subscriptions
            .get(&admitted.target)
            .ok_or_else(|| {
                WorthQueryClassifiedOwnerDeliveryEmissionError::Runtime(
                    WorthQueryRuntimeError::MissingLiveSubscription(
                        admitted.target.view_name().to_string(),
                    ),
                )
            })?
            .request
            .target_collection_identity();
        let mutation = super::super::conditional_owner_delivery_lowering::owner_mutation_receipt(
            target_collection,
            admitted.queued.receipt(),
        )
        .map_err(WorthQueryClassifiedOwnerDeliveryEmissionError::Runtime)?;
        super::super::live_subscription_delivery_routing::route_classified_live_subscription_delivery(
            &mut self.active_subscriptions,
            &mut self.live_subscriptions,
            super::super::live_subscription_delivery_routing::ClassifiedLiveSubscriptionRoute::new(
                &admitted.target,
                &mutation,
                impact,
                admitted.queued.routing_work(),
            ),
        )
        .map_err(WorthQueryClassifiedOwnerDeliveryEmissionError::Runtime)?;
        Ok(())
    }

    fn consume_staged_conditional_owner_delivery(
        &mut self,
        admitted: &WorthQueryAdmittedStagedOwnerDelivery,
    ) {
        let route = self
            .installed_live_routes
            .routes
            .get_mut(&admitted.target)
            .expect("admitted installed owner route must remain registered");
        route.owner_deliveries.consume(&admitted.queued);
    }
}

fn admission_error(
    work: WorthQueryStagedOwnerDeliveryAdmission,
) -> WorthQueryStagedOwnerDeliveryAdmissionError {
    WorthQueryStagedOwnerDeliveryAdmissionError::causal_mismatch(work)
}
