//! Equivalent receiving-owner work for nearby and distant retained payloads.
use crate::relational::relational_certification::owner_service_completion::retained_entity_selection::{create, update, values, fixture, perform};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};
use worth_testkit_relational::aspect_storage::publications::{changed_entities, release_test_commit_snapshot, string_aspect_value};
use worth_relational::facade::{
    branch::{AdmittedRelationalBranchBasis, RelationalBranchIdentity},
    history::BranchId,
    inspection::{RelationalMvccCostScope, RelationalMvccCounterObservation},
    identity::EntityId, runtime::RelationalRuntime,
    mvcc::RelationalTransactionIntent, transactions::CommitResult,
};

#[test]
fn equal_payload_jumps_charge_the_same_receiving_owner_work_at_near_and_distant_history_nodes() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let mut runtime = fixture().build_runtime();
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
    assert!(near_result.1 > 0);
    assert_eq!(near_result.1, far_result.1, "actual request charged work");
    assert_eq!(
        near_result.2, far_result.2,
        "actual request peak charged memory"
    );
    let near_cost = near_result.3.sharing_cost_delta();
    let far_cost = far_result.3.sharing_cost_delta();
    assert_eq!(
        (
            near_cost.copied_truth_bytes,
            near_cost.copied_commit_envelopes,
            near_cost.publication_persistent_index_path_nodes,
            near_cost.publication_content_values_hashed,
            near_cost.publication_new_root_authoritative_bytes,
            near_cost.retained_history_head_lookups
        ),
        (
            far_cost.copied_truth_bytes,
            far_cost.copied_commit_envelopes,
            far_cost.publication_persistent_index_path_nodes,
            far_cost.publication_content_values_hashed,
            far_cost.publication_new_root_authoritative_bytes,
            far_cost.retained_history_head_lookups
        )
    );
    assert_eq!(near_result.3.branch_population_scans(), 0);
    assert_eq!(far_result.3.branch_population_scans(), 0);
    assert_eq!(
        near_result.3.canonical_settlement_route_visits(),
        far_result.3.canonical_settlement_route_visits()
    );
    for selected in [&near_result.0, &far_result.0] {
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
) -> (CommitResult, u64, u64, RelationalMvccCounterObservation) {
    let scope = RelationalMvccCostScope::capture(
        runtime,
        vec![branch.clone(), historical.identity().clone()],
    );
    let (committed, report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
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
        })
        .unwrap();
    let counters = runtime.observe_mvcc_counters(&scope).unwrap();
    (
        committed,
        report.charged_work(),
        report.physical().peak_charged_memory_bytes(),
        counters,
    )
}
