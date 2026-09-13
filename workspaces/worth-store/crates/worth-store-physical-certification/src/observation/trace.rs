use super::{
    CheckpointCrashReplayObservation, CheckpointInterlockObservation,
    CompactionInterlockObservation,
};
use crate::{
    BlobHarnessOracleObservation, IndependentVerifierObservation, IoPressureOracleObservation,
    ObserverKind, PhysicalScenarioCanonicalIdentity, PhysicalSimulationObservationBasis,
    PhysicalSimulationPlan, PhysicalSimulationPlanIdentity, ProductionBoundaryDriverTrace,
    RecoveryOutcomeObservation, ShortcutRejectionObservation,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedPhysicalTrace {
    observer: ObserverKind,
    scenario_identity: PhysicalScenarioCanonicalIdentity,
    plan_identity: PhysicalSimulationPlanIdentity,
    observation_basis: PhysicalSimulationObservationBasis,
    runtime_trace: ProductionBoundaryDriverTrace,
    independent_verifier: Option<IndependentVerifierObservation>,
    recovery_outcome: Option<RecoveryOutcomeObservation>,
    checkpoint_crash_replay: Option<CheckpointCrashReplayObservation>,
    checkpoint_interlock: Option<CheckpointInterlockObservation>,
    compaction_interlock: Option<CompactionInterlockObservation>,
    io_pressure: Option<IoPressureOracleObservation>,
    blob_harness: Option<BlobHarnessOracleObservation>,
    shortcut_rejections: Vec<ShortcutRejectionObservation>,
}

pub(crate) struct ObservedPhysicalEvidence {
    pub(crate) independent_verifier: Option<IndependentVerifierObservation>,
    pub(crate) recovery_outcome: Option<RecoveryOutcomeObservation>,
    pub(crate) checkpoint_crash_replay: Option<CheckpointCrashReplayObservation>,
    pub(crate) checkpoint_interlock: Option<CheckpointInterlockObservation>,
    pub(crate) compaction_interlock: Option<CompactionInterlockObservation>,
    pub(crate) io_pressure: Option<IoPressureOracleObservation>,
    pub(crate) blob_harness: Option<BlobHarnessOracleObservation>,
    pub(crate) shortcut_rejections: Vec<ShortcutRejectionObservation>,
}

impl ObservedPhysicalTrace {
    pub(crate) fn from_parts(
        observer: ObserverKind,
        plan: &PhysicalSimulationPlan,
        observation_basis: PhysicalSimulationObservationBasis,
        runtime_trace: ProductionBoundaryDriverTrace,
        evidence: ObservedPhysicalEvidence,
    ) -> Self {
        Self {
            observer,
            scenario_identity: plan.scenario_identity().clone(),
            plan_identity: plan.identity().clone(),
            observation_basis,
            runtime_trace,
            independent_verifier: evidence.independent_verifier,
            recovery_outcome: evidence.recovery_outcome,
            checkpoint_crash_replay: evidence.checkpoint_crash_replay,
            checkpoint_interlock: evidence.checkpoint_interlock,
            compaction_interlock: evidence.compaction_interlock,
            io_pressure: evidence.io_pressure,
            blob_harness: evidence.blob_harness,
            shortcut_rejections: evidence.shortcut_rejections,
        }
    }

    pub const fn observer(&self) -> ObserverKind {
        self.observer
    }

    pub const fn scenario_identity(&self) -> &PhysicalScenarioCanonicalIdentity {
        &self.scenario_identity
    }

    pub const fn plan_identity(&self) -> &PhysicalSimulationPlanIdentity {
        &self.plan_identity
    }

    pub const fn runtime_trace(&self) -> &ProductionBoundaryDriverTrace {
        &self.runtime_trace
    }

    pub const fn observation_basis(&self) -> PhysicalSimulationObservationBasis {
        self.observation_basis
    }

    pub const fn independent_verifier(&self) -> Option<&IndependentVerifierObservation> {
        self.independent_verifier.as_ref()
    }

    pub const fn recovery_outcome(&self) -> Option<&RecoveryOutcomeObservation> {
        self.recovery_outcome.as_ref()
    }

    pub const fn checkpoint_crash_replay(&self) -> Option<&CheckpointCrashReplayObservation> {
        self.checkpoint_crash_replay.as_ref()
    }

    pub const fn checkpoint_interlock(&self) -> Option<CheckpointInterlockObservation> {
        self.checkpoint_interlock
    }

    pub const fn compaction_interlock(&self) -> Option<CompactionInterlockObservation> {
        self.compaction_interlock
    }

    pub const fn io_pressure_observation(&self) -> Option<IoPressureOracleObservation> {
        self.io_pressure
    }

    pub const fn blob_harness_observation(&self) -> Option<BlobHarnessOracleObservation> {
        self.blob_harness
    }

    pub fn shortcut_rejections(&self) -> &[ShortcutRejectionObservation] {
        &self.shortcut_rejections
    }
}
