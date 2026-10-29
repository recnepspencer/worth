use {worth_store_certification::S6CertifiedQueueExecutionEvidence};
use {worth_store_io_scheduler::execute_grouped_ready_queue_plans, worth_store_io_scheduler::group_ready_queue_pair, worth_store_io_scheduler::QueueGroupingOutcome};
use {worth_store_physical_backend::BackendQueueExecutionAdaptation, worth_store_physical_backend::BackendQueueExecutionCompletion, worth_store_physical_backend::BackendQueueExecutionPosture, worth_store_physical_backend::BackendQueueSpeculativeScope};

use {crate::support::admitted_plan, crate::support::backend_witness};

#[test]
fn grouped_certification_preserves_secondary_replay_identity() {
    let grouping = group_ready_queue_pair(admitted_plan(), admitted_plan());
    let worth_store_io_scheduler::QueueGroupingOutcome::Grouped(grouped) = grouping else {
        panic!("equivalent ready plans should group");
    };
    let expected_secondary = grouped.replay_identities()[1].clone();
    let scope = worth_store_physical_backend::BackendQueueSpeculativeScope::admitted(
        grouped.first().grouping_basis().security_scope_identity(),
        grouped.first().grouping_basis().tenant_scope(),
        grouped.first().grouping_basis().key_scope(),
    );
    let witness = backend_witness();
    let posture = worth_store_physical_backend::BackendQueueExecutionPosture::from_admitted_capability(
        &witness,
        worth_store_physical_backend::BackendQueueExecutionAdaptation::None,
    )
    .expect("certification backend posture should admit");
    let completion = worth_store_physical_backend::BackendQueueExecutionCompletion::for_certification(
        grouped
            .backend_completion_binding()
            .backend_execution_binding(),
        posture,
    )
    .observe_queue_depth(1)
    .observe_read_ahead(1, scope);
    let outcome = execute_grouped_ready_queue_plans(*grouped, completion);

    let certified = worth_store_certification::S6CertifiedQueueExecutionEvidence::from_outcome(&outcome)
        .expect("executed grouped outcome should certify");

    assert_eq!(
        certified.secondary_replay_identity(),
        Some(expected_secondary)
    );
    assert_eq!(certified.counters().grouped_writes(), 2);
}
