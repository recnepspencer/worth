use worth_query::facade::{
    foundation,
    installed::{self, collection, observation, operation},
    runtime,
};

use crate::application_binding::WorthUiSnapshotNativeAccess;
use crate::{
    operation_live::resource::WorthUiOperationLiveSources, WorthUiCollectionAllocationPolicy,
    WorthUiInstalledQueryBindingReference, WorthUiOperationLiveResource,
    WorthUiQueryConsumerRequirements, WorthUiQueryOperationAttemptDenial,
    WorthUiSnapshotConsumerPreparationDenial,
};

pub(crate) type Settled = operation::WorthQuerySettledDomainProjection<
    crate::WorthUiDomainEntry,
    crate::WorthUiSnapshotMeasurement,
    crate::WorthUiSnapshotMeasurementFamily,
    foundation::ObservationLaneWitness,
>;
pub(crate) struct WorthUiOperationLiveSettlement {
    pub(crate) settled: Settled,
    pub(crate) native_access: WorthUiSnapshotNativeAccess,
}
type Deferred = operation::WorthQueryDeferredDomainOperation<
    crate::WorthUiDomainEntry,
    crate::WorthUiSnapshotMeasurement,
    crate::WorthUiSnapshotMeasurementFamily,
    foundation::ObservationLaneWitness,
>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiOperationLiveOpenRequest {
    requirements: WorthUiQueryConsumerRequirements,
    breadth: collection::WorthQueryCollectionWindowBreadth,
    allocation: WorthUiCollectionAllocationPolicy,
}

impl WorthUiOperationLiveOpenRequest {
    pub fn new(
        requirements: WorthUiQueryConsumerRequirements,
        breadth: collection::WorthQueryCollectionWindowBreadth,
        allocation: WorthUiCollectionAllocationPolicy,
    ) -> Self {
        Self {
            requirements,
            breadth,
            allocation,
        }
    }
}

pub enum WorthUiOperationLiveOpenError {
    Attempt(Box<WorthUiQueryOperationAttemptDenial>),
    Preparation(Box<WorthUiSnapshotConsumerPreparationDenial>),
    Derivation(Box<crate::WorthUiQueryMeasurementFactObservationError>),
    Deferred(Box<Deferred>),
    ResourceAdmission(Box<installed::transition::WorthQueryResourceAdmissionStop>),
    Execution(Box<operation::WorthQueryBoundExecutionDenial>),
    Publication(Box<operation::WorthQueryPublicationDenial>),
    Consumption(Box<operation::WorthQueryProgressionDenial>),
    Settlement(Box<operation::WorthQueryProgressionDenial>),
    CollectionConsumer(Box<installed::collection::WorthQueryCollectionConsumerPreparationDenial>),
    Promotion(
        Box<
            observation::WorthQueryProjectionPromotionOutcome<
                crate::WorthUiDomainEntry,
                crate::WorthUiSnapshotMeasurement,
                crate::WorthUiSnapshotMeasurementFamily,
                foundation::ObservationLaneWitness,
            >,
        >,
    ),
    LeaseAdmission(
        Box<
            observation::WorthQueryProjectionLeaseAdmissionStop<
                crate::WorthUiDomainEntry,
                crate::WorthUiSnapshotMeasurement,
                crate::WorthUiSnapshotMeasurementFamily,
                foundation::ObservationLaneWitness,
            >,
        >,
    ),
}

impl std::fmt::Debug for WorthUiOperationLiveOpenError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Attempt(_) => "Attempt",
            Self::Preparation(_) => "Preparation",
            Self::Derivation(_) => "Derivation",
            Self::Deferred(_) => "Deferred",
            Self::ResourceAdmission(_) => "ResourceAdmission",
            Self::Execution(_) => "Execution",
            Self::Publication(_) => "Publication",
            Self::Consumption(_) => "Consumption",
            Self::Settlement(_) => "Settlement",
            Self::CollectionConsumer(_) => "CollectionConsumer",
            Self::Promotion(_) => "Promotion",
            Self::LeaseAdmission(_) => "LeaseAdmission",
        };
        formatter
            .debug_tuple(name)
            .field(&"exact stop retained")
            .finish()
    }
}

pub(crate) fn open_operation_live_resource(
    request_execution: &worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    reference: WorthUiInstalledQueryBindingReference,
    request: WorthUiOperationLiveOpenRequest,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> Result<WorthUiOperationLiveResource, WorthUiOperationLiveOpenError> {
    let settled = settle_once(&reference, request.requirements, workspace)?;
    WorthUiOperationLiveResource::open(
        request_execution,
        WorthUiOperationLiveSources {
            installed_reference: reference,
            settlement: settled,
        },
        request.breadth,
        request.allocation,
        workspace,
    )
}

pub(crate) fn settle_once(
    reference: &WorthUiInstalledQueryBindingReference,
    requirements: WorthUiQueryConsumerRequirements,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> Result<WorthUiOperationLiveSettlement, WorthUiOperationLiveOpenError> {
    let gateway = reference
        .enter_snapshot_attempt(workspace)
        .map_err(|error| WorthUiOperationLiveOpenError::Attempt(Box::new(error)))?;
    let prepared = gateway
        .prepare_snapshot_consumer(requirements)
        .map_err(|error| WorthUiOperationLiveOpenError::Preparation(Box::new(error)))?;
    let (_, bound, native_request, _) = prepared.into_parts();
    let (projection, native_access) = native_request.into_parts();
    let admitted = installed::transition::resource_admission(bound.admit_execution_resources(
        (),
        crate::installed_domain::execution_resources::operation_execution_resource_request(),
        workspace,
    ))
    .into_result()
    .map_err(|stop| WorthUiOperationLiveOpenError::ResourceAdmission(Box::new(stop)))?;
    let executed = match installed::transition::execution(admitted.execute(workspace)) {
        installed::transition::WorthQueryExecutionTransition::Executed(executed) => executed,
        installed::transition::WorthQueryExecutionTransition::Deferred(deferred) => {
            return Err(WorthUiOperationLiveOpenError::Deferred(Box::new(deferred)));
        }
        installed::transition::WorthQueryExecutionTransition::Denied(stop)
        | installed::transition::WorthQueryExecutionTransition::Stale(stop)
        | installed::transition::WorthQueryExecutionTransition::RebindRequired(stop)
        | installed::transition::WorthQueryExecutionTransition::Failed(stop) => {
            return Err(WorthUiOperationLiveOpenError::Execution(Box::new(stop)));
        }
    };
    let published = match installed::transition::publication(executed.publish()) {
        installed::transition::WorthQueryPublicationTransition::Published(value) => value,
        installed::transition::WorthQueryPublicationTransition::Denied(stop)
        | installed::transition::WorthQueryPublicationTransition::Stale(stop)
        | installed::transition::WorthQueryPublicationTransition::RebindRequired(stop)
        | installed::transition::WorthQueryPublicationTransition::Failed(stop) => {
            return Err(WorthUiOperationLiveOpenError::Publication(Box::new(stop)));
        }
    };
    let consumed = match installed::transition::consumption(published.consume_bound(projection)) {
        installed::transition::WorthQueryConsumptionTransition::Consumed(value) => value,
        installed::transition::WorthQueryConsumptionTransition::Denied(stop)
        | installed::transition::WorthQueryConsumptionTransition::Deferred(stop)
        | installed::transition::WorthQueryConsumptionTransition::Stale(stop)
        | installed::transition::WorthQueryConsumptionTransition::RebindRequired(stop)
        | installed::transition::WorthQueryConsumptionTransition::Failed(stop) => {
            return Err(WorthUiOperationLiveOpenError::Consumption(Box::new(stop)));
        }
    };
    match installed::transition::settlement(consumed.settle()) {
        installed::transition::WorthQuerySettlementTransition::Settled(settled) => {
            Ok(WorthUiOperationLiveSettlement {
                settled,
                native_access,
            })
        }
        installed::transition::WorthQuerySettlementTransition::Denied(stop)
        | installed::transition::WorthQuerySettlementTransition::Stale(stop)
        | installed::transition::WorthQuerySettlementTransition::RebindRequired(stop)
        | installed::transition::WorthQuerySettlementTransition::Failed(stop) => {
            Err(WorthUiOperationLiveOpenError::Settlement(Box::new(stop)))
        }
    }
}
