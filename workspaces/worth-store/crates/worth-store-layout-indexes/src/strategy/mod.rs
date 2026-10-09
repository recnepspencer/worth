mod admission;
mod admitted_strategy;
mod authority_basis;
pub(crate) mod btree;
mod capability;
mod capability_queries;
mod counter_evidence;
mod counter_path;
mod counter_planning;
mod declaration;
mod denial;
mod family;
mod invariant_suite;
mod key_law_validation;
mod lsm;
mod materialization_policy;
mod posture;
pub(crate) mod registry;
#[cfg(test)]
pub(crate) mod tests;
#[cfg(test)]
pub(crate) mod tests_support;

pub(crate) use admission::admit_strategy_from_basis;
pub use admitted_strategy::AdmittedLayoutStrategy;
pub(crate) use authority_basis::StrategyAuthorityBasis;
pub use btree::{
    BTreeCorruptionRegion, BTreeInvariantSuite, BTreeLookupBranch, BTreeNodeFormatLaw,
    BTreeRebuildMigrationLaw, BTreeRootPublicationLaw, BTreeSearchPathLaw, BTreeSeparatorLaw,
    BTreeSiblingLinkLaw, BTreeSplitMergeLaw, BTreeStableReadLaw, BTreeTombstoneLaw,
};
pub use counter_evidence::StrategyCounterEvidence;
pub(crate) use counter_planning::planned_counter_envelope_for;
pub(crate) use declaration::StrategyDeclaration;
pub use denial::StrategyDenial;
pub use family::LayoutStrategyFamily;
pub use invariant_suite::{
    StrategyCounterProfile, StrategyIntegrityInvariant, StrategyInvariantSuite,
    StrategyLookupInvariant, StrategyMutationInvariant, StrategyPublicationInvariant,
    StrategyRecoveryInvariant,
};
pub use lsm::{
    baseline_lsm_lookup_admission_cases, baseline_lsm_lookup_cases, lsm_compaction_runtime,
    lsm_execution_owner_case_inventory, lsm_lookup_runtime, lsm_physical_compaction_runtime,
    lsm_publication_runtime, lsm_replay_runtime, lsm_strategy, AdmittedLsmCompactionDemand,
    BaselineLsmCompactionAdmission, BaselineLsmCompactionKeyIdentity, BaselineLsmCompactionPlan,
    BaselineLsmCompactionPublicationReceipt, BaselineLsmCompactionRecordIdentity,
    BaselineLsmCompactionRecordKind, BaselineLsmCompactionTransition,
    BaselineLsmCounterObservation, BaselineLsmExecutionAdmissionDenial,
    BaselineLsmExecutionAdmissionDenialKind, BaselineLsmLookupAbsence, BaselineLsmLookupAdmission,
    BaselineLsmLookupAdmissionCaseId, BaselineLsmLookupAdmissionOutcome,
    BaselineLsmLookupAdmissionView, BaselineLsmLookupCaseId, BaselineLsmLookupCounterReceipt,
    BaselineLsmLookupDisposition, BaselineLsmLookupExecution, BaselineLsmLookupSource,
    BaselineLsmLookupView, BaselineLsmManifestPublicationExecution,
    BaselineLsmMembershipObservation, BaselineLsmReplayAdmission, BaselineLsmReplayExecution,
    BaselineLsmRunIdentity, BaselineLsmRunPublicationAdmission, InterlockedLsmCompaction,
    LsmAdvisoryFilterLaw, LsmCompactionPreparationOutcome, LsmCompactionPreparationView,
    LsmCompactionPublicationOutcome, LsmCompactionPublicationView, LsmCompactionRuntime,
    LsmExecutionDisposition, LsmExecutionOperation, LsmExecutionOwnerCaseDeclaration,
    LsmExecutionOwnerCaseId, LsmExecutionOwnerCaseObservation, LsmInvariantSuite,
    LsmLookupDisposition, LsmLookupRuntime, LsmMembershipActivationOutcome,
    LsmMembershipActivationView, LsmMemtableWalLaw, LsmPhysicalCompactionBindingOutcome,
    LsmPhysicalCompactionBindingView, LsmPhysicalCompactionIntent, LsmPhysicalCompactionRuntime,
    LsmPublicationRuntime, LsmReplayExecutionOutcome, LsmReplayExecutionView, LsmReplayRuntime,
    LsmRunPublicationLaw, LsmStaleRunCleanupLaw, LsmStrategy, LsmTombstoneLaw,
    LsmWriteAmplificationLaw, PreparedLsmCompaction, PublishedLsmCompaction,
};
pub use posture::{
    StrategyAmplificationProfile, StrategyCorruptionIsolationBehavior, StrategyLocalityProfile,
    StrategyMaterializationPosture, StrategyRebuildSourceRequirement,
};
pub use registry::LayoutStrategyRegistrySnapshot;
