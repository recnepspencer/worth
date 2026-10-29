pub(crate) use worth_query_worlds::facade::certification::application_graph::workflow_approval::instance_cancellation_capacity::*;

#[test]
fn an_instance_that_used_its_whole_capacity_still_cancels_and_replays() {
    let (application, instance) = at_capacity(90_000);
    let ended = cancelled(cancel_instance(&application, instance.clone(), 90_030));
    assert!(!ended.replayed());
    assert!(ended.performed_node_paths().is_empty());
    let replay = cancelled(cancel_instance(&application, instance.clone(), 90_030));
    assert!(replay.replayed());
    assert_eq!(
        cancellation_denial(cancel_instance(&application, instance, 90_031)),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCancelled,
    );
}

#[test]
fn a_cold_full_instance_cancels_and_replays_from_settled_history() {
    let (application, instance) = at_capacity(90_100);
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    match propose_authoring_instance(&application, instance.clone(), 90_120) {
        Err(WorthQueryWorkflowProposalPreparationDenial::ProposalPreparation(
            WorkflowProposalPreparationDenial::Attempt(denial),
        )) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
            "a cold read refuses a full instance before commit, as a warm one does",
        ),
        other => panic!("a cold full instance takes no further step: {other:?}"),
    }
    let ended = cancelled(cancel_instance(&application, instance.clone(), 90_130));
    assert!(!ended.replayed());
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    let replay = cancelled(cancel_instance(&application, instance, 90_130));
    assert!(
        replay.replayed(),
        "the replay reads a full history in bounds"
    );
}
