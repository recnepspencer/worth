#![doc = include_str!("recovery_physics_compile_fail_proofs.md")]
#![forbid(unsafe_code)]

mod operation_reconciliation;
mod page_redo;
mod recovery_budget;
mod redo_replay;
mod source_precedence;
mod wal_prefix;

/// Expected limits for other crates' tests, minted through each owner's own
/// door. No production build enables `test-support`.
#[cfg(any(test, feature = "test-support"))]
pub mod test_support {
    pub use crate::redo_replay::head_replay_limit_for_test;
    pub use crate::source_precedence::{physics_limit_for_test, root_history_limit_for_test};
}

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
    admit_current_source_copy_publication, admit_physical_redo_members,
    decode_physical_redo_records, physical_redo_observation_target_identities,
    physical_redo_observation_targets, physical_redo_target_identities, plan_physical_redo,
    AdmittedPhysicalRedoMembers, AdmittedRootStepMemberView, ExceededHeadReplayBound,
    HeadReplayBound, HistoricalConsumedOperationSet, HistoricalReleasedDropTargetWitness,
    HistoricalRetiredTargetWitness, HistoricalRetirements, ImmutablePhysicalRedoPlan,
    PhysicalExtentCopyAdmission, PhysicalRedoAdmissionLimits, PhysicalRedoDecision,
    PhysicalRedoDecisionKind, PhysicalRedoDecisionPrior, PhysicalRedoDecisionView,
    PhysicalRedoExtentCoordinate, PhysicalRedoGroupBinding, PhysicalRedoMemberInput,
    PhysicalRedoPlanCounters, PhysicalRedoPlanningDenial, PhysicalRedoProjection,
    PhysicalRedoProjectionLimit, PhysicalRedoRecord, PhysicalRedoTarget,
    PhysicalRedoTargetIdentity, PhysicalRewriteAdmission, RecoveryPageObservation,
    RecoveryPageSource, SelectedReleaseHeadReplayDenial, VerifiedOrderedReleasedHeadReplayV14,
    VerifiedSelectedReleaseHeadReplayV14, VerifiedSelectedTerminalHeadRetirementReplay,
};
pub use source_precedence::{
    admit_physical_page_facts, admit_physical_wal_tail, classify_admitted_wal_segment,
    decide_ordered_root_step_basis, is_retirement_prefix,
    observe_structured_physical_root_candidate, select_current_previous_root,
    select_physical_recovery_sources, AddressedCheckpointBatchControl,
    AddressedReleaseHeadControlV2, AddressedReleasedControlDenial, AdmittedWalFrameRejectionKind,
    AdmittedWalSegmentPolicyInput, CheckpointCoveredWalArtifact, CheckpointRetiredReleaseIntent,
    EffectiveReleaseHeadDenial, ExceededPhysicsBound, ExceededRootHistoryBound,
    HistoricalReleaseRootChainBuilder, HistoricalReleaseRootChainDenial,
    HistoricalReleaseRootPrefixBuilder, ObservedOrdinaryRootMember,
    OrderedHistoricalReleaseCustodyDenial, OrderedRootHistoryBuilder, OrderedRootHistoryDenial,
    OrderedRootStepBasis, OrdinaryRootStepDenial, PageLsnSkipApplyDecision,
    PendingWalReleaseCustodyDenial, PhysicalBootstrapFallbackAnchor, PhysicalCheckpointBase,
    PhysicalCheckpointBaseDenial, PhysicalManifestBlockProjection, PhysicalPageFactDenial,
    PhysicalRecoveryResidue, PhysicalRecoveryResidueKind, PhysicalRecoverySource,
    PhysicalRootCandidateDenial, PhysicalRootManifestDenial, PhysicalRootSelectionDenial,
    PhysicalRootSelectorDenial, PhysicalRootSlotObservation, PhysicalRootSourceCandidate,
    PhysicalSourceSelection, PhysicalSourceSelectionDenial, PhysicalSourceSelectionTrace,
    PhysicalWalCandidatePreparation, PhysicalWalFrameFacts, PhysicalWalInterruptionFacts,
    PhysicalWalSegmentCandidate, PhysicalWalSegmentDisposition, PhysicsBound,
    ReleasedDirectoryReplacementDenial, ReleasedInventoryParts, ReleasedInventoryView,
    ReleasedV3InventoryTransitionDenial, RetirementReleaseIntent, RootHistoryBound,
    SelectedCompactionProduct, SelectedCustodyDenial, SelectedHeadRosterAdmissionDenial,
    SelectedNoReleaseCustodyDenial, SelectedPhysicalPageFacts, SelectedPhysicalRoot,
    SelectedPhysicalRootRole, SelectedPhysicalWalTail, SelectedPhysicalWalTailDenial,
    SelectedTierCustodyDenial, SelectedTierEpochCustodySource,
    VerifiedAddressedCheckpointReleaseBase, VerifiedAddressedReleasedControlFrame,
    VerifiedCheckpointReleaseHeadRosterV2, VerifiedEffectiveReleaseHeadRosterV14,
    VerifiedHistoricalPendingWalBatch, VerifiedHistoricalReleaseRootChain,
    VerifiedHistoricalReleaseRootPrefix, VerifiedOrderedHistoricalReleaseCustody,
    VerifiedOrderedPendingWalReleaseBatch, VerifiedOrderedRootEdge, VerifiedOrderedRootHistory,
    VerifiedOrdinaryRootStep, VerifiedPendingWalReleaseCustody,
    VerifiedReleasedDirectoryReplacement, VerifiedReleasedRootEdge,
    VerifiedReleasedV3InventoryTransition, VerifiedRetirementRootEdge,
    VerifiedSelectedCheckpointCustody, VerifiedSelectedNoReleaseCustody,
    VerifiedSelectedReleaseHeadCustodyV2, VerifiedSelectedTierEpochCustody,
    WitnessedSelectedControlFrame,
};
