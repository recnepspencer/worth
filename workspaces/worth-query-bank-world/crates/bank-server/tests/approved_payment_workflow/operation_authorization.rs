//! Ordinary permission denial preserves an issued workflow operation's single-use slot.

use super::*;
use bank_domain::{
    model::CustomerRole, queries::account_authorized_users, schema::RevokeAccountAuthorization,
};
use fixture::{ordinary_read_world_with_two_approvers, OWNER, STRANGER};
use worth_query_host::facade::application_entry::{
    WorkflowApprovalDecision, WorthQueryApplicationRequestMutationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryOperationAuthorizationDenialKind, WorthQueryOperationAuthorizationRefusalKind,
};

#[test]
fn revoked_starter_cannot_execute_or_spend_guarded_operation() {
    let fixture = ordinary_read_world_with_two_approvers(
        "approved-payment-operation-authorization",
        authentication::approval_configuration(),
    );
    let rail_process = spawn_rail();
    let rail = Arc::new(BankEstateRailTransport::connected_to(
        rail_process.local_addr(),
        rail_process.test_control_addr(),
    ));
    fixture
        .world
        .runtime
        .install_external_effect_transport(rail.clone())
        .expect("the real Bank settlement rail installs");
    let starter = fixture.authenticate(APPROVER);
    let actor = fixture.authenticate(STRANGER);
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
    let starter_input = ApprovePayment {
        payment: fixture.payment,
        approver: principal_id(APPROVER),
    };
    let actor_input = ApprovePayment {
        payment: fixture.payment,
        approver: principal_id(STRANGER),
    };
    let instance = journey::start_approved_payment_instance(&starter_workflow, &starter_input);
    let (instance, proposal, approval) = journey::prepare_approved_payment_approval_on_instance(
        &actor_workflow,
        instance,
        &actor_input,
    );
    require_completed(
        support::block_on(actor_workflow.approve(
            instance.clone(),
            &approval,
            &proposal,
            WorkflowApprovalDecision::Approve,
            authentication::valid_credential(),
            actor_input.clone(),
            &key("approved-payment:authorization:approve"),
        ))
        .expect("the second approver signs the actual proposal"),
        "approval",
    );
    let operation = match actor_workflow
        .advance(
            instance.clone(),
            actor_input.clone(),
            &key("approved-payment:authorization:operation"),
        )
        .expect("the authorized actor reaches the guarded operation")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected an issued payment operation, got {other:?}"),
    };
    let owning_operation = starter_workflow
        .retain_apply(
            &operation,
            actor_input.clone(),
            key("approved-payment:authorization:original-owner"),
        )
        .expect("construct the exact original owner before revocation without an effect");
    let grants = fixture
        .world
        .runtime
        .request(&owner, &scope)
        .on_branch(instance.branch())
        .query(account_authorized_users(fixture.business_account))
        .execute()
        .expect("the owner discovers authentic account authorization");
    let [account] = grants.rows() else {
        panic!("expected the business account grant row")
    };
    let grant = account
        .users()
        .iter()
        .find(|user| {
            user.principal() == principal_id(APPROVER) && user.role() == CustomerRole::Approver
        })
        .expect("the starter has the grant being revoked");
    assert_eq!(instance.branch(), fixture.world.runtime.current_branch());
    let revoked_scope = crate::support::request_scope();
    let revoked_request = fixture.world.runtime.request(&owner, &revoked_scope);
    let source_operation = fixture
        .world
        .runtime
        .mutate(bank_server::mutations::revoke_account_access(
            RevokeAccountAuthorization {
                account: fixture.business_account,
                authorization: grant.authorization(),
            },
        ))
        .as_principal(&owner)
        .controls(bank_server::BankMutationControls::new(
            scope.clone(),
            key("approved-payment:authorization:revoke"),
        ))
        .retain_operation()
        .unwrap_or_else(|(_, cause)| panic!("declared source action: {cause:?}"));
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
        revoked,
    ) = source_operation.advance(&revoked_request)
    else {
        panic!("fresh source completion")
    };
    assert!(matches!(
        revoked.evidence(),
        worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
    ));
    revoked
        .close(&revoked_request)
        .unwrap_or_else(|(_, cause)| panic!("lawful source close: {cause:?}"));

    let denied_scope = request_scope();
    let (owning_operation,denial)=match owning_operation.advance(&fixture.world.runtime.request(&starter,&denied_scope).on_branch(instance.branch())) {
        worth_query_host::facade::application_entry::WorthQueryWorkflowOperationAdvance::Refused{operation,denial}=>(operation,denial),
        _=>panic!("ordinary authorization refusal preserves exact owning descriptor and slot"),
    };
    assert!(
        matches!(denial,worth_query_host::facade::application_entry::WorthQueryWorkflowOperationDenial::Source(
        worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(WorthQueryApplicationRequestMutationDenial::Authorization(ref denial)))
        if denial.kind()==WorthQueryOperationAuthorizationDenialKind::Refused(WorthQueryOperationAuthorizationRefusalKind::PermissionDenied)),
        "exact ordinary PermissionDenied must survive owning composition"
    );
    assert!(rail.attempts().is_empty());
    assert_eq!(rail.completed_effect_count(), 0);
    let pending = fixture
        .world
        .runtime
        .request(&actor, &scope)
        .on_branch(instance.branch())
        .query(payment(fixture.payment))
        .execute()
        .expect("the actual payment remains readable");
    assert_eq!(pending.rows().len(), 1);
    assert_eq!(pending.rows()[0].status(), PaymentStatus::ApprovalRequired);
    assert_eq!(pending.rows()[0].deciding_principal(), None);
    let still_required = match actor_workflow
        .advance(
            instance.clone(),
            actor_input.clone(),
            &key("approved-payment:authorization:still-awaiting"),
        )
        .expect("denial leaves the workflow awaiting its operation")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected unchanged operation wait, got {other:?}"),
    };
    assert_eq!(still_required.branch(), operation.branch());
    assert_eq!(still_required.instance(), operation.instance());
    assert_eq!(still_required.occurrence(), operation.occurrence());
    assert_eq!(
        still_required.transition_identity(),
        operation.transition_identity()
    );
    assert_eq!(still_required.input_identity(), operation.input_identity());
    // The independently observed handle must not replace the original single-use slot.
    drop(still_required);
    let fresh_scope = request_scope();
    let completed=match owning_operation.advance(&fixture.world.runtime.request(&actor,&fresh_scope).on_branch(instance.branch())) {
        worth_query_host::facade::application_entry::WorthQueryWorkflowOperationAdvance::Completed(done)=>done,
        _=>panic!("lawful second approver advances SAME original owner after denial"),
    };
    assert!(matches!(
        completed.evidence(),
        worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
    ));
    let committed = fixture
        .world
        .runtime
        .request(&actor, &scope)
        .on_branch(instance.branch())
        .query(payment(fixture.payment))
        .execute()
        .expect("the real payment consequence is visible");
    assert_eq!(committed.rows().len(), 1);
    assert_eq!(committed.rows()[0].status(), PaymentStatus::Committed);
    assert_eq!(
        committed.rows()[0].deciding_principal(),
        Some(principal_id(STRANGER))
    );
    assert_eq!(rail.attempts().len(), 1);
    assert_eq!(rail.completed_effect_count(), 1);
    completed
        .close(
            &fixture
                .world
                .runtime
                .request(&actor, &fresh_scope)
                .on_branch(instance.branch()),
        )
        .unwrap_or_else(|_| panic!("real original payment owner cleanup"));
}
