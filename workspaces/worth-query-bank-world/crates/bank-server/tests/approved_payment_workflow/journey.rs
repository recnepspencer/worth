use bank_domain::schema::ApprovePayment;
use bank_server::BankApprovedPaymentWorkflow;
use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef, RequiredWorkflowApproval,
    RequiredWorkflowOperation, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorkflowProposalOutcome, WorthQueryOrdinaryWorkflowRunStop,
    WorthQueryWorkflowAssessmentDemandProgress,
};

use super::approval::require_authenticated_approval_and_replay;
use super::assertions::{key, require_assessment, require_completed};

pub(super) fn prepare_approved_payment_approval(
    workflow: &BankApprovedPaymentWorkflow<'_, '_, '_>,
    authority: &ApprovePayment,
) -> (
    PublishedWorkflowInstanceRef,
    PublishedWorkflowProposalRef,
    RequiredWorkflowApproval,
) {
    let instance = start_approved_payment_instance(workflow, authority);
    prepare_approved_payment_approval_on_instance(workflow, instance, authority)
}

pub(super) fn start_approved_payment_instance(
    starter: &BankApprovedPaymentWorkflow<'_, '_, '_>,
    starter_authority: &ApprovePayment,
) -> PublishedWorkflowInstanceRef {
    let published = match starter
        .publish_definition(
            starter_authority.clone(),
            WorkflowDefinitionExpectedPredecessor::Absent,
            &key("approved-payment:definition"),
        )
        .expect("the Bank-owned workflow definition publishes")
    {
        WorkflowDefinitionPublicationOutcome::Published(published) => published,
        other => panic!("expected definition publication, got {other:?}"),
    };
    let started = match starter
        .start(
            published.definition().clone(),
            starter_authority.clone(),
            &key("approved-payment:instance"),
        )
        .expect("the Bank starts its retained workflow")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected workflow instance, got {other:?}"),
    };
    let instance = started.instance().clone();
    instance
}

pub(super) fn prepare_approved_payment_approval_on_instance(
    actor: &BankApprovedPaymentWorkflow<'_, '_, '_>,
    instance: PublishedWorkflowInstanceRef,
    actor_authority: &ApprovePayment,
) -> (
    PublishedWorkflowInstanceRef,
    PublishedWorkflowProposalRef,
    RequiredWorkflowApproval,
) {
    let proposal = match actor
        .propose(
            instance.clone(),
            actor_authority.clone(),
            &key("approved-payment:proposal"),
        )
        .expect("the typed payment proposal publishes")
    {
        WorkflowProposalOutcome::Published(proposal) => proposal,
        other => panic!("expected workflow proposal, got {other:?}"),
    };

    require_assessment(
        actor
            .advance(
                instance.clone(),
                actor_authority.clone(),
                &key("approved-payment:advance:payment"),
            )
            .expect("the proposal advances to payment assessment"),
        "review/payment",
    );
    let mut payment_demand = actor
        .begin_payment_assessment(
            instance.clone(),
            actor_authority.clone(),
            &key("approved-payment:assessment:payment:settle"),
        )
        .expect("the payment assessment demand starts");
    let payment_assessment = match actor
        .settle_payment_assessment(&mut payment_demand)
        .expect("the payment assessment advances")
    {
        WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => settled,
        WorthQueryWorkflowAssessmentDemandProgress::Pending => {
            panic!("the fixture's payment assessment must settle within admitted work")
        }
    };
    require_completed(
        actor
            .accept_assessment(
                instance.clone(),
                actor_authority.clone(),
                &payment_assessment,
                &key("approved-payment:assessment:payment:accept"),
            )
            .expect("the exact payment assessment is accepted"),
        "review/payment",
    );

    require_assessment(
        actor
            .advance(
                instance.clone(),
                actor_authority.clone(),
                &key("approved-payment:advance:independent"),
            )
            .expect("the payment review advances to independent assessment"),
        "review/independent",
    );
    let mut independent_demand = actor
        .begin_payment_assessment(
            instance.clone(),
            actor_authority.clone(),
            &key("approved-payment:assessment:independent:settle"),
        )
        .expect("the independent assessment demand starts");
    let independent_assessment = match actor
        .settle_payment_assessment(&mut independent_demand)
        .expect("the independent assessment advances")
    {
        WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => settled,
        WorthQueryWorkflowAssessmentDemandProgress::Pending => {
            panic!("the fixture's independent assessment must settle within admitted work")
        }
    };
    require_completed(
        actor
            .accept_assessment(
                instance.clone(),
                actor_authority.clone(),
                &independent_assessment,
                &key("approved-payment:assessment:independent:accept"),
            )
            .expect("the exact independent assessment is accepted"),
        "review/independent",
    );
    let keys = [
        key("approved-payment:advance:evidence"),
        key("approved-payment:advance:approval"),
    ];
    let progressed = actor.run(instance.clone(), actor_authority.clone(), &keys);
    assert_eq!(progressed.attempted_steps(), 2);
    assert_eq!(progressed.transitions().len(), 1);
    assert_eq!(progressed.transitions()[0].node_path(), "review/evidence");
    let approval = match progressed.stop() {
        WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingApproval(
            required,
        )) => required,
        other => panic!("expected typed approval wait after the evidence join, got {other:?}"),
    };
    (instance, proposal.proposal().clone(), approval.clone())
}

pub(super) fn prepare_approved_payment_operation(
    workflow: &BankApprovedPaymentWorkflow<'_, '_, '_>,
    authority: &ApprovePayment,
) -> (PublishedWorkflowInstanceRef, RequiredWorkflowOperation) {
    let (instance, proposal, approval) = prepare_approved_payment_approval(workflow, authority);
    require_authenticated_approval_and_replay(workflow, &instance, &approval, &proposal, authority);

    let keys = [key("approved-payment:advance:operation")];
    let progressed = workflow.run(instance.clone(), authority.clone(), &keys);
    assert_eq!(progressed.attempted_steps(), 1);
    assert!(progressed.transitions().is_empty());
    let operation = match progressed.stop() {
        WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingOperation(
            required,
        )) => required.clone(),
        other => panic!("expected typed payment operation wait, got {other:?}"),
    };
    (instance, operation)
}
