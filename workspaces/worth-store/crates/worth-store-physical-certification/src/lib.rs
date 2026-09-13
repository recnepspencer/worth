#![forbid(unsafe_code)]

pub mod harness;

mod actors;
mod authoring;
#[cfg(test)]
mod c9_integrity_localization;
mod counters;
mod drivers;
mod faults;
mod fixtures;
mod fresh_process_offline_truth;
mod observation;
mod operational_recovery_audit_driver;
#[cfg(test)]
mod operational_recovery_authorization_fixture;
mod operational_recovery_control_driver;
mod operational_recovery_control_transition;
mod operational_recovery_crash_evidence;
mod operational_recovery_driver;
#[cfg(test)]
mod operational_recovery_driver_tests;
mod operational_recovery_process_crash;
mod operational_recovery_rejoin_driver;
#[cfg(test)]
mod operational_recovery_replica_driver_fixture;
#[cfg(test)]
mod operational_recovery_replica_driver_tests;
#[cfg(test)]
mod operational_recovery_replica_promotion_driver_tests;
mod operational_recovery_trace;
mod operational_recovery_yieldpoint;
mod oracles;
mod physical_isolation_entry;
mod planning;
mod pressure_harness;
mod process_probe;
mod qualification;
mod scenario;
mod scenarios;
mod schedule;
mod transcript;

pub use scenarios::*;
pub use worth_store_offline_verifier::OfflineVerifierBoundarySeam;

pub use actors::{
    BlobDedupeActor, BlobExportActor, BlobImportActor, BlobIngestActor,
    BlobPartialReplicationActor, BlobPlacementMoveActor, BlobReadActor, BlobReclaimActor,
    BlobResumeActor, BlobVerifyActor, CheckpointActor, CompactionActor, ForegroundReadActor,
    ForegroundWriteActor, OfflineVerifierActor, PhysicalSimulationActor,
    PhysicalSimulationActorAdmissionDenial, ReclaimActor, RecoveryActor, ScrubActor,
};
pub use authoring::{
    physical_scenario, PhysicalScenarioBuilder, ScenarioBuilderActorStep,
    ScenarioBuilderExpectationStep, ScenarioBuilderFixtureStep, ScenarioBuilderScheduleStep,
};
pub use counters::{
    admit_physical_counter_evidence, reject_hostile_counter_evidence_for_readmission,
    CounterContractDenial, CounterContractKind, CounterExpectationDenial, CounterExpectationKind,
    CounterExpectationStrength, CounterMismatchEvidence, CounterStrengthJustification,
    CounterStrengthPosture, HostileCounterEvidenceRow, HostileResourceEnvelopeObservation,
    OverExactCounterDenied, PhysicalCounterContract, PhysicalCounterEvidenceReceipt,
    PhysicalCounterEvidenceRow, PhysicalCounterExecutionSources, PhysicalCounterExpectation,
    PhysicalExecutedCounterEvidence, PhysicalResidencyEvidenceSource, PhysicalResourceEnvelope,
    PhysicalResourceEnvelopeObservation, RequiredCounterContractSet,
};
#[cfg(any(test, feature = "certification-test-support"))]
pub(crate) use counters::{observe_real_store_residency, CertificationResidencyWorkload};
pub use drivers::{
    private_mutation_driver_attempt, test_support_verdict_driver_attempt,
    AdmittedDriverContractSet, AdversarialStorageBoundaryDriver, CrashRuntimeIsolationDriver,
    DriverAdmissionDenial, DriverBoundaryKind, DriverCapabilityProfile, DriverEvidenceSurface,
    DriverFaultClass, IoPressureDriver, MemoryPressureDriver, OfflineVerifierDriver,
    PhysicalBoundarySeam, PhysicalBoundaryYieldpoint, PhysicalSimulationDriver,
    ProductionBoundaryDriverTrace, ProductionStorageBoundaryDriver, YieldpointDeclaration,
    YieldpointObservationReceipt, YieldpointPauseReceipt, YieldpointResumeReceipt,
    YieldpointScheduleBinding,
};
pub use faults::{
    physical_isolation_stable_read_plan_fault_event, BlockedReclaimEvent,
    BoundaryObservedFaultDeliveryRecipe, ByteCorruptionEvent, CrashEvent, DelayedReleaseEvent,
    DroppedFlushEvent, ExecutionReadyFaultDeliveryRecipe, ExecutionTimeReferenceDiscoveryEvent,
    ExpectedFaultLocalization, FaultDeliveryAttempt, FaultDeliveryBoundaryProof,
    FaultDeliveryDenial, FaultDeliveryPlan, FaultDeliveryReceipt, FaultObservedBoundaryKind,
    IoStallEvent, LoweredFaultDeliveryRecipe, NoFaultControlEvent, NoFaultProductionBoundaryParity,
    ObservedFaultBoundary, PhysicalArtifactFaultLocus, PhysicalArtifactKind, PhysicalFaultEvent,
    PhysicalFaultEventKind, PhysicalFaultFieldKind, PhysicalFaultOffset,
    PhysicalStorageFaultExecution, PhysicalStorageFaultInjection, ReorderedPersistenceEvent,
    StaleGenerationEvent, TornWriteEvent, UnboundedReadPlanFootprintEvent,
};
pub use fixtures::{
    FixtureActivityScale, FixtureAuthorityReceipt, FixtureCapabilityDeclaration,
    FixtureConstructionAuthority, FixtureConstructionBasis, FixtureConstructionProofBasis,
    FixtureMutationBoundary, FixtureMutationBoundarySet, FixtureNeedsBoundary,
    FixtureNeedsMaterialization, FixtureProfileNonClaim, FixtureProvenance,
    FixtureScaleDeclaration, FixtureStorageScale, LargeStoreFixtureProfile,
    MaterializedFixtureScaleEvidence, PersistedStoreFixtureManifest,
    PhysicalArtifactFixtureCatalog, PhysicalFixtureBuilder, ProductionBackedFixtureMaterialization,
    ProductionBackedFixtureSource, ProductionBackedPhysicalFixture,
    ResolvedFixtureConstructionRecipe, StoreFixtureAuthority, SyntheticFixtureAuthorityDenied,
};
pub use fresh_process_offline_truth::{
    write_offline_truth_observation_from_environment, FreshProcessDestroyedPrimaryCertification,
    FreshProcessDestroyedPrimaryEvidence, FreshProcessOfflineTruthBaseline,
    FreshProcessOfflineTruthDenial, FreshProcessOfflineTruthRunner, OFFLINE_TRUTH_CHALLENGE_ENV,
    OFFLINE_TRUTH_REPORT_ENV, OFFLINE_TRUTH_ROLE_ENV, OFFLINE_TRUTH_TARGET_ENV,
};
pub use harness::blob::{
    lower_blob_simulation_seed_plan, BlobHarnessLoweredSeedPlan, BlobHarnessLoweringDenial,
    BlobHarnessMaterializedProfile, BlobHarnessOracleObservation, BlobHarnessProfile,
    BlobHarnessProfileSet, BlobHarnessScenarioSeed, BlobHarnessScenarioSeedBuilder,
    BlobHarnessShortcutAttempt, BlobHarnessShortcutDenial, BlobResumeCrashPoint,
    BlobResumeExpectedOutcome, BlobResumeRecoveryScenario,
};
pub use observation::{
    CheckpointCrashReplayObservation, CheckpointInterlockObservation,
    CheckpointPublicationRecoveryExecution, CompactionInterlockObservation,
    IndependentVerifierObservation, IndependentVerifierObservationKind, ObservationDenial,
    ObservedPhysicalTrace, PhysicalIsolationCheckpointPublicationCrashLaneOutput,
    PhysicalIsolationCheckpointPublicationLaneBinding,
    PhysicalIsolationCheckpointPublicationRecoveryOutcomeLaneOutput,
    PhysicalIsolationCheckpointPublicationScheduledLaneOutput, PhysicalObservationBuilder,
    PhysicalSimulationBoundaryObservation, PhysicalSimulationObservationBasis,
    PhysicalSimulationObserver, RecoveryOutcomeKind, RecoveryOutcomeObservation,
    ShortcutRejectionObservation, ShortcutRejectionObservationKind,
};
pub use operational_recovery_control_driver::DrivenOperationalControlStore;
pub use operational_recovery_control_transition::OperationalRecoveryControlTransitionKind;
pub use operational_recovery_crash_evidence::{
    OperationalRecoveryCrashCutDenial, OperationalRecoveryCrashCutEvidence,
};
pub use operational_recovery_driver::{
    DrivenOperationalTransition, OperationalRecoveryProductionDriver, OperationalRecoveryYieldpoint,
};
pub use operational_recovery_process_crash::{
    write_reopen_observation_from_environment, OperationalRecoveryControlCutRequest,
    OperationalRecoveryFreshProcessRunner, OperationalRecoveryProcessCrashConfig,
    OperationalRecoveryProcessCrashDenial, OperationalRecoveryProcessCrashEvidence,
    PROCESS_CRASH_CHALLENGE_ENV, PROCESS_CRASH_REPORT_ENV, PROCESS_CRASH_ROLE_ENV,
    PROCESS_CRASH_YIELDPOINT_ENV,
};
pub use operational_recovery_trace::{
    OperationalRecoveryDriverTrace, OperationalRecoveryTraceJoinDenial,
};
pub use oracles::{
    oracle_verdict_topology, BlobByteEqualityOracle, BlobChunkOrderingOracle,
    BlobConstantMemoryOracle, BlobDigestChecksumDistinctionOracle, BlobHeavyCleanupOracle,
    BlobHeavyPatternLaneOracle, BlobHeavyQualificationEvidenceOracle, BlobNoCrossScopeDedupeOracle,
    BlobNoSidecarPathOracle, BlobReachabilityOracle, BlockedReclaimUntilReleaseOracle,
    CounterContractOracle, CrashRecoversOldOrNewNeverMixedOracle,
    IndependentVerifierAgreementOracle, IoPressureSimulationOracle, NoJsonAuthorityOracle,
    NoMixedRootOracle, NoPrivateMutationOracle, OldReaderSeesOldRootOracle, OracleDenial,
    OracleVerdictBasis, PhysicalIsolationInterleavingOracle, PhysicalOracleJudgment,
    PhysicalOracleNonClaim, PhysicalOracleVerdictTopology, PhysicalOracleVerdictTopologyPosture,
    PhysicalProofOracle, PhysicalProofOracleKind, PhysicalProofOracleVerdict,
    PhysicalProofOracleVerdictKind, PostSwapReaderSeesNewRootOracle, ReusablePhysicalOracleFamily,
    TranscriptReplayOracle,
};
pub use physical_isolation_entry::{
    admit_physical_isolation_entry, admit_physical_isolation_entry_checked,
    reject_copied_recovery_fields_as_physical_isolation_entry,
    reject_foundational_or_proof_projection_as_physical_isolation_entry,
    reject_json_authority_as_physical_isolation_entry,
    reject_live_runtime_state_as_physical_isolation_entry,
    reject_semantic_snapshot_as_physical_isolation_entry,
    reject_stale_recovery_readiness_as_physical_isolation_entry,
    reject_terminal_projection_as_physical_isolation_entry,
    require_rebound_recovery_readiness_for_physical_isolation_entry,
    PhysicalIsolationAdmittedEntryRecipe, PhysicalIsolationEntryAdmission,
    PhysicalIsolationEntryCheckedOutcome, PhysicalIsolationEntryDenial,
    PhysicalIsolationEntryEvidence, PhysicalIsolationEntryFoundationalEvidence,
    PhysicalIsolationEntryIdentity, PhysicalIsolationEntryProofProgression,
    PhysicalIsolationEntryProofRequest, PhysicalIsolationEntryRebindRequired,
    PhysicalIsolationEntryRequest, PhysicalIsolationLoweredEntryRecipe,
    PhysicalIsolationResolvedEntryRecipe, PhysicalIsolationRootEpochBasis, RecoveryReadinessBasis,
};
pub use planning::{
    lower_physical_simulation_plan, reject_unresolved_simulation_plan_recipe,
    require_lowered_physical_simulation_plan, FixtureClassKind, ForbiddenShortcutKind,
    ForbiddenShortcutSet, ObserverKind, OracleFamilyKind, PhysicalDriverKind,
    PhysicalSimulationCapability, PhysicalSimulationCapabilitySet, PhysicalSimulationPlan,
    PhysicalSimulationPlanIdentity, PhysicalSimulationProfile, PhysicalSimulationProfileSet,
    RequiredActorSet, RequiredFixtureClassSet, RequiredObserverSet, RequiredOracleFamilySet,
    RequiredPhysicalDriverSet, SimulationEvidencePolicy, SimulationPlanDenial,
    SimulationPlanningContext, SupportedObserverSet, SupportedOracleFamilySet,
    SupportedPhysicalDriverSet,
};
#[cfg(feature = "certification-test-support")]
pub use pressure_harness::test_replay_bundle_for as io_pressure_test_replay_bundle_for;
pub use pressure_harness::{
    all_io_pressure_fault_evidence_classes, all_io_pressure_fault_kinds,
    IoPressureBackendSafetyQualificationDenial, IoPressureEvidenceMaturity,
    IoPressureExecutionCounters, IoPressureFaultKind, IoPressureHarnessEvidence,
    IoPressureHarnessEvidenceDenial, IoPressureHarnessScenario, IoPressureHarnessSecureIoPosture,
    IoPressureOracleObservation, PhysicalFaultEvidenceClass, RealBackendSafetyQualification,
};
pub use process_probe::{
    admit_current_process_probe, AdmittedProcessProbe, ProcessArtifactDisposition,
    ProcessArtifactObservation, ProcessArtifactPath, ProcessEnvironmentBindingEvidence,
    ProcessIdentityEvidence, ProcessIsolationRequirement, ProcessProbeDeclaration,
    ProcessProbeEvidenceDenial, ProcessProbeExecution, ProcessProbeIntent, ProcessRole,
    ProcessTermination, ProcessTerminationRequirement, SealedProcessProbeInput,
    PROCESS_PROBE_EVIDENCE_ROOT_ENV,
};
pub use qualification::{
    evaluate_row_rebind, reject_copied_backend_qualification_row,
    reject_environment_name_backend_qualification, reject_log_output_backend_qualification,
    reject_test_only_backend_label_qualification, require_profile_local_row,
    BackendQualificationMatrix, BackendQualificationMatrixDenial,
    BackendQualificationParityComparison, BackendQualificationRow, BackendQualificationRowIdentity,
    CertifiedBackendQualificationSupport, PublishedQualificationPosture,
    QualificationCapabilityProofAuthority, QualificationHarnessProof,
    QualificationHarnessProofClaim, QualificationHarnessProofStrength,
    QualificationMatrixPublisher, QualificationPublicationShortcut, QualificationRebindEvaluation,
    QualificationResidualDebt, QualificationResidualDebtReason,
};
pub use scenario::{
    reject_raw_json_scenario_authority_attempt, BlobHarnessScenarioMetadata,
    CertifiedPhysicalScenario, FreshRuntimeCrashRecoveryEvidence,
    FreshRuntimeCrashRecoveryEvidenceDenial, JsonScenarioAuthorityDenied, PhysicalScenarioActor,
    PhysicalScenarioActorRole, PhysicalScenarioActorSet, PhysicalScenarioAuthorityWitness,
    PhysicalScenarioCanonicalIdentity, PhysicalScenarioDefinitionDenial,
    PhysicalScenarioExpectation, PhysicalScenarioExpectationKind, PhysicalScenarioFault,
    PhysicalScenarioFaultKind, PhysicalScenarioFixtureSet, PhysicalScenarioIntent,
    PhysicalScenarioNonClaim, PhysicalScenarioSchedule, PhysicalSimulationScenarioDefinition,
    PhysicalSimulationScenarioFamily, RecoveryCrashSeam, TerminalProjectionScenarioDenied,
};
pub use schedule::{
    execute_physical_schedule, explore_physical_interleavings, AdmittedScheduleOrderingAuthority,
    C7DurabilityCrashSeam, CounterMismatchSummary, OracleVerdictKind, OracleVerdictSummary,
    PartialOrderReductionPosture, PhysicalActorId, PhysicalActorStep, PhysicalActorStepSequence,
    PhysicalActorStorageExecution, PhysicalFaultLocus, PhysicalInterleavingSchedule,
    PhysicalScheduleExecution, PhysicalScheduleExecutionError, PhysicalScheduleExploration,
    PhysicalScheduleOwnerExecution, ScheduleExplorationCompletion, ScheduleExplorationCost,
    ScheduleFailureClass, ScheduleFailureSignature, ScheduleOrderingAuthorityAttempt,
    ScheduleOrderingAuthorityKind, SchedulePerturbationDecision, SchedulePerturbationSeed,
    SchedulePerturbationTrace, ScheduleReplayDenial, ScheduleReplayIdentity, ScheduleShrinkTrace,
    StateSpaceBudget,
};
pub use transcript::{
    reject_copied_transcript_fields, reject_loose_log_transcript_attempt,
    reject_same_run_self_comparison_transcript_attempt, reject_terminal_json_transcript_attempt,
    DetachedSimulationReplayParts, ExecutedTranscriptParts, PhysicalSimulationTranscript,
    PhysicalSimulationTranscriptIdentity, PhysicalStoryTranscript, SimulationReplayBundle,
    SimulationRunIdentity, TranscriptReplayDenial, TranscriptReplayEvidenceIdentity,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalCertificationLane {
    PowerLoss,
    TornWrite,
    ByteFlip,
    BoundedMemory,
    RecoveryTime,
    ForegroundLatency,
    BlobScale,
    PhysicalIsolation,
}
mod certification_child_process;
