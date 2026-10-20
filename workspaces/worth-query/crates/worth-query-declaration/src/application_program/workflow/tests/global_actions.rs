use super::*;

#[test]
fn unbound_operation_is_not_a_global_action_body_and_termination_is_not_an_authored_control_arm() {
    let commands = [
        ApplicationWorkflowAuthoringCommand::terminal("done").unwrap(),
        ApplicationWorkflowAuthoringCommand::start("done").unwrap(),
        ApplicationWorkflowAuthoringCommand::termination_action(
            ApplicationWorkflowOperationRef::declared::<ReviewedChange, ProposeChange>(),
        ),
    ];
    let authored = ApplicationWorkflowCommandAdapter::author::<ReviewedChange>(
        "unbound-ending",
        limits(),
        commands,
    )
    .unwrap();
    assert_eq!(
        authored.validate().err().unwrap().kind(),
        ApplicationWorkflowValidationDenialKind::InvalidGlobalAction
    );
    let mut graph =
        ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new("not-an-ending-arm", limits())
            .unwrap();
    let start = graph.operation::<ProposeChange>("start", false).unwrap();
    let done = graph.terminal("done").unwrap();
    graph
        .start(&start)
        .control(&start, ApplicationWorkflowControlOutcome::Completed, &done)
        .control(&start, ApplicationWorkflowControlOutcome::Terminated, &done);
    assert_eq!(
        graph.finish().unwrap().validate().err().unwrap().kind(),
        ApplicationWorkflowValidationDenialKind::UnexpectedControlOutcome
    );
}
