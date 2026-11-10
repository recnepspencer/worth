//! Journal replay preserves the actual admitted immutable owner content root.
use super::*;
use worth_foundational::FoundationalBranchTarget;
use worth_relational::facade::{history::BranchId, runtime::RelationalRuntime};
#[test]
fn tail_replay_reconstructs_the_owner_content_root() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity_outcome(execution, &runtime, "tail-content-root-parity");
    let expected = current_main_descriptor(&runtime);
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
        .expect("one production tail commit reconstructs");

    assert_eq!(current_main_descriptor(&recovered), expected);
}

fn current_main_descriptor(
    runtime: &RelationalRuntime,
) -> worth_relational::facade::branch::RelationalBranchRootDescriptor {
    let state = runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .expect("main branch exists");
    match state.observation().target() {
        FoundationalBranchTarget::Basis(target) => target.roots().clone(),
        FoundationalBranchTarget::Empty => panic!("committed main branch has an exact target"),
    }
}
