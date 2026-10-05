use crate::facade::transactions::CommitTraceEvent;
use crate::tests::support::*;

#[test]
fn staged_parallel_commit_records_preparation_strategy_and_packet_counters() {
    let runtime = runtime_with_test_schema();
    let result = create_entity_outcome_with_lease(&runtime, "staged", &test_execution_lease());

    assert!(result.complexity_delta().preparation_packet_count >= 1);
    assert!(result.complexity_delta().preparation_parallel_legal_count >= 1);
    assert!(
        result
            .complexity_delta()
            .preparation_parallel_profitable_count
            >= 1
    );
    assert!(
        result
            .complexity_delta()
            .preparation_staged_parallel_strategy_count
            >= 1
    );
    assert_eq!(
        result.complexity_delta().preparation_reducer_conflict_count,
        0
    );

    let staged_execution = result
        .invariant_executions()
        .iter()
        .find(|execution| {
            execution
                .metadata()
                .preparation_strategy()
                .is_some_and(|strategy| {
                    strategy.selected_mode
                        == crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection::StagedParallel
                })
        })
        .expect("staged preparation execution");
    assert_eq!(
        staged_execution
            .metadata()
            .preparation_strategy()
            .map(|strategy| strategy.selected_mode),
        Some(
            crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection::StagedParallel
        )
    );

    assert!(result.commit_log().events().iter().any(|event| matches!(
        event,
        CommitTraceEvent::InvariantEvaluated {
            preparation_selected_mode: Some(
                crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection::StagedParallel
            ),
            ..
        }
    )));
}

#[test]
fn staged_parallel_patch_preparation_matches_serial_patch_surface() {
    let serial_runtime = runtime_with_test_schema();
    let staged_runtime = runtime_with_test_schema();

    let serial = create_entity_outcome(&serial_runtime, "patch-parity");
    let staged =
        create_entity_outcome_with_lease(&staged_runtime, "patch-parity", &test_execution_lease());

    assert_eq!(serial.patch(), staged.patch());
    assert_eq!(serial.envelope().patch, staged.envelope().patch);
    assert!(staged.complexity_delta().preparation_packet_count >= serial.patch().len());
}
