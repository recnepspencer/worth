use worth_query::facade::installed::{self, collection, observation};

use super::{
    UiCollectionProjectionOpenOutcome, UiCollectionProjectionOpenReceipt,
    UiCollectionProjectionOpenStopKind as StopKind, UiLiveCollectionProjection,
};

pub(super) fn open_collection_projection(
    phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    binding: &mut crate::UiCollectionProjectionBinding,
    budget: crate::UiCollectionProjectionBudget,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> Result<OpenedCollection, StopKind> {
    let Some(prepared) = binding.take_prepared() else {
        return Err(StopKind::AlreadyOpened);
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
        Err(_) => return Err(StopKind::ResourceAdmission),
    };
    let executed =
        match installed::transition::execution(admitted.execute_in_advancement(phase, workspace)) {
            installed::transition::WorthQueryExecutionTransition::Executed(value) => value,
            installed::transition::WorthQueryExecutionTransition::Deferred(_) => {
                return Err(StopKind::ExecutionDeferred);
            }
            installed::transition::WorthQueryExecutionTransition::Denied(_) => {
                return Err(StopKind::ExecutionDenied);
            }
            installed::transition::WorthQueryExecutionTransition::Stale(_) => {
                return Err(StopKind::ExecutionStale);
            }
            installed::transition::WorthQueryExecutionTransition::RebindRequired(_) => {
                return Err(StopKind::ExecutionRebindRequired);
            }
            installed::transition::WorthQueryExecutionTransition::Failed(_) => {
                return Err(StopKind::ExecutionFailed);
            }
        };
    let published = match installed::transition::publication(executed.publish()) {
        installed::transition::WorthQueryPublicationTransition::Published(value) => value,
        installed::transition::WorthQueryPublicationTransition::Denied(_) => {
            return Err(StopKind::PublicationDenied);
        }
        installed::transition::WorthQueryPublicationTransition::Stale(_) => {
            return Err(StopKind::PublicationStale);
        }
        installed::transition::WorthQueryPublicationTransition::RebindRequired(_) => {
            return Err(StopKind::PublicationRebindRequired);
        }
        installed::transition::WorthQueryPublicationTransition::Failed(_) => {
            return Err(StopKind::PublicationFailed);
        }
    };
    let consumed = match installed::transition::consumption(published.consume_bound(request)) {
        installed::transition::WorthQueryConsumptionTransition::Consumed(value) => value,
        installed::transition::WorthQueryConsumptionTransition::Denied(_) => {
            return Err(StopKind::ConsumptionDenied);
        }
        installed::transition::WorthQueryConsumptionTransition::Deferred(_) => {
            return Err(StopKind::ConsumptionDeferred);
        }
        installed::transition::WorthQueryConsumptionTransition::Stale(_) => {
            return Err(StopKind::ConsumptionStale);
        }
        installed::transition::WorthQueryConsumptionTransition::RebindRequired(_) => {
            return Err(StopKind::ConsumptionRebindRequired);
        }
        installed::transition::WorthQueryConsumptionTransition::Failed(_) => {
            return Err(StopKind::ConsumptionFailed);
        }
    };
    let settled = match installed::transition::settlement(consumed.settle()) {
        installed::transition::WorthQuerySettlementTransition::Settled(value) => value,
        installed::transition::WorthQuerySettlementTransition::Denied(_) => {
            return Err(StopKind::SettlementDenied);
        }
        installed::transition::WorthQuerySettlementTransition::Stale(_) => {
            return Err(StopKind::SettlementStale);
        }
        installed::transition::WorthQuerySettlementTransition::RebindRequired(_) => {
            return Err(StopKind::SettlementRebindRequired);
        }
        installed::transition::WorthQuerySettlementTransition::Failed(_) => {
            return Err(StopKind::SettlementFailed);
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
        Err(_) => return Err(StopKind::CollectionConsumer),
    };
    let fact = crate::projection_consumption::derive_initial_collection_projection(
        crate::projection_consumption::UiCollectionDerivationContext {
            binding,
            consumer: &consumer,
            text_accesses: &text_accesses,
            application_item_key_access: application_item_key_access.as_ref(),
            budget,
        },
    );
    let promoted = match settled
        .into_lifecycle()
        .promote_in_advancement(phase, workspace)
    {
        observation::WorthQueryProjectionPromotionOutcome::Promoted(value) => value,
        observation::WorthQueryProjectionPromotionOutcome::Denied(_) => {
            return Err(StopKind::PromotionDenied);
        }
        observation::WorthQueryProjectionPromotionOutcome::Deferred(_) => {
            return Err(StopKind::PromotionDeferred);
        }
        observation::WorthQueryProjectionPromotionOutcome::Stale(_) => {
            return Err(StopKind::PromotionStale);
        }
        observation::WorthQueryProjectionPromotionOutcome::RebindRequired(_) => {
            return Err(StopKind::PromotionRebindRequired);
        }
        observation::WorthQueryProjectionPromotionOutcome::AuthorityRevalidationRequired(_) => {
            return Err(StopKind::PromotionAuthorityRevalidationRequired);
        }
        observation::WorthQueryProjectionPromotionOutcome::Failed(_) => {
            return Err(StopKind::PromotionFailed);
        }
    };
    let lease = match promoted.into_managed_lease(workspace) {
        observation::WorthQueryProjectionLeaseAdmissionOutcome::Admitted(value) => value,
        observation::WorthQueryProjectionLeaseAdmissionOutcome::Stopped(_) => {
            return Err(StopKind::LeaseAdmission);
        }
    };
    Ok(OpenedCollection {
        reference,
        lease,
        consumer,
        text_accesses,
        application_item_key_access,
        budget,
        fact,
    })
}

/// The phase owns the work; attachment of the already-opened resources only moves them.
pub(super) struct OpenedCollection {
    reference: crate::application_binding::WorthUiInstalledCollectionTextOperationReference,
    lease: super::QueryLease,
    consumer: collection::WorthQueryCollectionConsumerWindow,
    text_accesses: Box<[crate::application_binding::WorthUiCollectionTextNativeAccess]>,
    application_item_key_access:
        Option<crate::application_binding::WorthUiCollectionTextNativeAccess>,
    budget: crate::UiCollectionProjectionBudget,
    fact: crate::UiCollectionProjectionFactReceipt,
}
impl OpenedCollection {
    pub(super) fn into_outcome(
        self,
        binding: crate::UiCollectionProjectionBinding,
    ) -> UiCollectionProjectionOpenOutcome {
        UiCollectionProjectionOpenOutcome::Opened(UiCollectionProjectionOpenReceipt {
            live: UiLiveCollectionProjection {
                binding,
                reference: self.reference,
                lease: self.lease,
                consumer: self.consumer,
                text_accesses: self.text_accesses,
                application_item_key_access: self.application_item_key_access,
                budget: self.budget,
            },
            fact: self.fact,
        })
    }
}
