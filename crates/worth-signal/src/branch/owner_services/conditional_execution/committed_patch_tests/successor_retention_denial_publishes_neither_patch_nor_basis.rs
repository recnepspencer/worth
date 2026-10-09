//! Successor retention denial publishes neither patch nor basis.

use super::*;

#[test]
fn successor_retention_denial_publishes_neither_patch_nor_basis() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let (mut measured_graph, _, _, _) = installed_graph();
    let prepared = SignalExecutionBasis::prepare_capture(
        &mut measured_graph,
        &mut RetainedStoragePreparation::new(100_000),
    )
    .unwrap();
    let charges = prepared.charges();
    let reservation_handles =
        2 * std::mem::size_of::<SignalConditionalRetentionReservation>() as u64;
    let exact_initial_bytes =
        charges.retained.bytes() + charges.source_growth.bytes() + reservation_handles;

    let (graph, claimant, [contract, _], source_owner) = installed_graph();
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    runtime.set_runtime_policy(
        SignalRuntimePolicy::development().with_conditional_evaluation_budget(
            SignalConditionalEvaluationBudget {
                maximum_retained_slots: 1,
                maximum_retained_bytes: exact_initial_bytes,
                maximum_attempt_visits: 100_000,
            },
        ),
    );
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let _ports = runtime.owner_port_slots().unwrap();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    let graph_instance_id = contract.graph_instance_id();
    let before = current_basis_and_graph_observation(&service, &contract);
    let owner = service.owner.upgrade().unwrap();
    let retention_before = owner.conditional_retention.usage();
    drop(owner);

    assert!(matches!(
        service.deliver_committed_patch(
            request_execution,
            &contract,
            DeliveryRequest::new([target(graph_instance_id, &contract)]),
        ),
        Err(DeliveryDenial::SuccessorCaptureCapacityExhausted)
    ));
    assert_eq!(
        current_basis_and_graph_observation(&service, &contract),
        before
    );
    assert_eq!(
        service
            .owner
            .upgrade()
            .unwrap()
            .conditional_retention
            .usage(),
        retention_before
    );
}
