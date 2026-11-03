pub(crate) use worth_query_bank_testkit_server::facade::approved_payment_workflow::rejection::*;

#[test]
fn rejected_payment_workflow_terminates_without_payment_or_rail_effect() {
    let fixture = ordinary_read_world_with_approval_authentication(
        "rejected-payment-workflow",
        authentication::approval_configuration(),
    );
    let rail_process = spawn_rail();
    let settlement_rail = Arc::new(BankEstateRailTransport::connected_to(
        rail_process.local_addr(),
        rail_process.test_control_addr(),
    ));
    fixture
        .world
        .runtime
        .install_external_effect_transport(settlement_rail.clone())
        .expect("the Bank rail owner installs");
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
    let (instance, proposal, required) =
        journey::prepare_approved_payment_approval(&workflow, &authority);
    let original = support::block_on(workflow.approve(
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Reject,
        authentication::valid_credential(),
        authority.clone(),
        &key("approved-payment:approval:reject"),
    ))
    .expect("the authenticated rejection settles");
    let WorkflowProgressOutcome::Completed(original) = original else {
        panic!("the first rejection must perform: {original:?}");
    };
    assert_eq!(original.node_path(), "approval");
    assert!(!original.replayed());
    let retry = support::block_on(workflow.approve(
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Reject,
        authentication::invalid_credential(),
        authority.clone(),
        &key("approved-payment:approval:reject"),
    ))
    .expect("the exact rejection retries without repeating authentication");
    match retry {
        WorkflowProgressOutcome::RecoveredCommitted(data) => {
            worth_query_bank_testkit_server::facade::approved_payment_workflow::recorded_commit_axes::assert_agrees(&data, original.receipt());
        }
        other => panic!("the rejection retry returns original data: {other:?}"),
    }
    let keys = [key("approved-payment:advance:rejected")];
    let terminal = workflow.run(instance.clone(), authority, &keys);
    assert_eq!(terminal.attempted_steps(), 1);
    assert_eq!(terminal.transitions().len(), 1);
    assert_eq!(terminal.transitions()[0].node_path(), "rejected");
    assert!(matches!(
        terminal.stop(),
        WorthQueryOrdinaryWorkflowRunStop::Terminal
    ));
    assert!(settlement_rail.attempts().is_empty());
    assert_eq!(settlement_rail.completed_effect_count(), 0);
    // Rejecting this workflow attempt does not perform Bank's separate RejectPayment operation.
    let observed = fixture
        .world
        .runtime
        .request(&approver, &scope)
        .on_branch(instance.branch())
        .query(payment(fixture.payment))
        .execute()
        .expect("the rejected workflow branch still exposes the pending payment");
    assert_eq!(observed.rows().len(), 1);
    assert_eq!(observed.rows()[0].status(), PaymentStatus::ApprovalRequired);
    assert_eq!(observed.rows()[0].deciding_principal(), None);
}
