use std::num::NonZeroUsize;

use bank_domain::schema::ApprovePayment;
use bank_server::{
    BankApprovedPaymentAssessment, BankApprovedPaymentAssessmentDemand,
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
fn progressed_stages_settle_and_a_later_call_answers_the_same_demand() {
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
        .retain_payment_assessment(
            instance.clone(),
            authority.clone(),
            policy(1, 1),
            &key("assessment:settle"),
        )
        .expect("the payment assessment demand starts");
    let assessment = settle(&fixture, &mut demand);
    let repeated = match advance(&fixture, &mut demand) {
        BankApprovedPaymentAssessmentProgress::Settled(repeated) => repeated,
        BankApprovedPaymentAssessmentProgress::Progressed => {
            panic!("a settled demand cannot restart native progress");
        }
        BankApprovedPaymentAssessmentProgress::Pending => {
            panic!("a settled demand cannot become an external wait");
        }
    };
    assert_eq!(repeated.required(), assessment.required());
    require_completed(
        workflow
            .accept_assessment(instance, authority, &assessment, &key("assessment:accept"))
            .expect("the settled assessment is accepted"),
        "review/payment",
    );
}

#[test]
fn an_undelivered_readiness_reports_pending_and_a_later_call_resumes_the_same_demand() {
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
        .retain_payment_assessment(
            instance.clone(),
            authority.clone(),
            policy(1, 1),
            &key("assessment:settle"),
        )
        .expect("the payment assessment demand starts");
    fixture
        .world
        .runtime
        .delay_next_output_readiness_delivery_for_test();
    loop {
        match advance(&fixture, &mut demand) {
            BankApprovedPaymentAssessmentProgress::Progressed => continue,
            BankApprovedPaymentAssessmentProgress::Pending => break,
            BankApprovedPaymentAssessmentProgress::Settled(_) => {
                panic!("the delayed readiness must expose its actual Pending hold");
            }
        }
    }
    let assessment = settle(&fixture, &mut demand);
    require_completed(
        workflow
            .accept_assessment(instance, authority, &assessment, &key("assessment:accept"))
            .expect("the resumed assessment is accepted"),
        "review/payment",
    );
}

#[test]
fn notified_rounds_preserve_stage_yields_until_settlement() {
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
        .retain_payment_assessment(
            instance,
            authority,
            policy(1, 16),
            &key("assessment:settle"),
        )
        .expect("the payment assessment demand starts");
    fixture
        .world
        .runtime
        .delay_next_output_readiness_delivery_for_test();
    let settled = settle(&fixture, &mut demand);
    assert_eq!(settled.required().node_path(), "review/payment");
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

fn advance(
    fixture: &OrdinaryReadFixture,
    demand: &mut BankApprovedPaymentAssessmentDemand,
) -> BankApprovedPaymentAssessmentProgress {
    let principal = fixture.authenticate(APPROVER);
    let scope = request_scope();
    fixture
        .world
        .runtime
        .approved_business_payment(&principal, &scope)
        .advance_payment_assessment(demand)
        .expect("fresh actor authority advances the original assessment demand")
}

fn settle(
    fixture: &OrdinaryReadFixture,
    demand: &mut BankApprovedPaymentAssessmentDemand,
) -> BankApprovedPaymentAssessment {
    loop {
        match advance(fixture, demand) {
            BankApprovedPaymentAssessmentProgress::Settled(settled) => return settled,
            BankApprovedPaymentAssessmentProgress::Progressed => continue,
            BankApprovedPaymentAssessmentProgress::Pending => {
                panic!("the synchronous assessment is waiting for external progress");
            }
        }
    }
}
