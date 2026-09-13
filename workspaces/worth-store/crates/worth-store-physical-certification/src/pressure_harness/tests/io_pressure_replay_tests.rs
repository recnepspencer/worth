use crate::pressure_harness::fixtures::{
    replay_bundle_for, replay_bundle_with_sample, IoPressureExecutionSample,
};
use crate::{
    CounterContractKind, CounterExpectationKind, IoPressureHarnessEvidence,
    IoPressureHarnessScenario, PhysicalCounterEvidenceRow, PhysicalSimulationProfile,
};

#[test]
fn same_scenario_replay_preserves_all_io_pressure_topology() {
    let scenario = IoPressureHarnessScenario::deterministic_read_under_repair_pressure();

    let first = replay_bundle_for(scenario.clone(), PhysicalSimulationProfile::DeveloperSmoke);
    let second = replay_bundle_for(scenario.clone(), PhysicalSimulationProfile::DeveloperSmoke);
    let first_evidence =
        IoPressureHarnessEvidence::from_replay_bundle(scenario.clone(), &first).unwrap();
    let second_evidence = IoPressureHarnessEvidence::from_replay_bundle(scenario, &second).unwrap();

    assert_eq!(first.plan().identity(), second.plan().identity());
    assert_eq!(first.schedule(), second.schedule());
    assert_eq!(first.transcript_identity(), second.transcript_identity());
    assert_eq!(
        first.replay_basis_identity(),
        second.replay_basis_identity()
    );
    assert_eq!(first.oracle_verdicts(), second.oracle_verdicts());
    assert_eq!(
        first.counter_receipt().rows(),
        second.counter_receipt().rows()
    );
    assert_eq!(first.fault_events(), second.fault_events());
    assert_eq!(
        first_evidence.replay_identity(),
        second_evidence.replay_identity()
    );
}

#[test]
fn same_scenario_replay_counter_rows_are_execution_owned_and_varied() {
    let scenario = IoPressureHarnessScenario::deterministic_read_under_repair_pressure();
    let developer = IoPressureExecutionSample::developer_smoke();
    let ci = IoPressureExecutionSample::ci_certification();
    let developer_replay = replay_bundle_with_sample(
        scenario.clone(),
        PhysicalSimulationProfile::DeveloperSmoke,
        developer,
    );
    let ci_replay =
        replay_bundle_with_sample(scenario, PhysicalSimulationProfile::CiCertification, ci);

    assert_pressure_counter_rows(developer_replay.counter_receipt().rows(), developer);
    assert_pressure_counter_rows(ci_replay.counter_receipt().rows(), ci);
    assert_ne!(
        developer_replay.counter_receipt().rows(),
        ci_replay.counter_receipt().rows()
    );
}

fn assert_pressure_counter_rows(
    rows: &[PhysicalCounterEvidenceRow],
    sample: IoPressureExecutionSample,
) {
    assert_counter_row(
        rows,
        CounterContractKind::IoQueueDepth,
        CounterExpectationKind::Bounded,
        u64::from(sample.queue_depth),
    );
    assert_counter_row(
        rows,
        CounterContractKind::IoInterferenceEvents,
        CounterExpectationKind::Positive,
        u64::from(sample.interference_events),
    );
    assert_counter_row(
        rows,
        CounterContractKind::AllocationBytes,
        CounterExpectationKind::Bounded,
        sample.allocation_bytes,
    );
}

fn assert_counter_row(
    rows: &[PhysicalCounterEvidenceRow],
    kind: CounterContractKind,
    strength: CounterExpectationKind,
    observed_count: u64,
) {
    assert!(rows.iter().any(|row| {
        row.kind() == kind && row.strength() == strength && row.observed_count() == observed_count
    }));
}
