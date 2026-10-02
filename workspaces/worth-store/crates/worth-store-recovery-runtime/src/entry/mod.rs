mod admission;
mod authority;
mod authority_binding;
mod configuration;
mod counters;
mod integrity_observations;
mod limits;
mod outcome;
mod publication;
mod reopen;
mod request;
mod root_protocol_counters;
mod session;
mod source_denial;
mod staging;
mod successor_candidate;

pub use authority::{PhysicalRecoveryPlatformAdmissionError, PhysicalRecoveryPlatformAuthority};
pub use authority_binding::PhysicalRecoveryEntryBindingDrift;
pub use configuration::{PhysicalRecoveryConfigurationDenial, PhysicalRecoveryStaticConfiguration};
pub use counters::PhysicalRecoveryAdmissionCounters;
pub use integrity_observations::{
    PhysicalRecoveryIntegrityObservations, PhysicalRecoveryWalIntegrityObservation,
    PhysicalRecoveryWalIntegrityObservationOutcome,
};
pub use limits::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDenial, PhysicalRecoveryLimits,
};
pub use outcome::{
    HistoricalDropAdmissionStage, PhysicalRecoveryBlock, PhysicalRecoveryBlockEvidence,
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure,
    PhysicalRecoveryOrderedReleaseDenial, PhysicalRecoveryOrderedReleaseJoin,
    PhysicalRecoveryOrderedReleaseStorage, PhysicalRecoveryOutcome,
    PhysicalRecoveryPageAdmissionDenial, PhysicalRecoveryPlanningDenial,
    PhysicalRecoveryPublicationIndeterminate, PhysicalRecoveryRefusal, PhysicalRecoveryRefusalKind,
    PhysicalRecoveryReleaseHeadControlDenial, PhysicalRecoveryReleaseHeadReadDenial,
    PhysicalRecoveryReleaseHeadWalkDenial, PhysicalRecoverySelectedRecordReadDenial,
    PhysicalRecoverySelectedReleaseHeadDenial,
};
pub use publication::{
    PhysicalRecoveryPublicationCounters, PhysicalRecoveryPublicationDenial,
    PhysicalRecoveryPublicationSettlement, PhysicalRecoveryPublicationSettlementLedger,
};
pub use reopen::{PhysicalRecoveryReopenCounters, PhysicalRecoveryReopenFailure};
pub use request::PhysicalRecoveryOpenRequest;
pub use root_protocol_counters::PhysicalRecoveryRootProtocolCounters;
pub use session::PhysicalRecoverySessionIdentity;
pub use source_denial::{
    PhysicalManifestObservationDenial, PhysicalRecoveryCheckpointIntegrityDenial,
    PhysicalRecoveryMediaObservationFailure, PhysicalRecoveryRootProtocolArtifact,
    PhysicalRecoveryRootProtocolDenial, PhysicalRecoverySourceDenial,
    PhysicalRecoverySourceReadAllocationBoundary, PhysicalRecoverySourceReadAllocationDenial,
    PhysicalRecoveryWalIntegrityDenial, PhysicalRecoveryWalInventoryAllocationBoundary,
};
pub use staging::{
    PhysicalRecoveryStagingCounters, PhysicalRecoveryStagingDenial,
    PhysicalRecoveryStagingSettlement, PhysicalRecoveryStagingSettlementLedger,
};
pub use successor_candidate::{
    PhysicalRecoverySuccessorCandidateDenial, PhysicalRecoverySuccessorCandidateMismatch,
};

pub(crate) use authority::{AdmittedPlatformAdmission, AdmittedPlatformAuthority};
pub(crate) use counters::{
    record_binding_comparison, record_binding_denial, record_coordinator_created,
    snapshot as counter_snapshot,
};
pub(crate) use integrity_observations::WalIntegrityObservationBuilder;
pub(crate) use session::{RecoveredRecoverySessionReceipt, RecoverySession};
