pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_approval::rejection::*;

#[test]
fn rejection_routes_to_its_declared_terminal_and_replays_before_that_successor() {
    let (application, _, instance, proposal, required, _) = approval_journey("unused", 600);
    let original = match approve_instance(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Reject,
        610,
    )
    .expect("rejection prepares")
    {
        WorkflowProgressOutcome::Completed(performed) => performed,
        other => panic!("the rejection must perform: {other:?}"),
    };
    assert_eq!(original.node_path(), "approval");
    assert!(!original.replayed());
    match approve_instance(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Reject,
        610,
    )
    .expect("exact retry prepares")
    {
        WorkflowProgressOutcome::RecoveredCommitted(data) => {
            crate::recorded_receipt::assert_agrees(&data, original.receipt())
        }
        other => panic!("exact rejection retry returns data: {other:?}"),
    }
    match advance_instance(&application, instance, 611)
        .expect("the rejected successor terminal must prepare")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "rejected")
        }
        other => panic!("expected the rejected terminal, got {other:?}"),
    }
}
