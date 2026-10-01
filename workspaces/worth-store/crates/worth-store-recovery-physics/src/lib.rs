#![doc = include_str!("recovery_physics_compile_fail_proofs.md")]
#![forbid(unsafe_code)]

mod operation_reconciliation;
mod page_redo;
mod recovery_budget;
mod redo_replay;
mod source_precedence;
mod wal_prefix;

pub use operation_reconciliation::{
    classify_binding_freshness, reconcile_materialized_operation_fates, reconcile_operation_fates,
    OperationReconciliationDenial, ReconciledOperationFate, ReconciledOperationFates,
    RecoveryBindingFreshness, RecoveryOperationEvidenceInput, RecoveryOperationFate,
    RecoveryOperationIdentity,
};
pub use page_redo::{
    PageLsn, PageRedoApplicationBasis, PageRedoCounterSnapshot, PageRedoDenial, PageRedoDenialKind,
    PageRedoDigestState, PageRedoEligibility, PageRedoEligibilityKind,
};
pub use recovery_budget::{
    admit_recovery_plan_cost, RecoveryPlanCost, RecoveryPlanCostDenial, RecoveryPlanLimits,
    RecoveryPlanningCounters,
};
pub use redo_replay::{
    admit_physical_redo_members, decode_physical_redo_records,
    physical_redo_observation_target_identities, physical_redo_observation_targets,
    physical_redo_target_identities, plan_physical_redo, AdmittedPhysicalRedoMembers,
    AdmittedRootStepMemberView, HistoricalConsumedOperationSet,
    SelectedReleaseHeadReplayDenial, VerifiedOrderedReleasedHeadReplayV14,
    VerifiedSelectedReleaseHeadReplayV14,
    HistoricalReleasedDropTargetWitness, ImmutablePhysicalRedoPlan, PhysicalExtentCopyAdmission,
    PhysicalRedoAdmissionLimits, PhysicalRedoDecision, PhysicalRedoDecisionKind,
    PhysicalRedoDecisionPrior, PhysicalRedoDecisionView, PhysicalRedoExtentCoordinate,
    PhysicalRedoGroupBinding, PhysicalRedoMemberInput, PhysicalRedoPlanCounters,
    PhysicalRedoPlanningDenial, PhysicalRedoProjection, PhysicalRedoRecord, PhysicalRedoTarget,
    PhysicalRedoTargetIdentity, PhysicalRewriteAdmission, RecoveryPageObservation,
    RecoveryPageSource,
};
pub use source_precedence::{
    admit_physical_page_facts, admit_physical_wal_tail, classify_admitted_wal_segment,
    observe_structured_physical_root_candidate, select_current_previous_root,
    select_physical_recovery_sources, AdmittedWalFrameRejectionKind, AdmittedWalSegmentPolicyInput,
    CheckpointCoveredWalArtifact, PageLsnSkipApplyDecision, PendingWalReleaseCustodyDenial,
    PhysicalBootstrapFallbackAnchor, PhysicalCheckpointBase, PhysicalCheckpointBaseDenial,
    PhysicalManifestBlockProjection, PhysicalPageFactDenial, PhysicalRecoveryResidue,
    PhysicalRecoveryResidueKind, PhysicalRecoverySource, PhysicalRootCandidateDenial,
    PhysicalRootManifestDenial, PhysicalRootSelectionDenial, PhysicalRootSelectorDenial,
    PhysicalRootSlotObservation, PhysicalRootSourceCandidate, PhysicalSourceSelection,
    PhysicalSourceSelectionDenial, PhysicalSourceSelectionTrace, PhysicalWalFrameFacts,
    PhysicalWalInterruptionFacts, PhysicalWalSegmentCandidate, PhysicalWalSegmentDisposition,
    HistoricalReleaseRootChainBuilder,
    HistoricalReleaseRootChainDenial,
    HistoricalReleaseRootPrefixBuilder,
    ObservedOrdinaryRootMember, OrdinaryRootStepDenial, ReleasedInventoryView,
    ReleasedV3InventoryTransitionDenial,
    SelectedCompactionProduct,
    SelectedCustodyDenial, SelectedNoReleaseCustodyDenial, SelectedPhysicalPageFacts,
    SelectedPhysicalRoot, SelectedPhysicalRootRole, SelectedPhysicalWalTail,
    SelectedPhysicalWalTailDenial, SelectedTierCustodyDenial, SelectedTierEpochCustodySource,
    VerifiedHistoricalPendingWalBatch, VerifiedHistoricalReleaseRootChain,
    VerifiedOrdinaryRootStep, VerifiedOrderedPendingWalReleaseBatch,
    OrderedHistoricalReleaseCustodyDenial, VerifiedOrderedHistoricalReleaseCustody,
    VerifiedPendingWalReleaseCustody, VerifiedEffectiveReleaseHeadRosterV14,
    EffectiveReleaseHeadDenial,
    VerifiedHistoricalReleaseRootPrefix,
    VerifiedReleasedV3InventoryTransition,
    AddressedReleasedControlDenial, VerifiedAddressedReleasedControlFrame,
    OrderedRootHistoryBuilder, OrderedRootHistoryDenial, VerifiedOrderedRootEdge,
    VerifiedOrderedRootHistory, VerifiedReleasedRootEdge,
    AddressedCheckpointBatchControl, VerifiedAddressedCheckpointReleaseBase,
    VerifiedSelectedCheckpointCustody,
    AddressedReleaseHeadControlV2, VerifiedCheckpointReleaseHeadRosterV2,
    VerifiedSelectedReleaseHeadCustodyV2,
    VerifiedSelectedNoReleaseCustody,
    VerifiedSelectedTierEpochCustody, WitnessedSelectedControlFrame,
};
