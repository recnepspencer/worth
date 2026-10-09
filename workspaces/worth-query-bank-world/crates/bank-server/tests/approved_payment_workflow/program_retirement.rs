//! P2 removes ordinary payment approval while preserving the installed schema.

use bank_domain::schema::ApprovePayment;
use bank_server::{BankApplicationP2, BankProgramAdoptionPreparationDenial};
use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorthQueryApplicationRequestMutationDenial, WorthQueryBranchAdoptionPublicationOutcome,
};

use super::assertions::key;
use super::authentication::approval_configuration;
use super::fixture::{ordinary_read_world_with_approval_authentication, principal_id, APPROVER};
use super::journey::start_approved_payment_instance;
use super::support::request_scope;

#[test]
fn p2_adoption_does_not_reopen_ordinary_payment_approval() {
    let fixture = ordinary_read_world_with_approval_authentication(
        "p2-ordinary-payment-retirement",
        approval_configuration(),
    );
    let runtime = &fixture.world.runtime;
    let approver = fixture.authenticate(APPROVER);
    let scope = request_scope();
    let branch = runtime.current_branch();
    let target = runtime
        .supported_program_revision::<BankApplicationP2>()
        .expect("Bank P2 is explicitly rostered");
    let prepared = runtime
        .prepare_branch_program_adoption::<BankApplicationP2>(&approver, &scope, branch, 128)
        .expect("a branch without outstanding workflow custody can adopt P2");
    assert!(matches!(
        prepared.publish(),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
    let adopted = runtime
        .inspect_branch_program(&approver, &scope, branch)
        .expect("the Bank entry observes the selected program");
    assert_eq!(adopted.revision(), &target);

    let ordinary = runtime
        .request(&approver, &scope)
        .on_branch(branch)
        .mutate(ApprovePayment {
            payment: fixture.payment,
            approver: principal_id(APPROVER),
        })
        .idempotency(&key("p2:ordinary-payment-approval"))
        .execute(worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation);
    assert!(matches!(
        ordinary,
        Err(WorthQueryApplicationRequestMutationDenial::RequiresWorkflowTransition)
    ));
}

#[test]
fn p2_retirement_waits_for_the_exact_workflow_instance_to_finish() {
    let fixture = ordinary_read_world_with_approval_authentication(
        "p2-retirement-live-payment",
        approval_configuration(),
    );
    let runtime = &fixture.world.runtime;
    let approver = fixture.authenticate(APPROVER);
    let scope = request_scope();
    let branch = runtime.current_branch();
    let authority = ApprovePayment {
        payment: fixture.payment,
        approver: principal_id(APPROVER),
    };
    let workflow = runtime.approved_business_payment(&approver, &scope);
    let instance = start_approved_payment_instance(&workflow, &authority);
    let definition = match workflow
        .publish_definition(
            authority,
            WorkflowDefinitionExpectedPredecessor::Absent,
            &key("approved-payment:definition"),
        )
        .expect("the original definition publication replays")
    {
        WorkflowDefinitionPublicationOutcome::Published(published) => {
            published.definition().clone()
        }
        other => panic!("expected the retained definition, got {other:?}"),
    };
    assert_eq!(definition.branch(), instance.branch());

    let denied = runtime.prepare_branch_program_adoption_retiring_definition::<BankApplicationP2>(
        &approver,
        &scope,
        branch,
        &definition,
        4_096,
    );
    assert!(matches!(
        denied,
        Err(BankProgramAdoptionPreparationDenial::LiveWorkflowInstances)
    ));
}
