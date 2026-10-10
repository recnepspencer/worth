use worth_query::facade::installed::{self, collection, observation};

use super::{
    stopped, UiCollectionProjectionOpenOutcome, UiCollectionProjectionOpenReceipt,
    UiCollectionProjectionOpenStopKind as StopKind, UiLiveCollectionProjection,
};

pub(super) fn open_collection_projection(
    request_execution: &worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    mut binding: crate::UiCollectionProjectionBinding,
    budget: crate::UiCollectionProjectionBudget,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> UiCollectionProjectionOpenOutcome {
    let Some(prepared) = binding.take_prepared() else {
        return stopped(&binding, StopKind::AlreadyOpened);
    };
    let (reference, bound, native_request) = prepared.into_parts();
    let (request, text_accesses, application_item_key_access) = native_request.into_parts();
    let admitted = match installed::transition::resource_admission(bound.admit_execution_resources(
        (),
        crate::installed_domain::execution_resources::operation_execution_resource_request(),
        workspace,
    ))
    .into_result()
    {
        Ok(admitted) => admitted,
        Err(_) => return stopped(&binding, StopKind::ResourceAdmission),
    };
    let executed = match installed::transition::execution(admitted.execute(workspace)) {
        installed::transition::WorthQueryExecutionTransition::Executed(value) => value,
        installed::transition::WorthQueryExecutionTransition::Deferred(_) => {
            return stopped(&binding, StopKind::ExecutionDeferred);
        }
        installed::transition::WorthQueryExecutionTransition::Denied(_) => {
            return stopped(&binding, StopKind::ExecutionDenied);
        }
        installed::transition::WorthQueryExecutionTransition::Stale(_) => {
            return stopped(&binding, StopKind::ExecutionStale);
        }
        installed::transition::WorthQueryExecutionTransition::RebindRequired(_) => {
            return stopped(&binding, StopKind::ExecutionRebindRequired);
        }
        installed::transition::WorthQueryExecutionTransition::Failed(_) => {
            return stopped(&binding, StopKind::ExecutionFailed);
        }
    };
    let published = match installed::transition::publication(executed.publish()) {
        installed::transition::WorthQueryPublicationTransition::Published(value) => value,
        installed::transition::WorthQueryPublicationTransition::Denied(_) => {
            return stopped(&binding, StopKind::PublicationDenied);
        }
        installed::transition::WorthQueryPublicationTransition::Stale(_) => {
            return stopped(&binding, StopKind::PublicationStale);
        }
        installed::transition::WorthQueryPublicationTransition::RebindRequired(_) => {
            return stopped(&binding, StopKind::PublicationRebindRequired);
        }
        installed::transition::WorthQueryPublicationTransition::Failed(_) => {
            return stopped(&binding, StopKind::PublicationFailed);
        }
    };
    let consumed = match installed::transition::consumption(published.consume_bound(request)) {
        installed::transition::WorthQueryConsumptionTransition::Consumed(value) => value,
        installed::transition::WorthQueryConsumptionTransition::Denied(_) => {
            return stopped(&binding, StopKind::ConsumptionDenied);
        }
        installed::transition::WorthQueryConsumptionTransition::Deferred(_) => {
            return stopped(&binding, StopKind::ConsumptionDeferred);
        }
        installed::transition::WorthQueryConsumptionTransition::Stale(_) => {
            return stopped(&binding, StopKind::ConsumptionStale);
        }
        installed::transition::WorthQueryConsumptionTransition::RebindRequired(_) => {
            return stopped(&binding, StopKind::ConsumptionRebindRequired);
        }
        installed::transition::WorthQueryConsumptionTransition::Failed(_) => {
            return stopped(&binding, StopKind::ConsumptionFailed);
        }
    };
    let settled = match installed::transition::settlement(consumed.settle()) {
        installed::transition::WorthQuerySettlementTransition::Settled(value) => value,
        installed::transition::WorthQuerySettlementTransition::Denied(_) => {
            return stopped(&binding, StopKind::SettlementDenied);
        }
        installed::transition::WorthQuerySettlementTransition::Stale(_) => {
            return stopped(&binding, StopKind::SettlementStale);
        }
        installed::transition::WorthQuerySettlementTransition::RebindRequired(_) => {
            return stopped(&binding, StopKind::SettlementRebindRequired);
        }
        installed::transition::WorthQuerySettlementTransition::Failed(_) => {
            return stopped(&binding, StopKind::SettlementFailed);
        }
    };
    let breadth = collection::WorthQueryCollectionWindowBreadth::new(
        budget.max_rows(),
        0,
        0,
        budget.max_rows(),
    )
    .expect("the WUI collection budget proves a nonzero coherent Query breadth");
    let consumer = match settled.prepare_collection_consumer(breadth) {
        Ok(value) => value,
        Err(_) => return stopped(&binding, StopKind::CollectionConsumer),
    };
    let fact = crate::projection_consumption::derive_initial_collection_projection(
        crate::projection_consumption::UiCollectionDerivationContext {
            binding: &binding,
            consumer: &consumer,
            text_accesses: &text_accesses,
            application_item_key_access: application_item_key_access.as_ref(),
            budget,
        },
    );
    let promoted = match settled
        .into_lifecycle()
        .promote(request_execution, workspace)
    {
        observation::WorthQueryProjectionPromotionOutcome::Promoted(value) => value,
        observation::WorthQueryProjectionPromotionOutcome::Denied(_) => {
            return stopped(&binding, StopKind::PromotionDenied);
        }
        observation::WorthQueryProjectionPromotionOutcome::Deferred(_) => {
            return stopped(&binding, StopKind::PromotionDeferred);
        }
        observation::WorthQueryProjectionPromotionOutcome::Stale(_) => {
            return stopped(&binding, StopKind::PromotionStale);
        }
        observation::WorthQueryProjectionPromotionOutcome::RebindRequired(_) => {
            return stopped(&binding, StopKind::PromotionRebindRequired);
        }
        observation::WorthQueryProjectionPromotionOutcome::AuthorityRevalidationRequired(_) => {
            return stopped(&binding, StopKind::PromotionAuthorityRevalidationRequired);
        }
        observation::WorthQueryProjectionPromotionOutcome::Failed(_) => {
            return stopped(&binding, StopKind::PromotionFailed);
        }
    };
    let lease = match promoted.into_managed_lease(workspace) {
        observation::WorthQueryProjectionLeaseAdmissionOutcome::Admitted(value) => value,
        observation::WorthQueryProjectionLeaseAdmissionOutcome::Stopped(_) => {
            return stopped(&binding, StopKind::LeaseAdmission);
        }
    };
    UiCollectionProjectionOpenOutcome::Opened(UiCollectionProjectionOpenReceipt {
        live: UiLiveCollectionProjection {
            binding,
            reference,
            lease,
            consumer,
            text_accesses,
            application_item_key_access,
            budget,
        },
        fact,
    })
}
