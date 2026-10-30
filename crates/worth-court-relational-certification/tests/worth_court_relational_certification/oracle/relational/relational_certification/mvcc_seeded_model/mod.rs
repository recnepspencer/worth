#[allow(unused_imports)]
use crate::relational_certification::reference_context::mvcc_seeded_model::*;

#[test]
fn generated_standard_sequences_cover_all_deltas_and_probes() {
    let mut covered_deltas = std::collections::BTreeSet::new();
    let mut covered_probes = std::collections::BTreeSet::new();
    for sequence in 0..8_u64 {
        let trace = generate_trace(standard_sequence_seed(sequence));
        covered_deltas.extend(trace.scenarios.iter().map(|scenario| scenario.delta));
        covered_probes.extend(trace.scenarios.iter().map(|scenario| scenario.probe));
    }
    assert_eq!(covered_deltas, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ALL.into_iter().collect());
    assert_eq!(covered_probes.len(), 4);
}

#[test]
fn generated_standard_seed_91711200_is_replayable_and_shrinkable() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_standard_sequence_replayable_and_shrinkable(execution, 0);
}

#[test]
fn generated_standard_seed_12fa88bb9_is_replayable_and_shrinkable() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_standard_sequence_replayable_and_shrinkable(execution, 1);
}

#[test]
fn generated_standard_seed_1cde00572_is_replayable_and_shrinkable() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_standard_sequence_replayable_and_shrinkable(execution, 2);
}

#[test]
fn generated_standard_seed_26c177f2b_is_replayable_and_shrinkable() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_standard_sequence_replayable_and_shrinkable(execution, 3);
}

#[test]
fn generated_standard_seed_30a4ef8e4_is_replayable_and_shrinkable() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_standard_sequence_replayable_and_shrinkable(execution, 4);
}

#[test]
fn generated_standard_seed_3a886729d_is_replayable_and_shrinkable() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_standard_sequence_replayable_and_shrinkable(execution, 5);
}

#[test]
fn generated_standard_seed_446bdec56_is_replayable_and_shrinkable() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_standard_sequence_replayable_and_shrinkable(execution, 6);
}

#[test]
fn generated_standard_seed_4e4f5660f_is_replayable_and_shrinkable() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_standard_sequence_replayable_and_shrinkable(execution, 7);
}
