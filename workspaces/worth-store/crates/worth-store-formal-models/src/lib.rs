#![doc = include_str!("authority_compile_fail_proofs.md")]
#![forbid(unsafe_code)]

//! Cold checked-protocol support for Worth Store.
//!
//! Runtime owners issue outcomes and observations. This crate classifies those
//! observations into finite model actions, declares model assumptions, and
//! invokes checked artifacts. Nothing exported here carries runtime authority.

pub mod assumptions;
#[path = "protocols/import_publication/mod.rs"]
mod import_publication;
mod model_contract;
mod protocol_family;
pub mod protocols;
pub mod runner;

pub use model_contract::{
    protocol_model_contract, FiniteAbstractionRule, ProtocolLivenessContract, ProtocolModelContract,
};

pub use import_publication::{
    ImportPublicationAction, ImportPublicationModel, ImportPublicationModelDenial,
    ImportPublicationState,
};
pub use protocol_family::ProtocolFamily;
pub use protocols::compaction_visibility::{
    map_compaction_observation, map_lsm_maintenance_observation, map_lsm_membership_observation,
    CompactionLifecycleDenial, CompactionLifecycleModel, CompactionLifecycleState,
    CompactionVisibilityAction, LsmExecutionAction, LsmExecutionDenial, LsmMaintenanceAction,
    LsmMaintenanceDenial, LsmMembershipAction, LsmMembershipDenial, ModeledOutcome,
};
pub use protocols::durability_recovery::{
    map_checkpoint_selection, map_failed_wal_fence, map_recovery_completion, map_redo_execution,
    map_redo_generation_denial, map_reopened_physical_recovery, CheckpointFrontierState,
    DirectorySyncFrontierState, DurabilityOwnerMappingDenial, DurabilityRecoveryAction,
    DurabilityRecoveryDenial, DurabilityRecoveryFrontier, PageFrontierState,
    RecoveredRootFrontierState, ReplayFrontierState, WalFrontierState,
};
pub use protocols::lease_reclaim::{
    map_active_lease, map_expiry, map_identity_reuse_attempt, map_owned_copy,
    map_reclaim_eligibility, map_release, map_revocation, LeaseReclaimAction,
    LeaseReclaimActionKind, LeaseReclaimDenial,
};
pub use protocols::operational_recovery::{
    check_operational_recovery_records, map_operational_control_record, OperationalRecoveryAction,
    OperationalRecoveryActionKind, OperationalRecoveryCounterexample, OperationalRecoveryInvariant,
    OperationalRecoveryModel,
};
pub use protocols::quarantine_readmission::{
    map_quarantine_readmission_outcome, map_quarantine_record, QuarantineReadmissionDenial,
    QuarantineReadmissionModel, QuarantineReadmissionOutcomeObservation,
    QuarantineReadmissionState, QuarantineRecordObservation,
};
pub use protocols::replication_admission::{
    map_replication_progress_outcome, map_replication_publication_outcome,
    map_replication_publication_readiness, map_replication_source_admission_outcome,
    ReplicationAdmissionAction,
};
pub use protocols::shared_frontiers::{
    compose_compaction_action, compose_durability_action, compose_import_action,
    compose_lease_action, compose_quarantine_state, compose_replication_action,
    compose_source_precedence_action, SharedAdmissionFrontier, SharedDurabilityFrontier,
    SharedFrontierAction, SharedFrontierDenial, SharedFrontierModel, SharedQuarantineFrontier,
    SharedReachabilityFrontier, SharedVisibilityFrontier,
};
pub use protocols::source_precedence::{
    require_selectable_source, ModeledSourceCandidateRole, SourceAuthorityPosture,
    SourcePrecedenceAction, SourcePrecedenceActionKind, SourcePrecedenceDenial,
};
