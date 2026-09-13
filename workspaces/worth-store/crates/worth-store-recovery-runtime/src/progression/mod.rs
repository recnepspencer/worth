mod admitted;
mod completion;
mod discovered;
mod integrity_evidence;
mod namespace_durable;
mod planned;
mod reopened;
mod selected;
mod staged;

pub(crate) use planned::{
    derive_execution_basis, requires_successor_candidate, CandidateMaterializationCost,
    ExecutionBasisDenial, RecoveryObservedCandidateArtifact, RecoveryObservedSuccessorCandidate,
    RecoverySelectedSegmentPage, RecoverySelectedSourceInventory,
};

pub use admitted::AdmittedPhysicalRecovery;
pub use completion::RecoveryCompletion;
#[cfg(feature = "certification-test-authority")]
pub use completion::{complete_recovery, RecoveryCompletionDenial};
pub use discovered::{DiscoveredPhysicalRecovery, PhysicalRecoveryDiscoveryCounters};
pub(crate) use integrity_evidence::RecoveryIntegrityEvidence;
pub use namespace_durable::NamespaceDurablePhysicalRecovery;
pub(crate) use namespace_durable::NamespaceDurableState;
pub use planned::{
    PhysicalRecoveryStagingCancellation, PlannedPhysicalRecovery, RecoveryBaseImageAction,
    RecoveryBaseImagePlan, RecoveryPayloadManifestAction, RecoveryPublicationAction,
    RecoveryPublicationCandidateArtifact, RecoveryPublicationExpectation, RecoveryPublicationPlan,
    RecoveryQuiescencePlan, RecoverySegmentRoutingAction, RecoveryStagingAction,
    RecoveryStagingCommandPlan, RecoveryStagingLayoutPlan, RecoveryStagingRedoStep,
};
pub use reopened::ReopenedPhysicalRecovery;
pub use selected::SelectedPhysicalRecovery;
pub use staged::{ClosedRecoveryStagingGeneration, StagedPhysicalRecovery};
