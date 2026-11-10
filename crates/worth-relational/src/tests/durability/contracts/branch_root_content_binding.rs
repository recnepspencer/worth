use super::*;
use worth_foundational::FoundationalBranchTarget;

#[test]
fn tail_replay_reconstructs_the_owner_content_root() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "tail-content-root-parity");
    let expected = current_main_descriptor(&runtime);
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();

    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect("one production tail commit reconstructs");

    assert_eq!(current_main_descriptor(&recovered), expected);
}

fn current_main_descriptor(
    runtime: &RelationalRuntime,
) -> crate::branch::RelationalBranchRootDescriptor {
    let state = runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .expect("main branch exists");
    match state.observation().target() {
        FoundationalBranchTarget::Basis(target) => target.roots().clone(),
        FoundationalBranchTarget::Empty => panic!("committed main branch has an exact target"),
    }
}
