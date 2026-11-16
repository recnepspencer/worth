//! One actual workflow assessment original survives reopen and then observes a decision edit.
use super::installation::Home;
use bank_domain::{model::PaymentId, proposals::BankIdempotencyKey, schema::*};
use bank_server::{BankAssessedPaymentsApplication, BankAuthenticatedPrincipal, BankHomeRuntime};
use worth_query_bank_testkit_server::facade::{
    estate_operations::freeze_account::fixture::SPECIALIST, support::request_scope,
};
use worth_query_host::facade::{
    application_contribution::WorthQueryWorkflowAssessmentPosture, application_entry::*,
    home_program_transition::EntityId,
};

pub(super) type Original = WorthQueryOriginalWorkflowAssessment<
    BankSchema,
    ApprovedBusinessPaymentWorkflow,
    ApprovedPaymentAssessmentDemand,
>;
type Observer = WorthQueryWorkflowAssessmentOperation<
    BankSchema,
    ApprovedBusinessPaymentWorkflow,
    ApprovedPaymentAssessmentDemand,
    BankAssessedPaymentsApplication,
>;

struct AssessmentEvidence {
    source: [u8; 32],
    producer_contacts: usize,
    preserved_payment: EntityId,
    published_payment: Option<EntityId>,
}

pub(super) fn retain(
    home: &Home,
    principal: &BankAuthenticatedPrincipal,
    payment: PaymentId,
    instance: PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowAssessment,
) -> Original {
    let bank = BankHomeRuntime::attach(home.runtime()).unwrap();
    let scope = request_scope();
    let request = bank.request(principal, &scope);
    let mut owner = request
        .mutate(ApprovedBusinessPaymentAdvanceIntent {
            input: ApprovePayment {
                payment,
                approver: SPECIALIST,
            },
        })
        .without_source()
        .idempotency(&BankIdempotencyKey::new("h10-observe-original").unwrap())
        .prepare_workflow_advance(home.vocabulary(), instance)
        .unwrap()
        .into_assessment_demand_for(required, ApprovedPaymentAssessmentDemand::new(payment))
        .unwrap()
        .retain_operation(home.runtime())
        .unwrap();
    let _ = settle(
        &mut owner,
        &request,
        WorthQueryWorkflowAssessmentPosture::Passing,
    );
    owner
        .graceful_handoff(&request)
        .unwrap_or_else(|(_, cause)| panic!("retain actual assessment original: {cause:?}"))
}

pub(super) fn recover_and_advance(
    home: &Home,
    principal: &BankAuthenticatedPrincipal,
    payment: PaymentId,
    different_payment: PaymentId,
    instance: PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowAssessment,
    original: &Original,
) {
    let bank = BankHomeRuntime::attach(home.runtime()).unwrap();
    let scope = request_scope();
    let request = bank.request(principal, &scope);
    let mismatched = request
        .mutate(ApprovedBusinessPaymentAdvanceIntent {
            input: ApprovePayment {
                payment,
                approver: SPECIALIST,
            },
        })
        .without_source()
        .idempotency(&BankIdempotencyKey::new("h10-refuse-different-demand").unwrap())
        .prepare_workflow_advance(home.vocabulary(), instance.clone())
        .unwrap()
        .into_assessment_demand_for(
            required,
            ApprovedPaymentAssessmentDemand::new(different_payment),
        )
        .unwrap()
        .readmit_operation(home.runtime(), original);
    assert!(
        matches!(
            mismatched,
            Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch)
        ),
        "decoded DATA cannot substitute another demand for the original"
    );
    let mut owner = request
        .mutate(ApprovedBusinessPaymentAdvanceIntent {
            input: ApprovePayment {
                payment,
                approver: SPECIALIST,
            },
        })
        .without_source()
        .idempotency(&BankIdempotencyKey::new("h10-recover-observer").unwrap())
        .prepare_workflow_advance(home.vocabulary(), instance)
        .unwrap()
        .into_assessment_demand_for(required, ApprovedPaymentAssessmentDemand::new(payment))
        .unwrap()
        .readmit_operation(home.runtime(), original)
        .unwrap();
    let before = settle(
        &mut owner,
        &request,
        WorthQueryWorkflowAssessmentPosture::Passing,
    );
    // A real authenticated command changes the very PaymentStatus dependency
    // this recovered source owns. A new demand would not prove this transition.
    let (rejection, _) =
        super::deferred::retain(home, principal, payment, "h10-edit-recovered-source");
    let after = settle(
        &mut owner,
        &request,
        WorthQueryWorkflowAssessmentPosture::Failing,
    );
    assert_ne!(
        before.source, after.source,
        "same owner refreshes its actual changed source"
    );
    assert!(
        after.producer_contacts > before.producer_contacts,
        "the recovered original contacts its producer for the edit"
    );
    assert_eq!(
        before.preserved_payment, after.preserved_payment,
        "the actual exactly-one assessment output preserves the same payment"
    );
    assert_eq!(
        after.published_payment,
        Some(after.preserved_payment),
        "the dependency edit materializes the actual producer publication"
    );
    super::deferred::resume(home, principal, rejection.identity(), payment);
    drop(
        owner
            .close(&request)
            .unwrap_or_else(|(_, cause)| panic!("close observer: {cause:?}")),
    );
    super::deferred::assert_rejected(home, principal, payment);
}

fn settle(
    owner: &mut Observer,
    request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
    expected: WorthQueryWorkflowAssessmentPosture,
) -> AssessmentEvidence {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        assert!(
            std::time::Instant::now() < deadline,
            "actual assessment must settle"
        );
        match owner
            .advance(request)
            .expect("same original advances with fresh authority")
        {
            WorthQueryWorkflowAssessmentDemandProgress::Settled(settled) => {
                assert_eq!(settled.posture(), expected);
                assert_eq!(settled.required(), owner.required());
                let native = settled.settlement();
                let output = native
                    .outputs_of::<ApprovedPaymentAssessmentOutputs>()
                    .unwrap()
                    .entity::<ApprovedPaymentAssessmentOutput>()
                    .unwrap()
                    .entity_id();
                // Cold reuse carries typed original outputs without a live receipt.
                // The post-edit assertion requires actual fresh publication.
                let published_payment = native.application_commit_receipt().map(|receipt| {
                    receipt
                        .outputs_of::<ApprovedPaymentAssessmentOutputs>()
                        .unwrap()
                        .entity::<ApprovedPaymentAssessmentOutput>()
                        .unwrap()
                        .entity_id()
                });
                return AssessmentEvidence {
                    source: native
                        .observed_source()
                        .output_source_epoch()
                        .unwrap()
                        .runtime_idempotency_identity(),
                    producer_contacts: native.producer_contacts_in_this_demand(),
                    preserved_payment: output,
                    published_payment,
                };
            }
            WorthQueryWorkflowAssessmentDemandProgress::Pending
            | WorthQueryWorkflowAssessmentDemandProgress::Progressed => {}
        }
    }
}
