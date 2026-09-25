use bank_domain::schema::ApprovePayment;
use bank_server::BankApprovedPaymentWorkflow;
use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, RequiredWorkflowOperation, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorkflowProposalOutcome, WorthQueryWorkflowAssessmentDemandProgress,
};

use super::approval::require_authenticated_approval_and_replay;
use super::assertions::{key, require_assessment, require_completed};

pub(super) fn prepare_approved_payment_operation(
    workflow: &BankApprovedPaymentWorkflow<'_, '_, '_>,
    authority: &ApprovePayment,
) -> (PublishedWorkflowInstanceRef, RequiredWorkflowOperation) {
    let published = match workflow
        .publish_definition(
            authority.clone(),
            WorkflowDefinitionExpectedPredecessor::Absent,
            &key("approved-payment:definition"),
        )
        .expect("the Bank-owned workflow definition publishes")
    {
        WorkflowDefinitionPublicationOutcome::Published(published) => published,
        other => panic!("expected definition publication, got {other:?}"),
    };
    let started = match workflow
        .start(
            published.definition().clone(),
            authority.clone(),
            &key("approved-payment:instance"),
        )
        .expect("the Bank starts its retained workflow")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected workflow instance, got {other:?}"),
    };
    let instance = started.instance().clone();
    let proposal = match workflow
        .propose(
            instance.clone(),
            authority.clone(),
            &key("approved-payment:proposal"),
        )
        .expect("the typed payment proposal publishes")
    {
        WorkflowProposalOutcome::Published(proposal) => proposal,
        other => panic!("expected workflow proposal, got {other:?}"),
    };

    require_assessment(
        workflow
            .advance(
                instance.clone(),
                authority.clone(),
                &key("approved-payment:advance:payment"),
            )
            .expect("the proposal advances to payment assessment"),
        "review/payment",
    );
    let mut payment_demand = workflow
        .begin_payment_assessment(
            instance.clone(),
            authority.clone(),
            &key("approved-payment:assessment:payment:settle"),
        )
        .expect("the payment assessment demand starts");
    let payment_assessment = match workflow
        .settle_payment_assessment(&mut payment_demand)
        .expect("the payment assessment advances")
    {
        WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => settled,
        WorthQueryWorkflowAssessmentDemandProgress::Pending => {
            panic!("the fixture's payment assessment must settle within admitted work")
        }
    };
    require_completed(
        workflow
            .accept_assessment(
                instance.clone(),
                authority.clone(),
                &payment_assessment,
                &key("approved-payment:assessment:payment:accept"),
            )
            .expect("the exact payment assessment is accepted"),
        "review/payment",
    );

    require_assessment(
        workflow
            .advance(
                instance.clone(),
                authority.clone(),
                &key("approved-payment:advance:independent"),
            )
            .expect("the payment review advances to independent assessment"),
        "review/independent",
    );
    let mut independent_demand = workflow
        .begin_payment_assessment(
            instance.clone(),
            authority.clone(),
            &key("approved-payment:assessment:independent:settle"),
        )
        .expect("the independent assessment demand starts");
    let independent_assessment = match workflow
        .settle_payment_assessment(&mut independent_demand)
        .expect("the independent assessment advances")
    {
        WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => settled,
        WorthQueryWorkflowAssessmentDemandProgress::Pending => {
            panic!("the fixture's independent assessment must settle within admitted work")
        }
    };
    require_completed(
        workflow
            .accept_assessment(
                instance.clone(),
                authority.clone(),
                &independent_assessment,
                &key("approved-payment:assessment:independent:accept"),
            )
            .expect("the exact independent assessment is accepted"),
        "review/independent",
    );
    require_completed(
        workflow
            .advance(
                instance.clone(),
                authority.clone(),
                &key("approved-payment:advance:evidence"),
            )
            .expect("both assessments satisfy the evidence join"),
        "review/evidence",
    );

    let approval = match workflow
        .advance(
            instance.clone(),
            authority.clone(),
            &key("approved-payment:advance:approval"),
        )
        .expect("the evidence join advances")
    {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("expected approval requirement, got {other:?}"),
    };
    require_authenticated_approval_and_replay(
        workflow,
        &instance,
        &approval,
        proposal.proposal(),
        authority,
    );

    let operation = match workflow
        .advance(
            instance.clone(),
            authority.clone(),
            &key("approved-payment:advance:operation"),
        )
        .expect("approval advances to the real payment operation")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected payment operation requirement, got {other:?}"),
    };
    (instance, operation)
}
