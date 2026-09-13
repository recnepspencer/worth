use crate::{
    CheckpointInterlockObservation, CompactionInterlockObservation, IndependentVerifierObservation,
    ObservationDenial, ObservedPhysicalTrace, PhysicalInterleavingSchedule,
    PhysicalSimulationBoundaryObservation, PhysicalSimulationObserver, PhysicalSimulationPlan,
    PhysicalSimulationScenarioFamily, ShortcutRejectionObservation,
};

pub struct PhysicalIsolationTraceFixtures {
    checkpoint_interlock: Option<CheckpointInterlockObservation>,
    compaction_interlock: Option<CompactionInterlockObservation>,
    independent_verifier: Option<IndependentVerifierObservation>,
}

impl PhysicalIsolationTraceFixtures {
    pub const fn complete(
        checkpoint_interlock: CheckpointInterlockObservation,
        independent_verifier: IndependentVerifierObservation,
    ) -> Self {
        Self {
            checkpoint_interlock: Some(checkpoint_interlock),
            compaction_interlock: None,
            independent_verifier: Some(independent_verifier),
        }
    }

    pub const fn new(
        checkpoint_interlock: CheckpointInterlockObservation,
        independent_verifier: IndependentVerifierObservation,
    ) -> Self {
        Self::complete(checkpoint_interlock, independent_verifier)
    }

    pub fn without_checkpoint_interlock(mut self) -> Self {
        self.checkpoint_interlock = None;
        self
    }

    pub fn with_compaction_interlock_observation(
        mut self,
        observation: CompactionInterlockObservation,
    ) -> Self {
        self.compaction_interlock = Some(observation);
        self
    }

    pub fn without_independent_verifier(mut self) -> Self {
        self.independent_verifier = None;
        self
    }
}

pub fn observe_physical_isolation_trace(
    plan: &PhysicalSimulationPlan,
    schedule: &PhysicalInterleavingSchedule,
    fixtures: PhysicalIsolationTraceFixtures,
) -> Result<ObservedPhysicalTrace, ObservationDenial> {
    if !schedule.replay_identity_matches_plan(plan) {
        return Err(ObservationDenial::ScheduleExecutionMismatch);
    }
    let execution =
        PhysicalSimulationBoundaryObservation::from_declared_driver_shape_probe(plan).unwrap();
    let builder = PhysicalSimulationObserver::independent_physical_trace()
        .observe_boundary_observation(plan, &execution)
        .unwrap()
        .with_shortcut_rejection_observation(
            ShortcutRejectionObservation::private_mutation_denied(),
        );
    let builder = if let Some(observation) = fixtures.compaction_interlock {
        builder.with_compaction_interlock_observation(observation)
    } else {
        builder
    };
    let trace = match plan.scenario_family() {
        PhysicalSimulationScenarioFamily::PhysicalIsolationCheckpointPublicationInterlock
        | PhysicalSimulationScenarioFamily::PhysicalIsolationRestartDuringCutover => {
            let builder = if let Some(observation) = fixtures.checkpoint_interlock {
                builder.with_checkpoint_interlock_observation(observation)
            } else {
                builder
            };
            builder.complete().unwrap()
        }
        PhysicalSimulationScenarioFamily::PhysicalIsolationTierMovementStability
        | PhysicalSimulationScenarioFamily::PhysicalIsolationFutureChunkStability => {
            let builder = if let Some(observation) = fixtures.independent_verifier {
                builder.with_independent_verifier_observation(observation)
            } else {
                builder
            };
            builder.complete().unwrap()
        }
        _ => builder.complete().unwrap(),
    };
    Ok(trace)
}
