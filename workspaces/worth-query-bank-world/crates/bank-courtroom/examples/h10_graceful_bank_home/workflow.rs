//! Genuine published Bank proposal, held at its required payment assessment.
use super::installation::Home;
use bank_domain::{model::PaymentId, proposals::BankIdempotencyKey, schema::*};
use bank_server::{BankAuthenticatedPrincipal, BankHomeRuntime};
use worth_query_bank_testkit_server::facade::{
    estate_operations::freeze_account::fixture::SPECIALIST, support::request_scope,
};
use worth_query_host::facade::application_entry::*;

pub(super) fn start_wait(
    home: &Home,
    principal: &BankAuthenticatedPrincipal,
    payment: PaymentId,
) -> (
    WorthQueryWorkflowInstanceDiscoveryIntent<BankSchema>,
    RequiredWorkflowAssessment,
    PublishedWorkflowInstanceRef,
) {
    let bank = BankHomeRuntime::attach(home.runtime()).unwrap();
    let scope = request_scope();
    let request = bank.request(principal, &scope);
    let authority = ApprovePayment {
        payment,
        approver: SPECIALIST,
    };
    let contract = home
        .retained_workflow_spec::<bank_domain::schema::ApprovedBusinessPaymentWorkflow>()
        .expect("Bank payment workflow registered")
        .bind_definition(bank_server::approved_business_payment_definition().unwrap())
        .unwrap();
    let publication = request
        .mutate(ApprovedBusinessPaymentAuthoringIntent {
            input: authority.clone(),
        })
        .without_source()
        .idempotency(&key("h10-definition"))
        .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)
        .unwrap()
        .execute();
    let definition = match publication {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("real workflow definition must publish: {other:?}"),
    };
    let prepared = request
        .mutate(ApprovedBusinessPaymentInstanceStartIntent {
            input: authority.clone(),
        })
        .without_source()
        .idempotency(&key("h10-instance"))
        .prepare_workflow_instance_start(
            home.workflow_vocabulary::<bank_domain::schema::ApprovedBusinessPaymentWorkflow>()
                .expect("Bank payment workflow registered"),
            definition,
        )
        .unwrap();
    let original = prepared.discovery_intent();
    let instance = match prepared.execute() {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("real payment instance must start: {other:?}"),
    };
    let proposed = request
        .mutate(ApprovedBusinessPaymentAuthoringIntent { input: authority })
        .without_source()
        .idempotency(&key("h10-proposal"))
        .prepare_workflow_proposal(
            home.workflow_vocabulary::<bank_domain::schema::ApprovedBusinessPaymentWorkflow>()
                .expect("Bank payment workflow registered"),
            instance.clone(),
        )
        .unwrap()
        .execute();
    assert!(matches!(proposed, WorkflowProposalOutcome::Published(_)));
    let required = wait(
        home,
        principal,
        payment,
        instance.clone(),
        "h10-wait-before-close",
    );
    assert_eq!(required.node_path(), "review/payment");
    (original, required, instance)
}

pub(super) fn wait(
    home: &Home,
    principal: &BankAuthenticatedPrincipal,
    payment: PaymentId,
    instance: PublishedWorkflowInstanceRef,
    command: &str,
) -> RequiredWorkflowAssessment {
    let bank = BankHomeRuntime::attach(home.runtime()).unwrap();
    let scope = request_scope();
    let progress = bank
        .request(principal, &scope)
        .on_branch(instance.branch())
        .mutate(ApprovedBusinessPaymentAdvanceIntent {
            input: ApprovePayment {
                payment,
                approver: SPECIALIST,
            },
        })
        .without_source()
        .idempotency(&key(command))
        .prepare_workflow_advance(
            home.workflow_vocabulary::<bank_domain::schema::ApprovedBusinessPaymentWorkflow>()
                .expect("Bank payment workflow registered"),
            instance,
        )
        .unwrap()
        .execute();
    match progress {
        WorkflowProgressOutcome::AwaitingAssessment(required) => required,
        other => panic!("the proposed payment must wait for real assessment evidence: {other:?}"),
    }
}
pub(super) fn resume_wait(
    home: &Home,
    principal: &BankAuthenticatedPrincipal,
    payment: PaymentId,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<BankSchema>,
    before: &RequiredWorkflowAssessment,
) -> PublishedWorkflowInstanceRef {
    use worth_query_host::facade::application_discovery::WorthQueryWorkflowInstanceRead;
    let bank = BankHomeRuntime::attach(home.runtime()).unwrap();
    let scope = request_scope();
    let request = bank.request(principal, &scope);
    let basis = request.retain_current_read().unwrap();
    let input = ApprovePayment {
        payment,
        approver: SPECIALIST,
    };
    let recovered = request.at(&basis).read_original_workflow::<ApprovedBusinessPaymentWorkflow,
        ApprovedBusinessPaymentRead, ApprovedBusinessPaymentRead>(home.workflow_vocabulary::<bank_domain::schema::ApprovedBusinessPaymentWorkflow>().expect("Bank payment workflow registered"), original,
        input.clone(), input).expect("fresh authenticated manager reads original waiting instance");
    let WorthQueryWorkflowInstanceRead::Live {
        instance,
        observation,
    } = recovered
    else {
        panic!("the same proposed payment workflow must remain live");
    };
    assert_eq!(observation.selected_commit(), basis.selected_commit());
    let after = wait(
        home,
        principal,
        payment,
        instance.clone(),
        "h10-wait-after-reopen",
    );
    assert_eq!(
        &after, before,
        "reopen preserves the exact required assessment occurrence and proposal"
    );
    instance
}
fn key(value: &str) -> BankIdempotencyKey {
    BankIdempotencyKey::new(value).unwrap()
}
