use super::*;
use std::sync::Arc;
use worth_relational::facade::{
    durability::RecoveredRelationalRuntimeAuthority, history::BranchId, runtime::RelationalRuntime,
};

fn recovered_unassociated_inventory(
    execution: &ExecutionRequest<'_, '_>,
) -> (
    RelationalRuntime,
    RelationalRuntime,
    RecoveredRelationalRuntimeAuthority,
    Arc<TestDurableMedium>,
) {
    let medium = TestDurableMedium::new();
    let source = durable_runtime(&medium);
    create_entity(execution, &source, "selected-checkpoint-head");
    medium.file_checkpoint(capture(execution, &source));
    create_entity(execution, &source, "uncovered-commit-tail");
    let forks = source.fork_port();
    let (_, basis) = forks
        .observe_fork_source(source.main_branch_identity().branch_id())
        .unwrap();
    forks
        .fork_branch(
            execution,
            BranchId("uncovered-transition-tail".into()),
            basis,
        )
        .unwrap();
    let recorded = current(execution, &source);
    // This existing issuer reads the selected checkpoint and BOTH full tails
    // from the same externally retained original medium.
    let (runtime, authority) = recover(execution, &medium);
    let admitted = authority
        .admit_recorded_component(execution, &runtime, &recorded)
        .unwrap()
        .into_basis();
    assert_recorded_basis(&runtime, &admitted, &recorded);
    runtime
        .fork_port()
        .observe_fork_source(&BranchId("uncovered-transition-tail".into()))
        .expect("actual uncovered transition was replayed");
    (source, runtime, authority, medium)
}

#[test]
fn recovered_associated_inventory_rejects_foreign_live_runtime_even_when_empty() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (source, runtime, authority, _medium) = recovered_unassociated_inventory(execution);
    assert_ne!(source.runtime_instance_id(), runtime.runtime_instance_id());
    assert!(source
        .durability_authority()
        .discover_all_associated_effects(execution)
        .unwrap()
        .is_empty());
    assert!(matches!(
        authority.discover_recovered_associated_effects(execution, &source),
        Err(Refusal::ForeignRuntime)
    ));
}

#[test]
fn recovered_associated_inventory_admits_matching_checkpoint_and_both_tail_runtime() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (_source, runtime, authority, _medium) = recovered_unassociated_inventory(execution);
    assert!(authority
        .discover_recovered_associated_effects(execution, &runtime)
        .unwrap()
        .is_empty());
}
