use std::collections::{HashMap, HashSet};

use worth_query::facade::runtime;
use worth_runtime_bridge::facade::{
    BridgeAsyncRequestTruthViewBasis, RelationalBridgeRecordIdentityParts,
};

use super::{
    semantic_transition::{PresentationSemanticTransition, RetainedPresentationSemanticState},
    WorthUiPresentationAsyncDeclaration, WorthUiPresentationRequestBasis,
    WorthUiPresentationRuntimeAdmission, WorthUiPresentationRuntimeAdmissionDenial,
};

mod admission_index;
mod execution;
mod partition_scope;
pub(super) mod partitions;
mod subscriber_index;
use execution::{PendingSemanticExecution, SemanticExecutionKey};
pub use partitions::WorthUiPresentationSemanticSubscriberIdentity;
use partitions::{
    semantic_instance_specifications, PresentationSemanticInstanceSpecification,
    PresentationSemanticPartition,
};
use subscriber_index::SemanticSubscriberIndex;
pub use subscriber_index::WorthUiPresentationScopeRejectionCounters;

const DEPENDENCY_COUNT: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiPresentationSemanticChange {
    Content,
    Width,
    PaintValue,
    PaintBoundary,
    Dpi,
    UploadCompletion,
    PinRelease,
    Currentness,
}

pub(crate) struct PresentationSemanticPublication {
    change: WorthUiPresentationSemanticChange,
    partitions: Box<[PresentationSemanticPartition]>,
}

pub struct WorthUiPresentationSemanticExecution {
    deliveries: Box<[worth_runtime_bridge::facade::CorrespondenceDeliveryCounters]>,
    query: Box<[WorthUiPresentationSemanticQueryObservation]>,
    subscribers: Box<[WorthUiPresentationSemanticSubscriberIdentity]>,
    scope_rejections: WorthUiPresentationScopeRejectionCounters,
}

pub struct WorthUiPresentationSemanticQueryObservation {
    outcome: worth_query::facade::domain::WorthQueryConditionalOutcomeClass,
    performed: worth_signal::facade::adapters::InvalidationExecutionSummary,
}

#[derive(Debug)]
pub enum WorthUiPresentationSemanticExecutionDenial {
    ExecutionAttemptExhausted,
    ScopeCounterOverflow,
    MissingSourceInstance,
    Delivery(runtime::WorthQueryOwnedConditionalInstanceDenial),
    DeliveryDenied(Box<worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryDenial>),
    DeliveryDeferred(worth_runtime_bridge::facade::BridgeCorrespondenceDeferred),
    DeliveryStale(worth_runtime_bridge::facade::BridgeCorrespondenceStale),
    DeliveryRebindRequired(worth_runtime_bridge::facade::BridgeCorrespondenceRebindRequired),
    DeliveryFailed(worth_runtime_bridge::facade::BridgeCorrespondenceAdmissionFailure),
    Domain(worth_query::facade::domain::WorthQueryDomainHandleDenial),
    OperatingWorld(Box<worth_query::facade::installed::WorthQueryOperatingWorldEntryDenial>),
    Binding(Box<worth_query::facade::domain::WorthQueryOperationBindingDenial>),
    Resources(
        worth_query::facade::installed::operation::WorthQueryExecutionResourceAdmissionDenial,
    ),
    Query(Box<worth_query::facade::domain::WorthQueryOwnedConditionalExecutionDenial>),
}

#[derive(Clone)]
pub(super) struct RegisteredSemanticInstance {
    subscriber: WorthUiPresentationSemanticSubscriberIdentity,
    query: runtime::WorthQueryInstalledOwnedConditionalInstance,
}

struct AdmissionSemanticRegistration {
    registration: u64,
    subscriber: WorthUiPresentationSemanticSubscriberIdentity,
    partitions: [PresentationSemanticPartition; DEPENDENCY_COUNT],
}

#[derive(Default)]
pub struct WorthUiPresentationAsyncRegistry {
    next_partition_identity: u64,
    next_semantic_version: u64,
    next_execution_attempt: u64,
    partitions: HashMap<PresentationSemanticPartition, RelationalBridgeRecordIdentityParts>,
    partition_references: HashMap<PresentationSemanticPartition, usize>,
    execution_attempts: HashMap<PresentationSemanticPartition, std::num::NonZeroU64>,
    semantic_executions: HashMap<SemanticExecutionKey, PendingSemanticExecution>,
    semantic_retirements: HashMap<SemanticExecutionKey, usize>,
    admissions: HashMap<SemanticExecutionKey, Vec<AdmissionSemanticRegistration>>,
    instances: SemanticSubscriberIndex,
}

impl PresentationSemanticPublication {
    pub(crate) fn new(
        change: WorthUiPresentationSemanticChange,
        partitions: Vec<PresentationSemanticPartition>,
    ) -> Self {
        Self {
            change,
            partitions: partitions.into_boxed_slice(),
        }
    }

    pub(crate) const fn change(&self) -> WorthUiPresentationSemanticChange {
        self.change
    }

    pub(crate) fn partitions(&self) -> &[PresentationSemanticPartition] {
        &self.partitions
    }
}

impl WorthUiPresentationSemanticChange {
    pub(crate) const fn ordinal(self) -> usize {
        match self {
            Self::Content => 0,
            Self::Width => 1,
            Self::PaintValue => 2,
            Self::PaintBoundary => 3,
            Self::Dpi => 4,
            Self::UploadCompletion => 5,
            Self::PinRelease => 6,
            Self::Currentness => 7,
        }
    }
}

impl WorthUiPresentationAsyncRegistry {
    pub(super) fn admit_retained(
        &mut self,
        workspace: &mut runtime::WorthQueryWorkspace,
        basis: WorthUiPresentationRequestBasis,
        transition: &PresentationSemanticTransition,
        truth_basis: BridgeAsyncRequestTruthViewBasis,
    ) -> Result<WorthUiPresentationRuntimeAdmission, WorthUiPresentationRuntimeAdmissionDenial>
    {
        self.admit_semantic_state(
            workspace,
            basis,
            transition.successor(),
            transition.removed_mechanics(),
            truth_basis,
        )
    }

    fn admit_semantic_state(
        &mut self,
        workspace: &mut runtime::WorthQueryWorkspace,
        basis: WorthUiPresentationRequestBasis,
        semantic_state: &RetainedPresentationSemanticState,
        removed: &[super::WorthUiPresentationMechanicBasis],
        truth_basis: BridgeAsyncRequestTruthViewBasis,
    ) -> Result<WorthUiPresentationRuntimeAdmission, WorthUiPresentationRuntimeAdmissionDenial>
    {
        let declaration = WorthUiPresentationAsyncDeclaration::declare(&basis)
            .map_err(|_| WorthUiPresentationRuntimeAdmissionDenial::QueryDeclarationMismatch)?;
        let specifications = semantic_instance_specifications(&basis, semantic_state, removed);
        let (installations, new_partitions) = self.semantic_installations(&specifications)?;
        let admission = WorthUiPresentationRuntimeAdmission::admit_in_workspace(
            workspace,
            declaration,
            truth_basis,
            installations,
        );
        let admission = match admission {
            Ok(admission) => admission,
            Err(denial) => {
                for partition in new_partitions {
                    self.partitions.remove(&partition);
                }
                return Err(denial);
            }
        };
        let key = SemanticExecutionKey::for_admission(&admission, 0);
        let mut registrations = Vec::with_capacity(specifications.len());
        for (specification, query) in specifications
            .into_iter()
            .zip(admission.semantic_instances().iter().cloned())
        {
            let registration = self.register_instance(&specification, query.clone());
            registrations.push(AdmissionSemanticRegistration {
                registration,
                subscriber: specification.subscriber,
                partitions: specification.partitions,
            });
        }
        self.admissions.insert(key, registrations);
        Ok(admission)
    }

    pub fn retire(
        &mut self,
        workspace: &mut runtime::WorthQueryWorkspace,
        admission: &WorthUiPresentationRuntimeAdmission,
    ) -> Result<(), runtime::WorthQueryOwnedConditionalInstanceDenial> {
        let key = SemanticExecutionKey::for_admission(admission, 0);
        let mut next = self.semantic_retirements.remove(&key).unwrap_or(0);
        while next < admission.semantic_instances().len() {
            if let Err(denial) = admission.retire_semantic_at(workspace, next) {
                self.semantic_retirements.insert(key, next);
                return Err(denial);
            }
            next += 1;
        }
        self.semantic_executions
            .retain(|candidate, _| !candidate.same_admission(key));
        if let Some(registrations) = self.admissions.remove(&key) {
            for registration in registrations {
                self.unregister_instance(registration);
            }
        }
        Ok(())
    }

    pub(super) fn publication_for_admission(
        &self,
        admission: &WorthUiPresentationRuntimeAdmission,
        change: WorthUiPresentationSemanticChange,
    ) -> Result<PresentationSemanticPublication, WorthUiPresentationSemanticExecutionDenial> {
        let key = SemanticExecutionKey::for_admission(admission, 0);
        let registrations = self
            .admissions
            .get(&key)
            .ok_or(WorthUiPresentationSemanticExecutionDenial::MissingSourceInstance)?;
        let has_removal = registrations
            .iter()
            .any(|registration| registration.subscriber.removal());
        let mut partitions = Vec::new();
        let mut seen = HashSet::new();
        for registration in registrations {
            let include = match change {
                WorthUiPresentationSemanticChange::PinRelease => {
                    registration.subscriber.removal() || !has_removal
                }
                WorthUiPresentationSemanticChange::UploadCompletion
                | WorthUiPresentationSemanticChange::Currentness => {
                    !registration.subscriber.removal()
                }
                _ => true,
            };
            if !include {
                continue;
            }
            let partition = registration.partitions[change.ordinal()].clone();
            if seen.insert(partition.clone()) {
                partitions.push(partition);
            }
        }
        Ok(PresentationSemanticPublication::new(change, partitions))
    }
}

impl WorthUiPresentationSemanticExecution {
    pub fn deliveries(&self) -> &[worth_runtime_bridge::facade::CorrespondenceDeliveryCounters] {
        &self.deliveries
    }

    pub fn query_observations(&self) -> &[WorthUiPresentationSemanticQueryObservation] {
        &self.query
    }

    pub fn subscribers(&self) -> &[WorthUiPresentationSemanticSubscriberIdentity] {
        &self.subscribers
    }

    pub const fn scope_rejections(&self) -> WorthUiPresentationScopeRejectionCounters {
        self.scope_rejections
    }
}

impl WorthUiPresentationSemanticQueryObservation {
    pub const fn outcome(&self) -> worth_query::facade::domain::WorthQueryConditionalOutcomeClass {
        self.outcome
    }

    pub const fn performed_signal_invalidation(
        &self,
    ) -> &worth_signal::facade::adapters::InvalidationExecutionSummary {
        &self.performed
    }
}

impl std::fmt::Display for WorthUiPresentationSemanticExecutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExecutionAttemptExhausted => formatter.write_str("execution attempt exhausted"),
            Self::ScopeCounterOverflow => formatter.write_str("scope counter overflow"),
            Self::MissingSourceInstance => formatter.write_str("missing source instance"),
            Self::Delivery(denial) => write!(formatter, "semantic delivery: {denial:?}"),
            Self::DeliveryDenied(denial) => write!(formatter, "delivery denied: {denial:?}"),
            Self::DeliveryDeferred(denial) => write!(formatter, "delivery deferred: {denial:?}"),
            Self::DeliveryStale(denial) => write!(formatter, "delivery stale: {denial:?}"),
            Self::DeliveryRebindRequired(denial) => {
                write!(formatter, "delivery rebind required: {denial:?}")
            }
            Self::DeliveryFailed(denial) => write!(formatter, "delivery failed: {denial:?}"),
            Self::Domain(denial) => write!(formatter, "domain access: {denial:?}"),
            Self::OperatingWorld(denial) => write!(formatter, "operating world: {denial:?}"),
            Self::Binding(denial) => write!(formatter, "operation binding: {denial:?}"),
            Self::Resources(denial) => write!(formatter, "execution resources: {denial:?}"),
            Self::Query(denial) => write!(formatter, "Query execution: {denial:?}"),
        }
    }
}
