//! Permanent assembly for independently borrowable Signal owner services.
//!
//! This module exposes the frozen public vocabulary and assembles the private
//! synchronization and capacity kernel consumed by later service phases.

mod admission_table;
mod basis_port;
mod branch_cell_state;
mod branch_execution_cell;
mod branch_registry;
mod cancellation;
mod cell_incarnation;
mod compatibility;
pub(crate) mod conditional_execution;
mod counters;
mod definition_publication_port;
mod lifecycle_observation;
mod lifecycle_port;
mod lifecycle_state;
mod mutation_port;
mod operation_control;
pub(super) mod owner;
mod owner_metadata;
mod service_ports;
mod unavailable;

pub(crate) use cancellation::SignalOwnerMovementPermit;
pub(in crate::branch) use cell_incarnation::SignalBranchCellIncarnation;

pub use basis_port::SignalBranchBasisPort;
pub(crate) use branch_cell_state::SignalBranchCellState;
pub(crate) use branch_execution_cell::SignalBranchExecutionCell;
#[cfg(test)]
pub(crate) use branch_execution_cell::{
    SignalBranchCellAdmissionDenial, SignalBranchCellPoisonRecovery, SignalBranchCellWork,
};
#[cfg(test)]
pub(crate) use branch_registry::SignalBranchRegistryPoisonRecovery;
pub(crate) use branch_registry::{
    SignalBranchRegistry, SignalBranchRegistryDenial, SignalBranchRetirement,
};
pub use conditional_execution::{
    SignalCommittedPatchDeliveryCompletion, SignalCommittedPatchDeliveryDenial,
    SignalCommittedPatchDeliveryRequest, SignalCommittedPatchTarget,
    SignalConditionalDefinitionAdvanceBinding, SignalConditionalDefinitionPublicationOperation,
    SignalConditionalEvaluationAdmission, SignalConditionalEvaluationBindingEvidence,
    SignalConditionalEvaluationReadmission, SignalConditionalEvaluationReadmissionCounters,
    SignalConditionalEvaluationReadmissionDenial, SignalConditionalEvaluationReadmissionRequest,
    SignalConditionalEvaluationSourceEvidence, SignalConditionalExecutionPort,
    SignalConditionalInstallationChangeDenial, SignalConditionalInstallationCustody,
    SignalConditionalInstallationExtensionCompletion, SignalConditionalInstallationExtensionDenial,
    SignalConditionalInstallationExtensionRequest, SignalConditionalReconstitutionReport,
    SignalConditionalRetentionObservation, SignalConditionalRetirementCompletion,
    SignalConditionalServiceCompletion, SignalConditionalServiceExecutionDenial,
    SignalConditionalServiceExecutionRequest, SignalConditionalServiceIssuanceDenial,
    SignalConditionalSuccessorTransition, SignalConditionalTemporalPartition,
    SignalConditionalTemporalPartitionDenial, SignalConditionalTemporalPromotion,
    SignalConditionalTemporalPromotionView, SignalOwnedAsyncRequestAdmission,
    SignalOwnedAsyncRetryAdmission, SignalOwnedAsyncRetrySchedule,
    SignalOwnedAsyncRevalidationAdmission, SignalOwnedAsyncSourceBinding,
    SignalOwnedAsyncTimeoutAdmission, SignalPreparedConditionalInstallationExtension,
};
pub(crate) use counters::SignalOwnerServiceCounters;
pub use lifecycle_port::SignalBranchLifecyclePort;
#[cfg(test)]
pub(in crate::branch) use lifecycle_state::SignalOwnerLifecyclePoisonRecovery;
pub(in crate::branch) use lifecycle_state::{
    SignalOwnerAdmissionDenial, SignalOwnerLifecycleIdentity, SignalOwnerLifecycleState,
    SignalOwnerOperationAdmission,
};
pub use mutation_port::{SignalBranchForkReservation, SignalBranchMutationPort};
pub(crate) use owner::{SignalOwner, SignalOwnerRoot, DEFAULT_MAXIMUM_LIVE_SIGNAL_BRANCHES};

pub use cancellation::{SignalOwnerCancellationSource, SignalOwnerCancellationToken};
pub use counters::SignalOwnerServiceCostSnapshot;
pub use definition_publication_port::{
    AdmittedSignalConditionalDefinitionPublication, SignalConditionalDefinitionPublicationPort,
};
pub use lifecycle_observation::SignalOwnerLifecycleObservation;
#[cfg(feature = "test-operation-control")]
pub use operation_control::{
    SignalOwnerOperationBoundary, SignalOwnerOperationControl, SignalOwnerOperationPause,
};
pub use owner::SignalOwnerServiceIssuanceDenial;
pub use service_ports::SignalOwnerServicePorts;
pub use unavailable::SignalOwnerUnavailable;

#[cfg(test)]
mod tests;
