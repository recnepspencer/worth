//! The approved-payment workflow's limit is an expression over the payment's
//! current amount: a payment over it is rejected before any approver acts.

use super::*;
use fixture::ordinary_read_world_with_payment_amount;
use worth_query_host::facade::application_entry::WorthQueryOrdinaryWorkflowRunStop;

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
    let instance = journey::start_approved_payment_instance(&workflow, &authority);
    let (_, limit) = journey::reach_approval_limit(&workflow, &instance, &authority);
    assert_eq!(limit.operands().len(), 1);
    assert_eq!(
        limit.operands()[0].name(),
        bank_server::APPROVAL_LIMIT_OPERAND
    );
    for replayed in [false, true] {
        let outcome = workflow
            .accept_approval_limit(
                instance.clone(),
                &limit,
                authority.clone(),
                &key("approved-payment:condition:limit"),
            )
            .expect("the over-limit amount settles the condition");
        let WorkflowProgressOutcome::Completed(performed) = outcome else {
            panic!("the approval limit did not settle: {outcome:?}");
        };
        assert_eq!(performed.node_path(), "approval/limit");
        assert_eq!(performed.replayed(), replayed);
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
