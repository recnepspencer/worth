//! Equivalent receiving-owner work for nearby and distant retained payloads.
use crate::relational::relational_certification::owner_service_completion::retained_entity_selection::{create, update, values, fixture, perform};
use worth_execution::ExecutionRequest;
use worth_testkit_relational::aspect_storage::publications::{changed_entities, release_test_commit_snapshot, string_aspect_value};
use worth_relational::facade::{
    branch::{AdmittedRelationalBranchBasis, RelationalBranchIdentity},
    history::BranchId,
    identity::EntityId, runtime::RelationalRuntime,
    mvcc::RelationalTransactionIntent, transactions::CommitResult,
};

#[test]
fn retained_entity_selections_preserve_near_far_and_current_values() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let runtime = fixture().build_runtime();
    let old = create(&execution, &runtime, "model", "3");
    let model = changed_entities(&old)[0];
    let far = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    release_test_commit_snapshot(&runtime, &old);
    for _ in 0..24 {
        for value in ["5", "3"] {
            let intervening = update(&execution, &runtime, [(model, value)]);
            release_test_commit_snapshot(&runtime, &intervening);
        }
    }
    let near = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let current = update(&execution, &runtime, [(model, "5")]);
    let services = runtime.owner_component_services();
    let branches = ["near", "farx"].map(|name| {
        let (_, source) = services
            .fork_port()
            .observe_fork_source(&BranchId("main".into()))
            .unwrap();
        services
            .fork_port()
            .fork_branch(&execution, BranchId(name.into()), source)
            .unwrap()
            .target_identity()
            .clone()
    });
    // Setup reclamation is outside the measured owner operation.
    while runtime.run_branch_root_reclamation_pass().roots_reclaimed() > 0 {}
    let near_result = measure(execution, &runtime, &near, &branches[0], model);
    let far_result = measure(execution, &runtime, &far, &branches[1], model);
    for selected in [&near_result, &far_result] {
        assert_eq!(
            values(&execution, &runtime, selected, [model, model]),
            ["3", "3"].map(string_aspect_value)
        );
        release_test_commit_snapshot(&runtime, selected);
    }
    assert_eq!(
        values(&execution, &runtime, &current, [model, model]),
        ["5", "5"].map(string_aspect_value)
    );
    release_test_commit_snapshot(&runtime, &current);
}

fn measure(
    execution: ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    historical: &AdmittedRelationalBranchBasis,
    branch: &RelationalBranchIdentity,
    entity: EntityId,
) -> CommitResult {
    let target = runtime.admit_branch_basis(branch).unwrap();
    let transaction = runtime
        .begin_branch_transaction(&target, RelationalTransactionIntent::ordinary())
        .unwrap();
    let services = runtime.owner_component_services();
    let selection = services
        .preparation_port()
        .admit_retained_entity_selection(transaction, historical, entity)
        .unwrap();
    let candidate = services
        .preparation_port()
        .prepare_retained_entity_selection(&execution, selection)
        .unwrap();
    perform(&execution, runtime, candidate)
}
