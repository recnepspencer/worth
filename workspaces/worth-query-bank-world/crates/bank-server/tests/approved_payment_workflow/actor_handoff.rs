use super::*;
use bank_domain::{
    model::CustomerRole, queries::account_authorized_users, schema::RevokeAccountAuthorization,
};
use fixture::{ordinary_read_world_with_two_approvers, OWNER, STRANGER};
use worth_query_host::facade::application_entry::{
    WorkflowApprovalDecision, WorthQueryApplicationMutationOutcome,
    WorthQueryApplicationRequestMutationDenial, WorthQueryWorkflowAdvancePreparationDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind;

const SECOND_APPROVER: usize = STRANGER;

#[test]
fn revoked_starter_cannot_advance_but_second_actor_commits_attributed_payment() {
    let fixture = ordinary_read_world_with_two_approvers(
        "approved-payment-actor-handoff",
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
    let starter = fixture.authenticate(APPROVER);
    let actor = fixture.authenticate(SECOND_APPROVER);
    let owner = fixture.authenticate(OWNER);
    let scope = request_scope();
    let starter_workflow = fixture
        .world
        .runtime
        .approved_business_payment(&starter, &scope);
    let actor_workflow = fixture
        .world
        .runtime
        .approved_business_payment(&actor, &scope);
    let starter_authority = ApprovePayment {
        payment: fixture.payment,
        approver: principal_id(APPROVER),
    };
    let actor_authority = ApprovePayment {
        payment: fixture.payment,
        approver: principal_id(SECOND_APPROVER),
    };
    let instance = journey::start_approved_payment_instance(&starter_workflow, &starter_authority);
    let authorized_users = fixture
        .world
        .runtime
        .request(&owner, &scope)
        .on_branch(instance.branch())
        .query(account_authorized_users(fixture.business_account))
        .execute()
        .expect("the Bank owner reads current business-account grants");
    let [account] = authorized_users.rows() else {
        panic!("expected the business account's authorization row");
    };
    let starter_grant = account
        .users()
        .iter()
        .find(|user| {
            user.principal() == principal_id(APPROVER) && user.role() == CustomerRole::Approver
        })
        .expect("the starter has a live Approver authorization");
    let revoked = fixture
        .world
        .runtime
        .request(&owner, &scope)
        .on_branch(instance.branch())
        .mutate(RevokeAccountAuthorization {
            account: fixture.business_account,
            authorization: starter_grant.authorization(),
        })
        .idempotency(&key("approved-payment:handoff:revoke-starter"))
        .execute_in_program(fixture.world.runtime.application_program())
        .expect("the Bank owner revokes the starter's account authorization");
    assert!(matches!(
        revoked,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let denied = starter_workflow.advance(
        instance.clone(),
        starter_authority.clone(),
        &key("approved-payment:handoff:revoked-starter-advance"),
    );
    assert!(
        matches!(
            &denied,
            Err(bank_server::BankApprovedPaymentWorkflowError::Advance(
                WorthQueryWorkflowAdvancePreparationDenial::RequestAdmission(
                    WorthQueryApplicationRequestMutationDenial::Authorization(denial)
                )
            )) if denial.kind() == WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing
        ),
        "the revoked starter must lose current capability authority: {denied:?}"
    );
    let (instance, proposal, approval) = journey::prepare_approved_payment_approval_on_instance(
        &actor_workflow,
        instance,
        &actor_authority,
    );
    require_completed(
        support::block_on(actor_workflow.approve(
            instance.clone(),
            &approval,
            &proposal,
            WorkflowApprovalDecision::Approve,
            authentication::valid_credential(),
            actor_authority.clone(),
            &key("approved-payment:handoff:approve"),
        ))
        .expect("the second authenticated actor approves the retained proposal"),
        "approval",
    );
    let operation = match actor_workflow
        .run(
            instance.clone(),
            actor_authority.clone(),
            &[key("approved-payment:handoff:operation")],
        )
        .stop()
    {
        WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingOperation(
            required,
        )) => required.clone(),
        other => panic!("expected the payment operation wait, got {other:?}"),
    };
    let spoofed = actor_workflow.perform_apply(
        instance.clone(),
        &operation,
        starter_authority,
        &key("approved-payment:handoff:spoof-starter"),
    );
    assert!(
        matches!(
            &spoofed,
            Err(bank_server::BankApprovedPaymentWorkflowError::OperationBinding(
                worth_query_host::facade::application_entry::WorthQueryWorkflowOperationBindingDenial::RequirementMismatch
            ))
        ),
        "the second actor cannot attribute an effect to the former starter: {spoofed:?}"
    );
    assert!(settlement_rail.attempts().is_empty());
    let performed = actor_workflow
        .perform_apply(
            instance.clone(),
            &operation,
            actor_authority,
            &key("approved-payment:handoff:perform"),
        )
        .expect("the second actor reaches the real payment operation")
        .into_performed()
        .expect("the second actor commits the payment effect");
    assert!(performed.newly_committed());
    let observed = fixture
        .world
        .runtime
        .request(&actor, &scope)
        .on_branch(instance.branch())
        .query(payment(fixture.payment))
        .execute()
        .expect("the workflow branch exposes the actual Bank payment");
    assert_eq!(observed.rows().len(), 1);
    assert_eq!(observed.rows()[0].status(), PaymentStatus::Committed);
    assert_eq!(
        observed.rows()[0].deciding_principal(),
        Some(principal_id(SECOND_APPROVER))
    );
    assert_eq!(settlement_rail.completed_effect_count(), 1);
}
