use std::num::NonZeroUsize;

use bank_domain::schema::ApprovePayment;
use bank_server::{
    BankApprovedPaymentAssessmentProgress, BankApprovedPaymentWorkflow,
    BankPaymentAssessmentSettlement,
};
use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, WorkflowProposalOutcome,
};

use super::assertions::{key, require_assessment, require_completed};
use super::fixture::{
    ordinary_read_world_with_approval_authentication, principal_id, OrdinaryReadFixture, APPROVER,
};
use super::support::request_scope;
use super::{authentication, journey};

#[test]
fn a_spent_round_reports_pending_and_a_later_call_resumes_the_same_demand() {
    let fixture = world();
    let approver = fixture.authenticate(APPROVER);
    let scope = request_scope();
    let workflow = fixture
        .world
        .runtime
        .approved_business_payment(&approver, &scope);
    let authority = authority(&fixture);
    let instance = awaiting_payment_assessment(&workflow, &authority);
    let mut demand = workflow
        .begin_payment_assessment(
            instance.clone(),
            authority.clone(),
            policy(1, 1),
            &key("assessment:settle"),
        )
        .expect("the payment assessment demand starts");
    assert!(
        matches!(
            workflow.settle_payment_assessment(&mut demand),
            Ok(BankApprovedPaymentAssessmentProgress::Pending)
        ),
        "one source reading cannot settle the assessment"
    );
    let assessment = (0..8)
        .find_map(|_| match workflow.settle_payment_assessment(&mut demand) {
            Ok(BankApprovedPaymentAssessmentProgress::Settled(settled)) => Some(settled),
            Ok(BankApprovedPaymentAssessmentProgress::Pending) => None,
            Err(denial) => panic!("the resumed demand must keep advancing: {denial}"),
        })
        .expect("later calls resume the same demand to settlement");
    require_completed(
        workflow
            .accept_assessment(instance, authority, &assessment, &key("assessment:accept"))
            .expect("the resumed assessment is accepted"),
        "review/payment",
    );
}

#[test]
fn notified_rounds_carry_one_call_to_settlement() {
    let fixture = world();
    let approver = fixture.authenticate(APPROVER);
    let scope = request_scope();
    let workflow = fixture
        .world
        .runtime
        .approved_business_payment(&approver, &scope);
    let authority = authority(&fixture);
    let instance = awaiting_payment_assessment(&workflow, &authority);
    let mut demand = workflow
        .begin_payment_assessment(
            instance,
            authority,
            policy(1, 16),
            &key("assessment:settle"),
        )
        .expect("the payment assessment demand starts");
    assert!(
        matches!(
            workflow.settle_payment_assessment(&mut demand),
            Ok(BankApprovedPaymentAssessmentProgress::Settled(_))
        ),
        "each advancing round notifies the demand, so the next round runs"
    );
}

fn world() -> OrdinaryReadFixture {
    ordinary_read_world_with_approval_authentication(
        "approved-payment-assessment-settlement",
        authentication::approval_configuration(),
    )
}

fn authority(fixture: &OrdinaryReadFixture) -> ApprovePayment {
    ApprovePayment {
        payment: fixture.payment,
        approver: principal_id(APPROVER),
    }
}

fn awaiting_payment_assessment(
    workflow: &BankApprovedPaymentWorkflow<'_, '_, '_>,
    authority: &ApprovePayment,
) -> PublishedWorkflowInstanceRef {
    let instance = journey::start_approved_payment_instance(workflow, authority);
    let proposal = workflow
        .propose(
            instance.clone(),
            authority.clone(),
            &key("assessment:proposal"),
        )
        .expect("the typed payment proposal publishes");
    assert!(matches!(proposal, WorkflowProposalOutcome::Published(_)));
    require_assessment(
        workflow
            .advance(
                instance.clone(),
                authority.clone(),
                &key("assessment:advance"),
            )
            .expect("the proposal advances to payment assessment"),
        "review/payment",
    );
    instance
}

fn policy(attempts: usize, rounds: usize) -> BankPaymentAssessmentSettlement {
    BankPaymentAssessmentSettlement::new(
        NonZeroUsize::new(attempts).unwrap(),
        NonZeroUsize::new(rounds).unwrap(),
    )
}
