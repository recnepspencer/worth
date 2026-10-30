pub use std::num::NonZeroUsize;

pub use bank_domain::schema::ApprovePayment;
pub use bank_server::{
    BankApprovedPaymentAssessment, BankApprovedPaymentAssessmentDemand,
    BankApprovedPaymentAssessmentProgress, BankApprovedPaymentWorkflow,
    BankPaymentAssessmentSettlement,
};
pub use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, WorkflowProposalOutcome,
};

pub use super::assertions::{key, require_assessment, require_completed};
pub use super::fixture::{
    ordinary_read_world_with_approval_authentication, principal_id, OrdinaryReadFixture, APPROVER,
};
pub use super::support::request_scope;
pub use super::{authentication, journey};

pub fn world() -> OrdinaryReadFixture {
    ordinary_read_world_with_approval_authentication(
        "approved-payment-assessment-settlement",
        authentication::approval_configuration(),
    )
}

pub fn authority(fixture: &OrdinaryReadFixture) -> ApprovePayment {
    ApprovePayment {
        payment: fixture.payment,
        approver: principal_id(APPROVER),
    }
}

pub fn awaiting_payment_assessment(
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

pub fn policy(attempts: usize, rounds: usize) -> BankPaymentAssessmentSettlement {
    BankPaymentAssessmentSettlement::new(
        NonZeroUsize::new(attempts).unwrap(),
        NonZeroUsize::new(rounds).unwrap(),
    )
}

pub fn advance(
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

pub fn settle(
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
