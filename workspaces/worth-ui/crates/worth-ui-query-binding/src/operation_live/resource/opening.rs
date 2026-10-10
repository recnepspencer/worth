use worth_query::facade::{
    installed::{collection, observation},
    runtime,
};

use crate::{
    WorthUiAdmittedQueryBindingReference, WorthUiAdmittedQuerySettlementReference,
    WorthUiCollectionAllocationPolicy, WorthUiOperationLiveOpenError, WorthUiSettledSnapshotFact,
};

use super::{WorthUiOperationLiveResource, WorthUiOperationLiveSources};

impl WorthUiOperationLiveResource {
    pub(crate) fn open(
        request_execution: &worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
        sources: WorthUiOperationLiveSources,
        breadth: collection::WorthQueryCollectionWindowBreadth,
        allocation: WorthUiCollectionAllocationPolicy,
        workspace: &mut runtime::WorthQueryWorkspace,
    ) -> Result<Self, WorthUiOperationLiveOpenError> {
        let consumer = sources
            .settlement
            .settled
            .prepare_collection_consumer(breadth)
            .map_err(|error| WorthUiOperationLiveOpenError::CollectionConsumer(Box::new(error)))?;
        Self::open_with_consumer(request_execution, sources, consumer, allocation, workspace)
    }

    pub(crate) fn open_with_consumer(
        request_execution: &worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
        sources: WorthUiOperationLiveSources,
        consumer: collection::WorthQueryCollectionConsumerWindow,
        allocation: WorthUiCollectionAllocationPolicy,
        workspace: &mut runtime::WorthQueryWorkspace,
    ) -> Result<Self, WorthUiOperationLiveOpenError> {
        let WorthUiOperationLiveSources {
            installed_reference,
            settlement,
        } = sources;
        let super::super::open::WorthUiOperationLiveSettlement {
            settled,
            native_access,
        } = settlement;
        let binding_reference = WorthUiAdmittedQueryBindingReference::admit(&installed_reference);
        let settlement_reference = WorthUiAdmittedQuerySettlementReference::mint();
        let fact = WorthUiSettledSnapshotFact::from_settled(
            &settled,
            &native_access,
            binding_reference,
            settlement_reference,
        )
        .map_err(|error| WorthUiOperationLiveOpenError::Derivation(Box::new(error)))?;
        let promoted = match settled
            .into_lifecycle()
            .promote(request_execution, workspace)
        {
            observation::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
            stopped => {
                return Err(WorthUiOperationLiveOpenError::Promotion(Box::new(stopped)));
            }
        };
        let lease = match promoted.into_managed_lease(workspace) {
            observation::WorthQueryProjectionLeaseAdmissionOutcome::Admitted(lease) => lease,
            observation::WorthQueryProjectionLeaseAdmissionOutcome::Stopped(stop) => {
                return Err(WorthUiOperationLiveOpenError::LeaseAdmission(Box::new(
                    stop,
                )));
            }
        };
        Ok(Self {
            installed_reference,
            fact: std::sync::Arc::new(fact),
            lease,
            consumer,
            allocation,
            collection_source: None,
            next_change_order: 0,
            staged_change: None,
            staged_change_admitted: false,
            admitted_changes: Default::default(),
        })
    }
}
