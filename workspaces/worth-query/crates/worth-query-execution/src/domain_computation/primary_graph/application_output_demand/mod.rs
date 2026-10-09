pub(in crate::domain_computation::primary_graph) use registry::ConsumerCustody;
mod currentness;
mod readiness_delivery;
mod recovered_outputs;
mod registry;
mod settlement;

pub use readiness_delivery::WorthQueryOutputReadinessDeliveryEvidence;
pub(super) use recovered_outputs::WorthQueryRecoveredOutputs;
pub(in crate::domain_computation::primary_graph) use registry::HeldRequiredSuccessor;
pub(in crate::domain_computation::primary_graph) use registry::PendingUpstream;
pub(in crate::domain_computation::primary_graph) use registry::PreparedPrerequisiteClaims;
pub(in crate::domain_computation::primary_graph) use registry::ReplacedRequiredWorkHint;
pub(in crate::domain_computation::primary_graph) use registry::RequestedOutputReadClaims;
pub(in crate::domain_computation::primary_graph) use registry::RequiredWorkMembership;
pub(in crate::domain_computation::primary_graph) use registry::RetainedOutputReadmissionSource;
pub(in crate::domain_computation::primary_graph) use registry::SelectedOutputAdmission;
pub(in crate::domain_computation::primary_graph) use registry::SelectedReadyReadmission;
pub(in crate::domain_computation::primary_graph) use registry::SelectedRequiredRefreshClaim;
pub(in crate::domain_computation::primary_graph) use registry::SelectedRequiredWork;
pub(in crate::domain_computation::primary_graph) use registry::SelectedRequiredWorkKind;
pub use registry::WorthQueryOutputDemandNotifications;
#[cfg(feature = "test-query-execution-observer")]
pub use registry::{
    required_custody_layout_for_test, required_handoff_custody_bytes_for_test,
    required_ready_custody_bytes_for_test, RequiredCustodyLayoutForTest,
};
pub(super) use registry::{
    AcceptedCheckpointFactSource, BoundOutputSource, DemandAdmissionKind, OutputRefreshPredecessor,
    OutputRowStage, PreparedOutputRootKind, PreparedReadyBacking, PreparedSelectedCheckpointFinish,
    ReadyCompletion, RequiredOutputCustodyCapacity, SelectedCheckpointFinishStop,
    WorthQueryAcceptedOutputAuthority, WorthQueryAcceptedOutputCheckpointIdentity,
    WorthQueryAcceptedOutputCheckpointPosture, WorthQueryCompletedOutputDemand,
    WorthQueryOutputCheckpoint, WorthQueryOutputClaimIdentity,
    WorthQueryOutputDemandAdvanceAdmission, WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandKey, WorthQueryOutputDemandRegistry, WorthQueryOutputSchedulingResult,
    WorthQueryPendingOutputDelivery, WorthQueryPerformedOutputDemandSource,
    WorthQueryReadmittedAcceptedOutput, WorthQueryRequiredOutputSourcePreparation,
    WorthQueryRestoredAcceptedOutput,
};
pub(in crate::domain_computation) use registry::{
    RequiredOutputDemandContext, RequiredOutputExecution,
};
pub use settlement::{WorthQueryOutputDemandSettlement, WorthQueryOutputSettlementPosture};

#[cfg(feature = "test-query-execution-observer")]
pub(in crate::domain_computation::primary_graph) use registry::join_observation::{
    observe_required_refresh_stop, observe_retained_source_selection, observe_stable_join_admission,
};
