use super::*;

#[test]
fn rejection_routes_to_its_declared_terminal_and_replays_before_that_successor() {
    let (application, _, instance, proposal, required, _) = approval_journey("unused", 600);
    for replayed in [false, true] {
        match approve_instance(
            &application,
            instance.clone(),
            &required,
            &proposal,
            WorkflowApprovalDecision::Reject,
            610,
        )
        .expect("rejection and its exact retry must prepare")
        {
            WorkflowProgressOutcome::Completed(performed) => {
                assert_eq!(performed.node_path(), "approval");
                assert_eq!(performed.replayed(), replayed);
            }
            other => panic!("expected a performed rejection, got {other:?}"),
        }
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
