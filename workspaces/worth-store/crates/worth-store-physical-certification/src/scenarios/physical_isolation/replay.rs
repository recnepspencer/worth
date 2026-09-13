use crate::{
    admit_physical_counter_evidence, ExecutedTranscriptParts, ObservedPhysicalTrace,
    PhysicalExecutedCounterEvidence, PhysicalFaultEvent, PhysicalInterleavingSchedule,
    PhysicalIsolationInterleavingOracle, PhysicalProofOracleVerdict, PhysicalSimulationPlan,
    PhysicalSimulationTranscript, ProductionBackedPhysicalFixture, ReusablePhysicalOracleFamily,
    SimulationReplayBundle,
};

pub fn assemble_physical_isolation_replay_bundle(
    plan: &PhysicalSimulationPlan,
    schedule: PhysicalInterleavingSchedule,
    fixture: &ProductionBackedPhysicalFixture,
    trace: ObservedPhysicalTrace,
    residency: worth_store::physical_runtime::PhysicalResidencyObservation,
    expected_fault: crate::PhysicalScenarioFaultKind,
) -> SimulationReplayBundle {
    let sources = crate::PhysicalCounterExecutionSources::admit_for_plan(
        plan,
        &schedule,
        &trace,
        residency,
        io_queue_evidence(plan),
    )
    .unwrap();
    let executed = PhysicalExecutedCounterEvidence::from_execution_sources(plan, sources).unwrap();
    let counter_receipt = admit_physical_counter_evidence(plan, executed).unwrap();
    let parts =
        ExecutedTranscriptParts::new(plan, schedule, fixture, trace.clone(), counter_receipt)
            .unwrap()
            .with_faults(physical_isolation_fault_events(expected_fault))
            .with_oracle_verdict(physical_isolation_verdict(plan, &trace))
            .with_transcript_replay_verdict()
            .unwrap();
    let transcript = PhysicalSimulationTranscript::from_executed_parts(parts).unwrap();
    crate::DetachedSimulationReplayParts::from_transcript(&transcript)
        .admit_replay_bundle()
        .unwrap()
}

fn physical_isolation_verdict(
    plan: &PhysicalSimulationPlan,
    trace: &ObservedPhysicalTrace,
) -> PhysicalProofOracleVerdict {
    ReusablePhysicalOracleFamily::physical_isolation_interleaving()
        .oracle(PhysicalIsolationInterleavingOracle)
        .judge(plan, trace)
        .unwrap()
}

fn physical_isolation_fault_events(
    expected_fault: crate::PhysicalScenarioFaultKind,
) -> Vec<PhysicalFaultEvent> {
    crate::physical_isolation_stable_read_plan_fault_event(expected_fault)
        .unwrap()
        .into_iter()
        .collect()
}

fn io_queue_evidence(
    plan: &PhysicalSimulationPlan,
) -> worth_store_io_scheduler::IoQueueExecutedEvidenceSource {
    let mut recorder = worth_store_io_scheduler::IoQueueExecutionRecorder::from_envelope(
        plan.resource_envelope().io_queue(),
    );
    recorder.observe_queue_depth(1).unwrap();
    recorder.executed_evidence().unwrap()
}
