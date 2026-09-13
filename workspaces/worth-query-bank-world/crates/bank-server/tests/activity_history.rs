#[allow(
    dead_code,
    reason = "the shared read fixture includes scenarios used by sibling consumer courtrooms"
)]
#[path = "ordinary_reads/fixture.rs"]
mod fixture;
#[path = "activity_history/historical.rs"]
mod historical;
mod support;

use std::num::NonZeroUsize;

use bank_domain::model::Money;
use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::{Deposit, RevokeAccountAuthorization};
use bank_server::{
    mutations, queries, BankApplicationQueryAdmissionDenialKind, BankApplicationQueryDenial,
    BankMutationControls, BankMutationStatus, BankReadControls,
};
use fixture::{ordinary_read_world, OWNER, TELLER, VIEWER};
use support::request_scope;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::primary_graph::WorthQueryApplicationQueryResumeControls;

#[test]
fn activity_pages_keep_one_exact_basis_across_a_new_commit() {
    let fixture = ordinary_read_world("activity-pages", 0);
    let owner = fixture.authenticate(OWNER);
    let teller = fixture.authenticate(TELLER);
    let first_request = request_scope();
    let first = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .page(page_controls(&first_request, 1))
        .expect("first page must execute");
    assert_eq!(first.rows()[0].entries().len(), 1);
    let first_journal = first.rows()[0].entries()[0].journal();
    let (first_publication, continuation) = first.into_parts();
    let continuation = continuation.expect("first page must continue");
    assert!(first_publication
        .receipt()
        .inspect()
        .terminal_resources_released());

    let mutation = fixture
        .world
        .runtime
        .mutate(bank_server::mutations::deposit(Deposit {
            institution: fixture.institution,
            account: fixture.personal_account,
            amount: Money::from_minor(1).unwrap(),
        }))
        .as_principal(&teller)
        .controls(BankMutationControls::new(
            request_scope(),
            BankIdempotencyKey::new("activity-page-version-change").unwrap(),
        ))
        .execute();
    assert!(matches!(
        mutation.status(),
        BankMutationStatus::Committed(_)
    ));

    let next_request = request_scope();
    let second = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .resume(continuation, resume_controls(&next_request, 1))
        .expect("the retained version must remain readable");
    assert_eq!(second.rows()[0].entries().len(), 1);
    assert_ne!(second.rows()[0].entries()[0].journal(), first_journal);
    assert!(
        second.continuation().is_none(),
        "the post-page commit must not appear in the original two-row basis"
    );
    assert!(second.receipt().inspect().terminal_resources_released());
}

#[test]
fn activity_continuation_cannot_cross_account_scope() {
    let fixture = ordinary_read_world("activity-continuation-scope", 0);
    let owner = fixture.authenticate(OWNER);
    let first_request = request_scope();
    let first = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .page(page_controls(&first_request, 1))
        .expect("first page must execute");
    let (_, continuation) = first.into_parts();
    let continuation = continuation.expect("first page must continue");
    let resume_request = request_scope();
    let crossed = fixture
        .world
        .runtime
        .account_activity(fixture.business_account)
        .as_principal(&owner)
        .resume(continuation, resume_controls(&resume_request, 1));
    assert!(matches!(
        crossed,
        Err(BankApplicationQueryDenial::Admission(denial))
            if denial.kind()
                == BankApplicationQueryAdmissionDenialKind::ContinuationScopeMismatch
    ));
}

#[test]
fn activity_continuation_cannot_cross_runtime_authority() {
    let source = ordinary_read_world("activity-continuation-source-runtime", 0);
    let target = ordinary_read_world("activity-continuation-target-runtime", 0);
    let source_owner = source.authenticate(OWNER);
    let target_owner = target.authenticate(OWNER);
    let first_request = request_scope();
    let first = source
        .world
        .runtime
        .account_activity(source.personal_account)
        .as_principal(&source_owner)
        .page(page_controls(&first_request, 1))
        .expect("source runtime must issue a continuation");
    let (_, continuation) = first.into_parts();
    let continuation = continuation.expect("first page must continue");

    let resume_request = request_scope();
    let crossed = target
        .world
        .runtime
        .account_activity(target.personal_account)
        .as_principal(&target_owner)
        .resume(continuation, resume_controls(&resume_request, 1));
    assert!(matches!(
        crossed,
        Err(BankApplicationQueryDenial::Admission(denial))
            if denial.kind()
                == BankApplicationQueryAdmissionDenialKind::ForeignContinuation
    ));
}

#[test]
fn oversized_page_denies_before_continuation_plan_authority() {
    let fixture = ordinary_read_world("activity-continuation-width", 0);
    let owner = fixture.authenticate(OWNER);
    let request = request_scope();
    let denied = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .page(page_controls(&request, 257));
    assert!(matches!(
        denied,
        Err(BankApplicationQueryDenial::Admission(denial))
            if denial.kind()
                == BankApplicationQueryAdmissionDenialKind::ContinuationPageWidthUnsupported
    ));
}

#[test]
fn revocation_before_resume_denies_fresh_page_admission() {
    let fixture = ordinary_read_world("activity-continuation-revocation", 0);
    let owner = fixture.authenticate(OWNER);
    let viewer = fixture.authenticate(VIEWER);
    let authorization = authorized_user_id(&fixture, &owner, VIEWER);
    let first_request = request_scope();
    let first = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&viewer)
        .page(page_controls(&first_request, 1))
        .expect("authorized viewer must receive the first page");
    let (_, continuation) = first.into_parts();
    let continuation = continuation.expect("first page must continue");

    let revoked = fixture
        .world
        .runtime
        .mutate(mutations::revoke_account_access(
            RevokeAccountAuthorization {
                account: fixture.personal_account,
                authorization,
            },
        ))
        .as_principal(&owner)
        .controls(BankMutationControls::new(
            request_scope(),
            BankIdempotencyKey::new("revoke-continuation-viewer").unwrap(),
        ))
        .execute();
    assert!(matches!(revoked.status(), BankMutationStatus::Committed(_)));

    let resume_request = request_scope();
    let resumed = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&viewer)
        .resume(continuation, resume_controls(&resume_request, 1));
    assert!(matches!(
        resumed,
        Err(BankApplicationQueryDenial::Admission(denial))
            if matches!(
                denial.kind(),
                BankApplicationQueryAdmissionDenialKind::Authorization(_)
            )
    ));
}

#[test]
fn paged_and_one_shot_activity_have_identical_result_meaning() {
    let fixture = ordinary_read_world("activity-page-parity", 0);
    let owner = fixture.authenticate(OWNER);
    let one_shot_request = request_scope();
    let one_shot = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .execute(BankReadControls::current(one_shot_request, 64, 8_192).unwrap())
        .expect("one-shot activity must execute");
    let expected = one_shot.rows()[0].entries().to_vec();

    let first_request = request_scope();
    let first = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .page(page_controls(&first_request, 1))
        .expect("first page must execute");
    let (published, mut continuation) = first.into_parts();
    let mut observed = published.rows()[0].entries().to_vec();
    while let Some(next) = continuation {
        let request = request_scope();
        let page = fixture
            .world
            .runtime
            .account_activity(fixture.personal_account)
            .as_principal(&owner)
            .resume(next, resume_controls(&request, 1))
            .expect("every continuation page must execute");
        let (published, next) = page.into_parts();
        observed.extend_from_slice(published.rows()[0].entries());
        continuation = next;
    }
    assert_eq!(observed, expected);
}

#[test]
fn activity_order_follows_committed_account_sequence_not_derived_identity() {
    let fixture = ordinary_read_world("activity-chronology", 0);
    let owner = fixture.authenticate(OWNER);
    let teller = fixture.authenticate(TELLER);
    const COMMIT_COUNT: usize = 12;
    for ordinal in 0..COMMIT_COUNT {
        let key = format!("activity-chronology-{ordinal}");
        let amount = 100 + ordinal as i64;
        let outcome = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::deposit(Deposit {
                institution: fixture.institution,
                account: fixture.personal_account,
                amount: Money::from_minor(amount).unwrap(),
            }))
            .as_principal(&teller)
            .controls(BankMutationControls::new(
                request_scope(),
                BankIdempotencyKey::new(key).unwrap(),
            ))
            .execute();
        assert!(matches!(outcome.status(), BankMutationStatus::Committed(_)));
    }

    let request = request_scope();
    let history = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .page(page_controls(&request, 32))
        .expect("activity page must execute");
    let entries = history.rows()[0].entries();
    let recent = &entries[entries.len() - COMMIT_COUNT..];
    assert!(recent
        .windows(2)
        .all(|pair| pair[0].account_sequence() < pair[1].account_sequence()));
    for (ordinal, entry) in recent.iter().enumerate() {
        assert_eq!(entry.amount().minor_units(), 100 + ordinal as i64);
    }
    assert!(
        recent
            .windows(2)
            .any(|pair| pair[0].journal() > pair[1].journal()),
        "fixed commits must include an identity inversion so this proves sequence ordering"
    );
    assert!(history.receipt().inspect().terminal_resources_released());
}

fn page_controls<'a>(request: &'a WorthQueryRequestScope, page_width: usize) -> BankReadControls {
    BankReadControls::current(request.clone(), page_width, 4_096).unwrap()
}

fn resume_controls<'a>(
    request: &'a WorthQueryRequestScope,
    page_width: usize,
) -> WorthQueryApplicationQueryResumeControls<'a> {
    WorthQueryApplicationQueryResumeControls::new(
        NonZeroUsize::new(page_width).unwrap(),
        NonZeroUsize::new(4_096).unwrap(),
        request,
    )
}

fn authorized_user_id(
    fixture: &fixture::OrdinaryReadFixture,
    owner: &bank_server::BankAuthenticatedPrincipal,
    principal: usize,
) -> bank_domain::model::AccountAuthorizationId {
    let users = fixture
        .world
        .runtime
        .query(queries::account_authorized_users(fixture.personal_account))
        .as_principal(owner)
        .controls(BankReadControls::current(request_scope(), 16, 10_000).unwrap())
        .execute()
        .expect("owner should read authorized users");
    let [users] = users.rows() else {
        panic!("authorized users query must return one account row")
    };
    users
        .users()
        .iter()
        .find(|user| user.principal() == fixture::principal_id(principal))
        .expect("fixture authorization should exist")
        .authorization()
}
