use crate::{
    CheckpointInterlockObservation, ObservationDenial, PhysicalInterleavingSchedule,
    PhysicalScenarioActorRole, PhysicalSimulationPlan, RecoveryOutcomeObservation,
};
use worth_store_physical_isolation::{
    CheckpointInterlockEvidenceOrigin, CheckpointInterlockFoundationalEvidence,
};

use super::checkpoint_recovery_lane::PhysicalIsolationCheckpointPublicationRecoveryOutcomeLaneOutput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalIsolationCheckpointPublicationLaneBinding {
    pub(super) plan_identity: [u8; 32],
    pub(super) schedule_identity: [u8; 32],
    checkpoint_actor_step_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalIsolationCheckpointPublicationScheduledLaneOutput {
    plan_identity: [u8; 32],
    schedule_identity: [u8; 32],
    checkpoint_actor_step_index: usize,
    observation: CheckpointInterlockObservation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointCrashReplayObservation {
    checkpoint_origin: CheckpointInterlockEvidenceOrigin,
    recovery_outcome: RecoveryOutcomeObservation,
    checkpoint_actor_step_index: usize,
    recovery_actor_step_index: usize,
    recovery_plan_identity: [u8; 32],
    recovery_schedule_identity: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalIsolationCheckpointPublicationCrashLaneOutput {
    plan_identity: [u8; 32],
    schedule_identity: [u8; 32],
    checkpoint_actor_step_index: usize,
    recovery_actor_step_index: usize,
    observation: CheckpointCrashReplayObservation,
}

impl PhysicalIsolationCheckpointPublicationLaneBinding {
    pub fn from_plan_and_schedule(
        plan: &PhysicalSimulationPlan,
        schedule: &PhysicalInterleavingSchedule,
    ) -> Result<Self, ObservationDenial> {
        let checkpoint_actor_step_index = require_schedule_matches_checkpoint_plan(plan, schedule)?;
        Ok(Self {
            plan_identity: *plan.identity().digest_bytes(),
            schedule_identity: *schedule.identity().digest_bytes(),
            checkpoint_actor_step_index,
        })
    }

    pub const fn checkpoint_actor_step_index(&self) -> usize {
        self.checkpoint_actor_step_index
    }
}

impl CheckpointCrashReplayObservation {
    pub const fn checkpoint_origin(&self) -> &CheckpointInterlockEvidenceOrigin {
        &self.checkpoint_origin
    }

    pub const fn recovery_outcome(&self) -> RecoveryOutcomeObservation {
        self.recovery_outcome
    }

    pub const fn checkpoint_actor_step_index(&self) -> usize {
        self.checkpoint_actor_step_index
    }

    pub const fn recovery_actor_step_index(&self) -> usize {
        self.recovery_actor_step_index
    }

    pub const fn recovery_plan_identity(&self) -> &[u8; 32] {
        &self.recovery_plan_identity
    }

    pub const fn recovery_schedule_identity(&self) -> &[u8; 32] {
        &self.recovery_schedule_identity
    }
}

impl PhysicalIsolationCheckpointPublicationScheduledLaneOutput {
    pub fn from_schedule_step_evidence(
        binding: &PhysicalIsolationCheckpointPublicationLaneBinding,
        schedule: &PhysicalInterleavingSchedule,
        actor_step_index: usize,
        expected_origin: &CheckpointInterlockEvidenceOrigin,
        evidence: CheckpointInterlockFoundationalEvidence,
    ) -> Result<Self, ObservationDenial> {
        require_schedule_step_matches_binding(binding, schedule, actor_step_index)?;
        require_evidence_origin_matches(expected_origin, evidence.origin())?;
        let observation = CheckpointInterlockObservation::from_store_interlock_evidence(evidence)
            .ok_or(ObservationDenial::MissingCheckpointPublicationLane)?;
        Ok(Self {
            plan_identity: binding.plan_identity,
            schedule_identity: binding.schedule_identity,
            checkpoint_actor_step_index: actor_step_index,
            observation,
        })
    }

    pub fn reject_copied_checkpoint_report_attempt(
        binding: &PhysicalIsolationCheckpointPublicationLaneBinding,
        schedule: &PhysicalInterleavingSchedule,
        actor_step_index: usize,
        expected_origin: &CheckpointInterlockEvidenceOrigin,
        copied_evidence: CheckpointInterlockFoundationalEvidence,
    ) -> Result<Self, ObservationDenial> {
        require_schedule_step_matches_binding(binding, schedule, actor_step_index)?;
        if expected_origin != copied_evidence.origin() {
            return Err(ObservationDenial::CheckpointPublicationEvidenceOriginMismatch);
        }
        Err(ObservationDenial::CopiedCheckpointReportObservationDenied)
    }

    pub const fn plan_identity(&self) -> &[u8; 32] {
        &self.plan_identity
    }

    pub const fn schedule_identity(&self) -> &[u8; 32] {
        &self.schedule_identity
    }

    pub const fn checkpoint_actor_step_index(&self) -> usize {
        self.checkpoint_actor_step_index
    }

    pub const fn observation(&self) -> CheckpointInterlockObservation {
        self.observation
    }
}

impl PhysicalIsolationCheckpointPublicationCrashLaneOutput {
    pub fn from_schedule_step_recovery(
        binding: &PhysicalIsolationCheckpointPublicationLaneBinding,
        schedule: &PhysicalInterleavingSchedule,
        checkpoint_actor_step_index: usize,
        expected_origin: &CheckpointInterlockEvidenceOrigin,
        evidence: CheckpointInterlockFoundationalEvidence,
        recovery_output: &PhysicalIsolationCheckpointPublicationRecoveryOutcomeLaneOutput,
    ) -> Result<Self, ObservationDenial> {
        require_schedule_step_matches_binding(binding, schedule, checkpoint_actor_step_index)?;
        require_evidence_origin_matches(expected_origin, evidence.origin())?;
        if recovery_output.checkpoint_plan_identity != binding.plan_identity
            || recovery_output.checkpoint_schedule_identity != binding.schedule_identity
            || &recovery_output.checkpoint_origin != expected_origin
        {
            return Err(ObservationDenial::CheckpointPublicationCrashRecoveryTraceMismatch);
        }
        let observation = CheckpointCrashReplayObservation {
            checkpoint_origin: recovery_output.checkpoint_origin.clone(),
            recovery_outcome: recovery_output.observation(),
            checkpoint_actor_step_index,
            recovery_actor_step_index: recovery_output.recovery_actor_step_index,
            recovery_plan_identity: recovery_output.recovery_plan_identity,
            recovery_schedule_identity: recovery_output.recovery_schedule_identity,
        };
        Ok(Self {
            plan_identity: binding.plan_identity,
            schedule_identity: binding.schedule_identity,
            checkpoint_actor_step_index,
            recovery_actor_step_index: recovery_output.recovery_actor_step_index,
            observation,
        })
    }

    pub const fn plan_identity(&self) -> &[u8; 32] {
        &self.plan_identity
    }

    pub const fn schedule_identity(&self) -> &[u8; 32] {
        &self.schedule_identity
    }

    pub const fn checkpoint_actor_step_index(&self) -> usize {
        self.checkpoint_actor_step_index
    }

    pub const fn recovery_actor_step_index(&self) -> usize {
        self.recovery_actor_step_index
    }

    pub const fn recovery_outcome(&self) -> RecoveryOutcomeObservation {
        self.observation.recovery_outcome()
    }

    pub fn observation(&self) -> CheckpointCrashReplayObservation {
        self.observation.clone()
    }
}

pub(super) fn require_evidence_origin_matches(
    expected_origin: &CheckpointInterlockEvidenceOrigin,
    observed_origin: &CheckpointInterlockEvidenceOrigin,
) -> Result<(), ObservationDenial> {
    if expected_origin == observed_origin {
        Ok(())
    } else {
        Err(ObservationDenial::CheckpointPublicationEvidenceOriginMismatch)
    }
}

pub(super) fn require_role_yieldpoint_step_matches_schedule(
    schedule: &PhysicalInterleavingSchedule,
    actor_step_index: usize,
    role: PhysicalScenarioActorRole,
    yieldpoint: &str,
    denial: ObservationDenial,
) -> Result<(), ObservationDenial> {
    schedule
        .actor_steps()
        .get(actor_step_index)
        .filter(|step| step.actor_role() == role && step.yieldpoint() == yieldpoint)
        .map(|_| ())
        .ok_or(denial)
}

fn require_schedule_matches_checkpoint_plan(
    plan: &PhysicalSimulationPlan,
    schedule: &PhysicalInterleavingSchedule,
) -> Result<usize, ObservationDenial> {
    if !schedule.replay_identity_matches_plan(plan) {
        return Err(ObservationDenial::CheckpointPublicationLaneScheduleMismatch);
    }
    let declared_yieldpoint = plan.yieldpoint_binding().declared_yieldpoint().name();
    schedule
        .actor_steps()
        .iter()
        .position(|step| {
            step.actor_role() == PhysicalScenarioActorRole::CheckpointDriver
                && step.yieldpoint() == declared_yieldpoint
        })
        .ok_or(ObservationDenial::MissingCheckpointPublicationLane)
}

fn require_schedule_step_matches_binding(
    binding: &PhysicalIsolationCheckpointPublicationLaneBinding,
    schedule: &PhysicalInterleavingSchedule,
    actor_step_index: usize,
) -> Result<(), ObservationDenial> {
    if schedule.identity().digest_bytes() != &binding.schedule_identity
        || actor_step_index != binding.checkpoint_actor_step_index
    {
        return Err(ObservationDenial::CheckpointPublicationLaneScheduleMismatch);
    }
    schedule
        .actor_steps()
        .get(actor_step_index)
        .filter(|step| step.actor_role() == PhysicalScenarioActorRole::CheckpointDriver)
        .map(|_| ())
        .ok_or(ObservationDenial::MissingCheckpointPublicationLane)
}
