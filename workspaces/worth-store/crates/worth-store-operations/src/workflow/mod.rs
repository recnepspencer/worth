mod backup;
mod owner_receipt_persistence;
mod point_in_time_recovery;
mod recovery_owner_plan;
mod recovery_replay;
mod repair;
mod replica_bootstrap;
mod replica_promotion;
mod restore;
mod rollback;

pub(crate) use owner_receipt_persistence::persist_recovery_owner_receipts;
#[cfg(any(test, feature = "certification-test-authority"))]
pub(crate) use repair::{
    certification_authority_repair_candidates_from_backup_observation,
    certification_authority_repair_from_backup_observation,
    certification_derived_maintenance_from_fixture_observation,
};

pub use backup::{
    admit_backup_for_production_restore, qualify_backup_custody,
    record_independent_backup_verification, recover_online_backups, AdmittedOnlineBackup,
    BackupAbandonmentDenial, BackupAbandonmentFailure, BackupCustodyQualificationDenial,
    BackupLeasePersistenceDenial, BackupLeasePersistenceFailure, BackupMaterializationAbandonment,
    BackupMaterializationAbandonmentDenial, BackupMaterializationAbandonmentRetry,
    BackupMaterializationCompletion, BackupMaterializationDenial,
    BackupMaterializationRecordDenial, BackupMaterializationSession, BackupPublicationSession,
    BackupSourceVerificationDenial, BackupVerificationJoinDenial, CustodyQualifiedBackupBundle,
    IndependentlyVerifiedBackup, OnlineBackupAdmissionDenial, OnlineBackupIntent,
    OnlineBackupReadmissionDenial, OnlineBackupReadmissionFailure,
    ProductionRestoreAdmissibleBackupBundle, RecoverableOnlineBackup,
    UnpersistedBackupReachabilityLease, UnrecordedBackupMaterialization,
    UnreleasedIndependentBackupVerification,
};
pub use point_in_time_recovery::{
    AdmittedPitrSourceOperation, AuthorizedPointInTimeRecoveryPlan,
    EvidenceBoundPointInTimeRecoveryPlan, ExactRecoveryFrontier, ExecutedPointInTimeRecovery,
    ExecutionReadyPointInTimeRecovery, FrontierPartialOrder, LoweredPointInTimeRecoveryPlan,
    PitrCandidatePosture, PitrCandidateSelectionDenial, PitrExecutionDenial, PitrLoweringDenial,
    PitrReadinessDenial, PitrResolutionDenial, PitrRoundingPolicy, PitrSourceAdmissionDenial,
    PointInTimeCandidate, PointInTimeCandidateSet, PointInTimeRecoveryIntent,
    PointInTimeRecoveryOperationReceipt, PointInTimeRecoveryReceipt, PointInTimeReplayDenial,
    PointInTimeReplayOwner, PointInTimeReplayPlan, PointInTimeReplayRequest,
    PointInTimeReplaySourceCoordinates, RecoveryTimelineAdmission, RecoveryTimelineObservation,
    RecoveryTimelineOwner, ResolvedPitrCandidate,
};
pub use recovery_replay::{
    StagedWalApplicationDenial, StagedWalApplicationPort, StagedWalApplicationProviderReceipt,
    StagedWalApplicationReceipt, StagedWalApplicationRequest, StagedWalReplaySourceDenial,
    StagedWalReplaySourceReceipt,
};
pub use repair::{
    AuthorityAffectingRepairExecutionDenial, AuthorityAffectingRepairLoweringDenial,
    AuthorityAffectingRepairReadinessDenial, AuthorityAffectingStagedRepairPlan,
    AuthorizedAuthorityAffectingRepairPlan, AuthorizedRepairPlan,
    CurrentAuthorityPreservingMaintenancePlan, EvidenceBoundRepairPlan,
    ExecutedAuthorityAffectingRepair, ExecutedRepair, ExecutedRepairOwnerReceipt,
    ExecutedRepairOwnerReceiptDag, ExecutionReadyAuthorityAffectingRepair, ExecutionReadyRepair,
    IntegrityRepairClassificationDenial, IntegrityRepairClassificationReceipt,
    LoweredAuthorityAffectingRepairOwnerPlanDag, LoweredRepairOwnerPlanDag, RepairCandidateSet,
    RepairExecutionBoundary, RepairExecutionBoundaryMoment, RepairExecutionControlPort,
    RepairExecutionDenial, RepairExecutionDisposition, RepairExecutionInterrupted,
    RepairExecutionInterruptionCause, RepairIntent, RepairJournalDenial, RepairLoweringDenial,
    RepairPlanExplanation, RepairReadinessDenial, RepairResolutionDenial,
    UnrecoverableDamageReport,
};
pub use replica_bootstrap::{
    AbandonedReplicaBootstrap, AuthorizedReplicaBootstrapPlan, CompletedReplicaBootstrap,
    EvidenceBoundReplicaBootstrapPlan, ExecutedReplicaBootstrap, ExecutionReadyReplicaBootstrap,
    LoweredReplicaBootstrapOwnerPlanDag, PostVerifiedReplicaBootstrap, RecoveredReplicaBootstrap,
    RecoveredTerminalReplicaBootstrap, ReplicaBootstrapExecutionDenial,
    ReplicaBootstrapFinalizationDenial, ReplicaBootstrapIntent, ReplicaBootstrapLoweringDenial,
    ReplicaBootstrapPersistenceDenial, ReplicaBootstrapReadinessDenial,
    ReplicaBootstrapResolutionDenial, ReplicaBootstrapResume, TransferredReplicaBootstrap,
};
pub use replica_promotion::{
    AuthorizedReplicaPromotionPlan, CompletedOldPrimaryRejoin, CurrentReplicaPromotion,
    DurablyFencedReplicaPromotion, EvidenceBoundReplicaPromotionPlan, ExecutedReplicaPromotion,
    ExecutionReadyReplicaPromotion, FencedReplicaPromotion, GovernedOldPrimaryRejoinPlan,
    LoweredReplicaPromotionOwnerPlanDag, PostVerifiedReplicaPromotion, PublishedReplicaPromotion,
    RecoveredReplicaPromotion, ReplicaPromotionExecutionDenial,
    ReplicaPromotionFencePersistenceDenial, ReplicaPromotionFencingDenial,
    ReplicaPromotionFinalizationDenial, ReplicaPromotionIntent, ReplicaPromotionLoweringDenial,
    ReplicaPromotionPublicationDenial, ReplicaPromotionPublicationPort,
    ReplicaPromotionPublicationReceipt, ReplicaPromotionPublicationRequest,
    ReplicaPromotionReadinessDenial, ReplicaPromotionResolutionDenial, ReplicaPromotionResume,
    ResolvedOldPrimaryRejoin,
};
pub use restore::{
    AuthorizedBackupRestorePlan, BackupRestoreExecutionDenial, BackupRestoreIntent,
    BackupRestoreLoweringDenial, BackupRestoreReadinessDenial, BackupRestoreReplayDenial,
    BackupRestoreReplayOwner, BackupRestoreReplayPlan, BackupRestoreReplayRequest,
    EvidenceBoundBackupRestorePlan, ExecutedBackupRestore, ExecutionReadyBackupRestore,
    LoweredBackupRestorePlan, RecoveredBackupFrontierReceipt, RestoreExecutionReceipt,
};
pub use rollback::{
    AdmittedRollbackSourceOperation, AuthorizedRollbackPlan, EvidenceBoundRollbackPlan,
    ExecutedRollback, ExecutionReadyRollback, LoweredRollbackPlanDag, ResolvedRollbackCandidate,
    ResolvedRollbackOperation, RollbackExecutionDenial, RollbackExecutionReceipt, RollbackIntent,
    RollbackLoweringDenial, RollbackOperationReceipt, RollbackReadinessDenial,
    RollbackReplayDenial, RollbackReplayOwner, RollbackReplayPlan, RollbackResolutionDenial,
    RollbackSourceAdmissionDenial,
};
