pub(crate) use worth_query_worlds::facade::certification::application_graph::workflow_assessment::join_policy::*;

#[test]
fn completed_failing_inventory_settles_the_declared_join_failure_branch() {
    let (application, instance) = failing_join(
        ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        490,
    );
    match advance_instance(&application, instance.clone(), 498)
        .expect("complete failing evidence must settle the join")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/join")
        }
        other => panic!("expected a settled failing join, got {other:?}"),
    }
    match advance_instance(&application, instance, 499)
        .expect("the failed join must select its declared successor")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "rejected")
        }
        other => panic!("expected rejection terminal completion, got {other:?}"),
    }
}

#[test]
fn all_completed_policy_accepts_a_complete_failing_inventory() {
    let (application, instance) = failing_join(
        ApplicationWorkflowEvidenceJoinPolicy::AllRequiredCompleted,
        500,
    );
    assert!(matches!(
        advance_instance(&application, instance.clone(), 508),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    match advance_instance(&application, instance, 509)
        .expect("the satisfied all-completed join must select completion")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "completed")
        }
        other => panic!("expected completion terminal, got {other:?}"),
    }
}
