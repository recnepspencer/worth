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
use bank_domain::schema::{ApprovePayment, InitiateBusinessPayment, RejectPayment};
use bank_server::{
    mutations, queries, BankApplicationP1, BankMutationControls, BankPendingPaymentContinuation,
    BankReadControls,
};

use fixture::{ordinary_read_world, principal_id, APPROVER, OWNER, RECIPIENT};
use support::request_scope;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
    WorthQueryBranchAdoptionPublicationOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionDenialKind;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome,
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

    let initiation_scope = request_scope();
    let request = fixture.world.runtime.request(&owner, &initiation_scope);
    let key = BankIdempotencyKey::new("initiate-workflow").unwrap();
    let first = request
        .mutate(input.clone())
        .idempotency(&key)
        .execute_in_program(fixture.world.runtime.application_program())
        .expect("the program-owned initiation should execute");
    let WorthQueryApplicationMutationOutcome::Committed { result, .. } = first else {
        panic!("the first initiation must commit: {first:?}");
    };
    let pending = BankPendingPaymentContinuation::from_payment_id(result.payment);
    let recovered = request
        .mutate(input)
        .idempotency(&key)
        .execute_in_program(fixture.world.runtime.application_program())
        .expect("the identical initiation retry should execute");
    assert!(matches!(
        recovered,
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(_)
    ));

    let approval_scope = request_scope();
    let approval_request = fixture.world.runtime.request(&approver, &approval_scope);
    let approval_key = BankIdempotencyKey::new("approve-workflow").unwrap();
    let approval = approval_request
        .mutate(ApprovePayment {
            payment: pending.payment_id(),
            approver: principal_id(APPROVER),
        })
        .idempotency(&approval_key)
        .execute_in_program(fixture.world.runtime.application_program())
        .expect("the program-owned approval should execute");
    assert!(matches!(
        approval,
        WorthQueryApplicationMutationOutcome::Committed { .. }
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
        unauthorized,
        Err(ref denial)
            if denial.kind()
                == worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind::Authorization
    ));

    let approval = fixture
        .world
        .runtime
        .mutate(pending.approve())
        .as_principal(&approver)
        .controls(controls("approver-can-approve"))
        .execute();
    assert!(matches!(
        approval,
        Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
    ));

    let stale_decision = fixture
        .world
        .runtime
        .mutate(pending.reject())
        .as_principal(&approver)
        .controls(controls("cannot-decide-twice"))
        .execute();
    assert!(matches!(
        stale_decision,
        Ok(WorthQueryApplicationMutationOutcome::DomainDenied(
            BankProposalDenial::PaymentAlreadyDecided(payment)
        )) if payment == pending.payment_id()
    ));
}

#[test]
fn a_fresh_authorized_rejector_can_reject_the_read_derived_continuation() {
    let fixture = ordinary_read_world("reject-continuation", 0);
    let approver = fixture.authenticate(APPROVER);
    let pending = pending_continuation(&fixture, &approver);

    let rejection_scope = request_scope();
    let rejection_request = fixture.world.runtime.request(&approver, &rejection_scope);
    let rejection_key = BankIdempotencyKey::new("reject-pending").unwrap();
    let rejection = rejection_request
        .mutate(RejectPayment {
            payment: pending.payment_id(),
            rejecting_principal: principal_id(APPROVER),
        })
        .idempotency(&rejection_key)
        .execute_in_program(fixture.world.runtime.application_program())
        .expect("the program-owned rejection should execute");

    assert!(
        matches!(
            rejection,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "program-owned rejection must commit: {rejection:?}"
    );
}

#[test]
fn pending_approval_crosses_adoption_only_through_fresh_p1_admission() {
    let fixture = ordinary_read_world("adopted-payment-continuation", 0);
    let approver = fixture.authenticate(APPROVER);
    let pending = pending_continuation(&fixture, &approver);
    let application = fixture.world.runtime.application_program();
    let branch = application.current_world();
    let target_owner = application
        .supported_program::<BankApplicationP1>()
        .expect("Bank P1 is rostered beside P0");
    let target = target_owner.owned_revision().clone();
    let adoption_scope = request_scope();
    let programs = fixture
        .world
        .runtime
        .request(&approver, &adoption_scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    let prepared = programs
        .adopt(&target)
        .requirements(&requirements)
        .prepare(128)
        .expect("Bank P1 adoption prepares");
    assert!(matches!(
        prepared.publish(),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));

    let approval_scope = request_scope();
    let approval_request = fixture.world.runtime.request(&approver, &approval_scope);
    let stale_approval_key = BankIdempotencyKey::new("approve-after-adoption-source").unwrap();
    let stale_approval = approval_request
        .mutate(ApprovePayment {
            payment: pending.payment_id(),
            approver: principal_id(APPROVER),
        })
        .idempotency(&stale_approval_key)
        .execute_in_program(application)
        .expect("the stale source request reaches occurrence gating");
    assert!(matches!(
        stale_approval,
        WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationCommitOutcome::Denied(ref denial)
        ) if denial.kind() == WorthQueryApplicationCommitDenialKind::ProgramNotActiveOnOccurrence
    ));

    let approval_key = BankIdempotencyKey::new("approve-after-adoption").unwrap();
    let approval = approval_request
        .mutate(ApprovePayment {
            payment: pending.payment_id(),
            approver: principal_id(APPROVER),
        })
        .idempotency(&approval_key)
        .execute_in_program(&target_owner)
        .expect("the carried payment identity receives fresh P1 admission");
    assert!(matches!(
        approval,
        WorthQueryApplicationMutationOutcome::Committed { .. }
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

    assert!(matches!(
        outcome.execution(),
        Err(WorthQueryApplicationRequestMutationDenial::PrincipalResolution(denial))
            if denial.kind() == WorthQueryPrincipalResolutionDenialKind::Cancelled
    ));
    assert_eq!(outcome.continuation(), None);
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
