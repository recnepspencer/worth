use crate::{
    ObservationDenial, ObservedPhysicalTrace, ObserverKind, PhysicalInterleavingSchedule,
    PhysicalScenarioActorRole, PhysicalSimulationPlan, RecoveryOutcomeKind,
    RecoveryOutcomeObservation,
};
use worth_foundational::CanonicalDigestId;
use worth_store_physical_isolation::{
    CheckpointInterlockEvidenceOrigin, CheckpointInterlockFoundationalEvidence,
};

use super::checkpoint_publication_lane::{
    require_evidence_origin_matches, require_role_yieldpoint_step_matches_schedule,
    PhysicalIsolationCheckpointPublicationLaneBinding,
};

const FRESH_RUNTIME_RECOVERY_YIELDPOINT: &str = "fresh-runtime-replay-open";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalIsolationCheckpointPublicationRecoveryOutcomeLaneOutput {
    pub(super) checkpoint_plan_identity: CanonicalDigestId,
    pub(super) checkpoint_schedule_identity: CanonicalDigestId,
    pub(super) checkpoint_origin: CheckpointInterlockEvidenceOrigin,
    pub(super) recovery_plan_identity: CanonicalDigestId,
    pub(super) recovery_schedule_identity: CanonicalDigestId,
    pub(super) recovery_actor_step_index: usize,
    observation: RecoveryOutcomeObservation,
}

#[derive(Debug, Clone, Copy)]
pub struct CheckpointPublicationRecoveryExecution<'a> {
    recovery_plan: &'a PhysicalSimulationPlan,
    recovery_schedule: &'a PhysicalInterleavingSchedule,
    recovery_actor_step_index: usize,
    recovery_trace: &'a ObservedPhysicalTrace,
}

impl<'a> CheckpointPublicationRecoveryExecution<'a> {
    pub const fn new(
        recovery_plan: &'a PhysicalSimulationPlan,
        recovery_schedule: &'a PhysicalInterleavingSchedule,
        recovery_actor_step_index: usize,
        recovery_trace: &'a ObservedPhysicalTrace,
    ) -> Self {
        Self {
            recovery_plan,
            recovery_schedule,
            recovery_actor_step_index,
            recovery_trace,
        }
    }
}

impl PhysicalIsolationCheckpointPublicationRecoveryOutcomeLaneOutput {
    pub fn from_fresh_runtime_recovery_trace(
        binding: &PhysicalIsolationCheckpointPublicationLaneBinding,
        checkpoint_schedule: &PhysicalInterleavingSchedule,
        recovery: CheckpointPublicationRecoveryExecution<'_>,
        expected_origin: &CheckpointInterlockEvidenceOrigin,
        evidence: CheckpointInterlockFoundationalEvidence,
    ) -> Result<Self, ObservationDenial> {
        if checkpoint_schedule.identity().digest_bytes() != binding.schedule_identity.bytes() {
            return Err(ObservationDenial::CheckpointPublicationLaneScheduleMismatch);
        }
        require_evidence_origin_matches(expected_origin, evidence.origin())?;
        if !recovery
            .recovery_schedule
            .replay_identity_matches_plan(recovery.recovery_plan)
            || recovery.recovery_trace.plan_identity().digest_bytes()
                != recovery.recovery_plan.identity().digest_bytes()
            || recovery.recovery_trace.observer() != ObserverKind::RecoveryOutcomeObserver
        {
            return Err(ObservationDenial::CheckpointPublicationCrashRecoveryTraceMismatch);
        }
        require_role_yieldpoint_step_matches_schedule(
            recovery.recovery_schedule,
            recovery.recovery_actor_step_index,
            PhysicalScenarioActorRole::RecoveryDriver,
            FRESH_RUNTIME_RECOVERY_YIELDPOINT,
            ObservationDenial::CheckpointPublicationCrashLaneScheduleMismatch,
        )?;
        let observation = *recovery
            .recovery_trace
            .recovery_outcome()
            .ok_or(ObservationDenial::MissingRecoveryOutcomeObservation)?;
        if observation.kind() == RecoveryOutcomeKind::MixedRoot {
            return Err(ObservationDenial::CheckpointPublicationCrashOutcomeMixedRoot);
        }
        Ok(Self {
            checkpoint_plan_identity: binding.plan_identity,
            checkpoint_schedule_identity: binding.schedule_identity,
            checkpoint_origin: expected_origin.clone(),
            recovery_plan_identity: CanonicalDigestId::new(
                *recovery.recovery_plan.identity().digest_bytes(),
            ),
            recovery_schedule_identity: CanonicalDigestId::new(
                *recovery.recovery_schedule.identity().digest_bytes(),
            ),
            recovery_actor_step_index: recovery.recovery_actor_step_index,
            observation,
        })
    }

    pub const fn observation(&self) -> RecoveryOutcomeObservation {
        self.observation
    }
}
