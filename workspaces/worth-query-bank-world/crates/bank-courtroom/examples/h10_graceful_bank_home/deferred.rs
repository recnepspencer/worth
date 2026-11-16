//! One native fixed action carries the same deferred output original across Home close.
use super::installation::Home;
use bank_domain::{model::PaymentId, proposals::BankIdempotencyKey, schema::*};
use bank_server::{
    BankAssessedPaymentsApplication, BankAuthenticatedPrincipal, BankHomeRuntime,
    RejectedPaymentAssessmentAction, RejectedPaymentAssessmentRoot,
};
use worth_query_bank_testkit_server::facade::{
    estate_operations::freeze_account::fixture::SPECIALIST, support::request_scope,
};
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::runtime::CompositeComponentChangePosture;

type Operation = WorthQueryFixedOperation<
    BankSchema,
    RejectPayment,
    BankAssessedPaymentsApplication,
    RejectedPaymentAssessmentRoot,
    RejectedPaymentAssessmentAction,
>;
pub(super) fn retain(
    home: &Home,
    principal: &BankAuthenticatedPrincipal,
    payment: PaymentId,
    command: &str,
) -> (
    WorthQueryOriginalOperation,
    [CompositeComponentChangePosture; 2],
) {
    let bank = BankHomeRuntime::attach(home.runtime()).unwrap();
    let scope = request_scope();
    let request = bank.request(principal, &scope);
    let mut operation = request.mutate(RejectPayment { payment, rejecting_principal: SPECIALIST })
        .without_source().idempotency_owned(BankIdempotencyKey::new(command).unwrap())
        .unwrap_or_else(|(_, _, cause)| panic!("capture real rejection: {cause:?}"))
        .retain_durable_operation::<BankAssessedPaymentsApplication, RejectedPaymentAssessmentAction>(
            &request, home.runtime())
        .unwrap_or_else(|(_, cause)| panic!("retain rejection original: {cause:?}"));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    let mut components = None;
    loop {
        assert!(
            std::time::Instant::now() < deadline,
            "native deferred owner must progress"
        );
        match operation.advance(&request) {
            WorthQueryOperationAdvance::Progress {
                operation: next,
                progress,
            } => {
                operation = next;
                if matches!(progress, WorthQueryOperationProgress::SourcePerformed) {
                    let publication = operation
                        .prepared_source_receipt()
                        .expect("performed source retains its actual receipt")
                        .committed_product_publication();
                    components = Some([
                        publication.relational_posture(),
                        publication.signal_posture(),
                    ]);
                    assert!(publication.signal_publication().is_some());
                    assert!(publication.conditional_definition_generation().is_some());
                }
                if matches!(progress, WorthQueryOperationProgress::OutputsStarted) {
                    break;
                }
            }
            WorthQueryOperationAdvance::Settled(_) => {
                panic!("output observation was not started separately")
            }
            WorthQueryOperationAdvance::Refused { denial, .. }
            | WorthQueryOperationAdvance::Terminal { denial, .. } => {
                panic!("reject must start real outputs: {denial:?}")
            }
        }
    }
    let (original, cleanup) = operation
        .handoff_for_reopen(&request)
        .unwrap_or_else(|(_, cause)| panic!("graceful fixed handoff: {cause:?}"));
    drop(cleanup);
    (
        original,
        components.expect("the original has a performed source publication"),
    )
}

pub(super) fn resume(
    home: &Home,
    principal: &BankAuthenticatedPrincipal,
    original: worth_foundational::facade::ProgramSupportObligationIdentity,
    payment: PaymentId,
) {
    let bank = BankHomeRuntime::attach(home.runtime()).unwrap();
    let scope = request_scope();
    let request = bank.request(principal, &scope);
    let description = WorthQueryOriginalOperation::readmit_native::<
        BankSchema,
        RejectPayment,
        BankAssessedPaymentsApplication,
        RejectedPaymentAssessmentAction,
    >(
        original,
        &request,
        home.runtime(),
        worth_foundational::facade::ValueDecodeBounds::unrestricted(),
    )
    .unwrap();
    let operation: Operation = description.retain_durable_operation::<
        BankAssessedPaymentsApplication, RejectedPaymentAssessmentAction>(&request, home.runtime())
        .unwrap_or_else(|(_, cause)| panic!("readmit the same original: {cause:?}"));
    settle(operation, &request);
    assert_rejected(home, principal, payment);
}

fn settle(
    mut operation: Operation,
    request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        assert!(
            std::time::Instant::now() < deadline,
            "recovered deferred owner must settle"
        );
        match operation.advance(request) {
            WorthQueryOperationAdvance::Progress {
                operation: next, ..
            } => operation = next,
            WorthQueryOperationAdvance::Settled(settled) => {
                // Close discharges this original only after its actual native output settlement.
                let closed = settled
                    .close(request)
                    .unwrap_or_else(|(_, cause)| panic!("settled original close: {cause:?}"));
                drop(closed);
                break;
            }
            WorthQueryOperationAdvance::Refused { denial, .. }
            | WorthQueryOperationAdvance::Terminal { denial, .. } => {
                panic!("fresh owner must continue original outputs: {denial:?}")
            }
        }
    }
}

pub(super) fn assert_rejected(
    home: &Home,
    principal: &BankAuthenticatedPrincipal,
    payment: PaymentId,
) {
    let bank = BankHomeRuntime::attach(home.runtime()).unwrap();
    let scope = request_scope();
    let result = bank
        .request(principal, &scope)
        .query(bank_domain::queries::payment(payment))
        .execute()
        .expect("independent admitted payment read");
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].id(), payment);
    assert_eq!(result.rows()[0].status(), PaymentStatus::Rejected);
    assert_eq!(result.rows()[0].deciding_principal(), Some(SPECIALIST));
}
