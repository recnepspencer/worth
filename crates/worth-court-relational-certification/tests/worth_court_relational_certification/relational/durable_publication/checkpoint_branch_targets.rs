use super::branch_history_publications::{create_entity_outcome_on_branch, fork};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::create_entity_outcome;
use worth_testkit_relational::durable_publication::durable_runtime;

use super::checkpoint_recovery_observation::snapshot_for_branch;
use super::checkpoint_source_capture::capture_durably;
use worth_relational::facade::durability::RecoveryVerificationMode;
#[test]
fn checkpoint_recovery_readmits_shared_root_once_for_sibling_heads() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity_outcome(execution, &runtime, "shared-root-seed");
    fork(execution, &runtime, "sibling");
    let sibling = BranchId("sibling".into());
    capture_durably(execution, &runtime, &medium).unwrap();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();

    let mut recovered = durable_runtime(&medium);
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    assert_eq!(
        current_branch_entity_count(execution, &recovered, &BranchId("main".to_owned())),
        1
    );
    assert_eq!(
        current_branch_entity_count(execution, &recovered, &sibling),
        1
    );
    let main = recovered
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let sibling = recovered.branch_reference_state(&sibling).unwrap();
    assert_eq!(main.observation().target(), sibling.observation().target());
}

#[test]
fn tail_recovery_resolves_the_checkpoint_target_root_not_the_fork_source_head() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity_outcome(execution, &runtime, "shared-seed");
    fork(execution, &runtime, "storm");
    let storm = BranchId("storm".into());
    capture_durably(execution, &runtime, &medium)
        .expect("checkpoint retains the shared pre-divergence root");

    create_entity_outcome(execution, &runtime, "main-tail-only");
    create_entity_outcome_on_branch(execution, &runtime, "storm-tail-only", storm.clone());
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();

    let mut recovered = durable_runtime(&medium);
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect("divergent tail commits recover from their exact pre-commit roots");

    let main_count =
        current_branch_entity_count(execution, &recovered, &BranchId("main".to_owned()));
    let storm_count = current_branch_entity_count(execution, &recovered, &storm);
    assert_eq!(main_count, 2, "main contains seed plus its own tail write");
    assert_eq!(
        storm_count, 2,
        "storm excludes the source's later tail write"
    );
}

fn current_branch_entity_count(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
) -> usize {
    let snapshot = snapshot_for_branch(runtime, branch_id);
    runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .expect("read request remains live")
        .expect("recovered branch snapshot is readable")
        .entities()
        .len()
}
