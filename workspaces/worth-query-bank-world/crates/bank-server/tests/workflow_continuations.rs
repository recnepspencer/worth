#[allow(
    dead_code,
    reason = "the shared read fixture has discovery-only helpers exercised by its owning test binary"
)]
#[path = "ordinary_reads/fixture.rs"]
mod fixture;
mod support;

use std::time::{Duration, Instant};

use bank_domain::model::{BusinessId, Money};
use bank_domain::proposals::{BankIdempotencyKey, BankProposalDenial};
use bank_domain::schema::InitiateBusinessPayment;
use bank_server::{
    mutations, queries, BankMutationControls, BankMutationExplanation,
    BankMutationExplanationStage, BankMutationStatus, BankPendingPaymentContinuation,
    BankReadControls,
};

use fixture::{ordinary_read_world, principal_id, APPROVER, OWNER, RECIPIENT};
use support::request_scope;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[test]
fn initiation_recovers_one_continuation_and_a_fresh_approver_commits() {
    let fixture = ordinary_read_world("workflow-continuation", 0);
    let owner = fixture.authenticate(OWNER);
    let approver = fixture.authenticate(APPROVER);
    let input = InitiateBusinessPayment {
        business: BusinessId::new(1).unwrap(),
        from: fixture.business_account,
        recipient: principal_id(RECIPIENT),
        amount: Money::from_minor(300).unwrap(),
    };

    let first = execute_initiation(&fixture, &owner, input.clone(), "initiate-workflow");
    assert!(matches!(first.status(), BankMutationStatus::Committed(_)));
    assert!(matches!(
        first.explanation(),
        BankMutationExplanation::Committed {
            recovered: false,
            ..
        }
    ));
    let pending = first.continuation().expect("commit must mint continuation");
    let recovered = execute_initiation(&fixture, &owner, input, "initiate-workflow");
    assert!(matches!(
        recovered.status(),
        BankMutationStatus::AlreadyCommitted(_)
    ));
    assert!(matches!(
        recovered.explanation(),
        BankMutationExplanation::Committed {
            recovered: true,
            ..
        }
    ));
    assert_eq!(recovered.continuation(), Some(pending));

    let approval = fixture
        .world
        .runtime
        .mutate(pending.approve())
        .as_principal(&approver)
        .controls(controls("approve-workflow"))
        .execute();
    assert!(matches!(
        approval.status(),
        BankMutationStatus::Committed(_)
    ));
}

#[test]
fn pending_read_mints_no_authority_and_decided_continuation_cannot_advance_again() {
    let fixture = ordinary_read_world("read-continuation", 0);
    let owner = fixture.authenticate(OWNER);
    let approver = fixture.authenticate(APPROVER);
    let pending = pending_continuation(&fixture, &approver);
    let copied = BankPendingPaymentContinuation::from_payment_id(pending.payment_id());

    let unauthorized = fixture
        .world
        .runtime
        .mutate(copied.approve())
        .as_principal(&owner)
        .controls(controls("owner-cannot-approve"))
        .execute();
    assert!(matches!(
        unauthorized.status(),
        BankMutationStatus::Denied(_)
    ));
    assert!(matches!(
        unauthorized.explanation(),
        BankMutationExplanation::Denied {
            stage: BankMutationExplanationStage::Admission,
            ..
        }
    ));

    let approval = fixture
        .world
        .runtime
        .mutate(pending.approve())
        .as_principal(&approver)
        .controls(controls("approver-can-approve"))
        .execute();
    assert!(matches!(
        approval.status(),
        BankMutationStatus::Committed(_)
    ));

    let stale_decision = fixture
        .world
        .runtime
        .mutate(pending.reject())
        .as_principal(&approver)
        .controls(controls("cannot-decide-twice"))
        .execute();
    assert!(matches!(
        stale_decision.status(),
        BankMutationStatus::InvariantViolated(
            BankProposalDenial::PaymentAlreadyDecided(payment)
        ) if *payment == pending.payment_id()
    ));
    assert!(matches!(
        stale_decision.explanation(),
        BankMutationExplanation::InvariantViolated(
            BankProposalDenial::PaymentAlreadyDecided(payment)
        ) if *payment == pending.payment_id()
    ));
}

#[test]
fn a_fresh_authorized_rejector_can_reject_the_read_derived_continuation() {
    let fixture = ordinary_read_world("reject-continuation", 0);
    let approver = fixture.authenticate(APPROVER);
    let pending = pending_continuation(&fixture, &approver);

    let rejection = fixture
        .world
        .runtime
        .mutate(pending.reject())
        .as_principal(&approver)
        .controls(controls("reject-pending"))
        .execute();

    assert!(matches!(
        rejection.status(),
        BankMutationStatus::Committed(_)
    ));
}

#[test]
fn cancelled_initiation_cannot_mint_a_continuation() {
    let fixture = ordinary_read_world("cancelled-continuation", 0);
    let owner = fixture.authenticate(OWNER);
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let input = InitiateBusinessPayment {
        business: BusinessId::new(1).unwrap(),
        from: fixture.business_account,
        recipient: principal_id(RECIPIENT),
        amount: Money::from_minor(300).unwrap(),
    };
    let outcome = fixture
        .world
        .runtime
        .mutate(mutations::initiate_business_payment(input))
        .as_principal(&owner)
        .controls(BankMutationControls::new(
            WorthQueryRequestScope::new(
                Instant::now() + Duration::from_secs(60),
                cancellation.token(),
            ),
            BankIdempotencyKey::new("cancelled-initiation").unwrap(),
        ))
        .execute();

    assert!(matches!(outcome.status(), BankMutationStatus::Cancelled));
    assert_eq!(outcome.continuation(), None);
}

fn execute_initiation(
    fixture: &fixture::OrdinaryReadFixture,
    principal: &bank_server::BankAuthenticatedPrincipal,
    input: InitiateBusinessPayment,
    idempotency: &str,
) -> bank_server::BankPaymentInitiationOutcome {
    fixture
        .world
        .runtime
        .mutate(mutations::initiate_business_payment(input))
        .as_principal(principal)
        .controls(controls(idempotency))
        .execute()
}

fn pending_continuation(
    fixture: &fixture::OrdinaryReadFixture,
    principal: &bank_server::BankAuthenticatedPrincipal,
) -> BankPendingPaymentContinuation {
    let result = fixture
        .world
        .runtime
        .query(queries::pending_payments())
        .as_principal(principal)
        .controls(BankReadControls::current(request_scope(), 16, 10_000).unwrap())
        .execute()
        .expect("pending payments must be readable");
    let summary = result
        .rows()
        .iter()
        .find(|payment| payment.id() == fixture.payment)
        .cloned()
        .expect("fixture pending payment must be present");
    BankPendingPaymentContinuation::from_summary(summary)
        .expect("an approval-required payment must yield a continuation")
}

fn controls(idempotency: &str) -> BankMutationControls {
    BankMutationControls::new(
        request_scope(),
        BankIdempotencyKey::new(idempotency).unwrap(),
    )
}
