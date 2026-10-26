//! A forked branch that commits after a checkpoint recovers even though its
//! source branch moved on before the checkpoint, so the checkpoint no longer
//! retains the root the fork was taken from. The tail commit's prior cell
//! state equals the branch cell the checkpoint restored; recovery accepts it
//! without readmitting the fork provenance root again.

use super::*;

#[test]
fn a_fork_commits_after_a_checkpoint_taken_once_its_source_moved_on() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = durable_runtime_with_test_schema();
    let storm = BranchId("storm".to_owned());
    create_entity_outcome(execution, &runtime, "source-a");
    let (_, basis) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("committed main is forkable");
    runtime
        .fork_branch(execution, storm.clone(), basis)
        .expect("the fork is admitted");
    create_entity_outcome_on_branch(execution, &runtime, "storm-a", storm.clone());
    create_entity_outcome(execution, &runtime, "source-b");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    create_entity_outcome_on_branch(execution, &runtime, "storm-b", storm.clone());

    let (outcome, recovered) = recover_with(execution, &runtime, durable_runtime_with_test_schema);
    assert_eq!(outcome.recovered_commits, 4);
    assert!(recovered.branch_identity(&storm).is_ok());
    assert_eq!(
        recovered.history().latest_commit(),
        runtime.history().latest_commit()
    );
}
