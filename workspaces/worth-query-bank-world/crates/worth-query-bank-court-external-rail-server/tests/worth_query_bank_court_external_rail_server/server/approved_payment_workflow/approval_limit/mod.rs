pub(crate) use worth_query_bank_testkit_server::facade::approved_payment_workflow::approval_limit::*;

#[test]
fn payment_over_the_approval_limit_is_rejected_before_approval() {
    let fixture = ordinary_read_world_with_payment_amount(
        "approval-limit-exceeded",
        authentication::approval_configuration(),
        15_001,
    );
    let approver = fixture.authenticate(APPROVER);
    let scope = request_scope();
    let workflow = fixture
        .world
        .runtime
        .approved_business_payment(&approver, &scope);
    let authority = ApprovePayment {
        payment: fixture.payment,
        approver: principal_id(APPROVER),
    };
    let (_definition, instance) = journey::start_approved_payment_instance(&workflow, &authority);
    let (_, limit) = journey::reach_approval_limit(&workflow, &instance, &authority);
    assert_eq!(limit.operands().len(), 1);
    assert_eq!(
        limit.operands()[0].name(),
        bank_server::APPROVAL_LIMIT_OPERAND
    );
    let original = match workflow
        .accept_approval_limit(
            instance.clone(),
            &limit,
            authority.clone(),
            &key("approved-payment:condition:limit"),
        )
        .expect("the over-limit amount settles the condition")
    {
        WorkflowProgressOutcome::Completed(performed) => performed,
        other => panic!("the approval limit did not settle: {other:?}"),
    };
    assert_eq!(original.node_path(), "approval/limit");
    assert!(!original.replayed());
    match workflow
        .accept_approval_limit(
            instance.clone(),
            &limit,
            authority.clone(),
            &key("approved-payment:condition:limit"),
        )
        .expect("the exact condition replays")
    {
        WorkflowProgressOutcome::RecoveredCommitted(data) => {
            worth_query_bank_testkit_server::facade::recorded_commit_axes::assert_agrees(
                &data,
                original.receipt(),
            );
        }
        other => panic!("the condition retry must return original data: {other:?}"),
    }
    let terminal = workflow.run(
        instance,
        authority,
        &[key("approved-payment:advance:rejected")],
    );
    assert_eq!(terminal.transitions().len(), 1);
    assert_eq!(terminal.transitions()[0].node_path(), "rejected");
    assert!(matches!(
        terminal.stop(),
        WorthQueryOrdinaryWorkflowRunStop::Terminal
    ));
}
