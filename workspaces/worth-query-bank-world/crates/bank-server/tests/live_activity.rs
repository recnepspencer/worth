#[path = "live_activity/consumer_scale.rs"]
mod consumer_scale;
#[allow(
    dead_code,
    reason = "the shared read fixture includes scenarios used by sibling consumer courtrooms"
)]
#[path = "ordinary_reads/fixture.rs"]
mod fixture;
#[path = "live_activity/support.rs"]
mod live_activity_support;
mod support;

use std::time::{Duration, Instant};

use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::RevokeAccountAuthorization;
use bank_server::{
    mutations, BankAccountActivityLiveOutcome, BankApplicationLiveCloseOutcome,
    BankApplicationQueryDenial, BankMutationControls,
};

use fixture::{ordinary_read_world, OWNER, TELLER, VIEWER};
use live_activity_support::{
    activity_count, authorized_user_id, commit_authorization_toggle, commit_deposit, live_controls,
    live_controls_for,
};
use support::request_scope;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationLiveControls;

#[test]
fn live_activity_delivers_only_matching_commits_as_fresh_reads() {
    let fixture = ordinary_read_world("live-activity", 0);
    let owner = fixture.authenticate(OWNER);
    let teller = fixture.authenticate(TELLER);
    let mut live = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .subscribe(live_controls())
        .expect("authorized activity lease should open");
    assert!(matches!(
        live.poll(&owner, &request_scope()),
        BankAccountActivityLiveOutcome::Pending
    ));

    commit_deposit(
        &fixture,
        &teller,
        fixture.recipient_account,
        "unrelated-live-deposit",
    );
    assert!(matches!(
        live.poll(&owner, &request_scope()),
        BankAccountActivityLiveOutcome::Pending
    ));

    let before = activity_count(&fixture, &owner);
    commit_deposit(
        &fixture,
        &teller,
        fixture.personal_account,
        "matching-live-deposit",
    );
    let outcome = live.poll(&owner, &request_scope());
    let BankAccountActivityLiveOutcome::Delivered(update) = outcome else {
        panic!("matching commit must deliver a fresh account projection");
    };
    let activity = update.result();
    assert_eq!(
        activity.entries()[0].account_sequence().get(),
        u64::try_from(before).unwrap() + 1
    );
    assert_eq!(activity.account(), fixture.personal_account);
    assert_eq!(activity.entries().len(), 1);
    assert_eq!(update.receipt().inspect().result_count(), 1);
    assert!(update.receipt().inspect().terminal_resources_released());
    let BankApplicationLiveCloseOutcome::Completed = live.close() else {
        panic!("live close must return its read-only terminal");
    };
}

#[test]
fn permission_revocation_closes_before_another_payload_is_delivered() {
    let fixture = ordinary_read_world("live-revocation", 0);
    let owner = fixture.authenticate(OWNER);
    let teller = fixture.authenticate(TELLER);
    let viewer = fixture.authenticate(VIEWER);
    let authorization = authorized_user_id(&fixture, &owner, VIEWER);
    let mut live = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&viewer)
        .subscribe(live_controls())
        .expect("viewer should open activity lease");

    let revoked_scope = crate::support::request_scope();
    let revoked_request = fixture.world.runtime.request(&owner, &revoked_scope);
    let operation = fixture
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
            BankIdempotencyKey::new("revoke-live-viewer").unwrap(),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("declared source action: {cause:?}"));
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
        revoked,
    ) = operation.advance(&revoked_request)
    else {
        panic!("fresh source completion")
    };
    assert!(matches!(
        revoked.evidence(),
        worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
    ));
    revoked
        .close(&revoked_request)
        .unwrap_or_else(|(_, cause)| panic!("lawful source close: {cause:?}"));
    commit_deposit(
        &fixture,
        &teller,
        fixture.personal_account,
        "revoked-viewer-live-deposit",
    );
    assert!(matches!(
        live.poll(&viewer, &request_scope()),
        BankAccountActivityLiveOutcome::AuthorizationDenied(_)
    ));
    assert!(matches!(
        live.poll(&viewer, &request_scope()),
        BankAccountActivityLiveOutcome::Closed
    ));
    assert_eq!(live.buffered_cause_count(), 0);
}

#[test]
fn cancellation_and_deadline_are_distinct_live_terminals() {
    let fixture = ordinary_read_world("live-interruption", 0);
    let owner = fixture.authenticate(OWNER);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    let mut cancelled = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .subscribe(live_controls_for(request, 16))
        .expect("live lease should open before cancellation");
    cancellation.cancel();
    let cancellation_request = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    let cancelled_outcome = cancelled.poll(&owner, &cancellation_request);
    assert!(
        matches!(cancelled_outcome, BankAccountActivityLiveOutcome::Cancelled),
        "unexpected cancellation outcome"
    );
    assert!(matches!(
        cancelled.poll(&owner, &cancellation_request),
        BankAccountActivityLiveOutcome::Closed
    ));

    let deadline = Instant::now() + Duration::from_secs(1);
    let deadline_source = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(Some(deadline), deadline_source.token());
    let mut expired = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .subscribe(live_controls_for(request, 16))
        .expect("live lease should open before its deadline");
    while Instant::now() < deadline {
        std::thread::yield_now();
    }
    let expired_request = WorthQueryRequestScope::new(Some(deadline), deadline_source.token());
    let expired_outcome = expired.poll(&owner, &expired_request);
    assert!(
        matches!(
            expired_outcome,
            BankAccountActivityLiveOutcome::DeadlineExceeded
        ),
        "unexpected deadline outcome"
    );
    assert!(matches!(
        expired.poll(&owner, &expired_request),
        BankAccountActivityLiveOutcome::Closed
    ));
}

/// Live declarations carry no resource contract (capability-index: "Request
/// Live controls retain channel buffer capacity"), so a caller buffer wider
/// than the removed 64-cause installed queue ceiling opens and governs delivery.
#[test]
fn caller_live_buffer_wider_than_the_removed_installed_ceiling_governs_the_queue() {
    let fixture = ordinary_read_world("live-buffer-ceiling", 0);
    let owner = fixture.authenticate(OWNER);
    let teller = fixture.authenticate(TELLER);
    let mut live = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .subscribe(live_controls_for(request_scope(), 65))
        .expect("the caller's 65-cause buffer opens without an installed queue ceiling");
    for ordinal in 0..3 {
        commit_deposit(
            &fixture,
            &teller,
            fixture.personal_account,
            &format!("wide-buffer-live-deposit-{ordinal}"),
        );
    }
    for remaining in (0..3).rev() {
        let BankAccountActivityLiveOutcome::Delivered(delivered) =
            live.poll(&owner, &request_scope())
        else {
            panic!("every buffered matching cause must deliver")
        };
        assert_eq!(delivered.result().entries().len(), 1);
        assert_eq!(live.buffered_cause_count(), remaining);
    }
}

/// Live controls carry no work field: the installed execution policy's one
/// work ceiling bounds the subscription, and the same caller controls open
/// only once that installed ceiling admits them.
#[test]
fn caller_cannot_enlarge_the_installed_live_work_ceiling() {
    use worth_query_host::facade::application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
    };
    use worth_query_host::facade::primary_graph::bound_advancement_requests_on_this_thread_for_test as bound;
    struct Restore(Option<worth_foundational::ExecutionBudget>);
    impl Drop for Restore {
        fn drop(&mut self) {
            bound(self.0);
        }
    }
    let fixture = ordinary_read_world("live-work-ceiling", 0);
    let owner = fixture.authenticate(OWNER);
    let subscribe = || {
        fixture
            .world
            .runtime
            .account_activity(fixture.personal_account)
            .as_principal(&owner)
            .subscribe(WorthQueryApplicationLiveControls::bounded(request_scope(), 16, 8).unwrap())
    };
    let restore = Restore(bound(Some(worth_foundational::ExecutionBudget::new(
        std::num::NonZeroUsize::MIN,
        Some(64 * 1_024 * 1_024),
        Some(0),
    ))));
    let denial = match subscribe() {
        Err(denial) => denial,
        Ok(_) => panic!("caller controls cannot widen an exhausted installed work ceiling"),
    };
    assert!(matches!(
        denial,
        BankApplicationQueryDenial::ExecutionRequest(Denial::Resource(Resource::WorkExhausted))
    ));
    drop(restore);
    let live = subscribe().expect("the installed policy admits the same caller controls");
    let BankApplicationLiveCloseOutcome::Completed = live.close() else {
        panic!("the admitted subscription closes cleanly")
    };
}

#[test]
fn admitted_buffer_capacity_retains_multiple_matching_commit_causes() {
    let fixture = ordinary_read_world("live-buffer-retention", 0);
    let owner = fixture.authenticate(OWNER);
    let teller = fixture.authenticate(TELLER);
    for ordinal in 0..16 {
        let outcome = commit_deposit(
            &fixture,
            &teller,
            fixture.personal_account,
            &format!("preexisting-live-history-{ordinal}"),
        );
        let work = outcome.mutation_work().unwrap();
        assert!(work.decision_fact_count() > 0);
        assert!(work.proposed_fact_count() > 0);
    }
    let mut live = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .subscribe(live_controls_for(request_scope(), 2))
        .expect("installed two-cause buffer should open");
    let first_commit = commit_deposit(
        &fixture,
        &teller,
        fixture.personal_account,
        "buffered-live-deposit-one",
    );
    let second_commit = commit_deposit(
        &fixture,
        &teller,
        fixture.personal_account,
        "buffered-live-deposit-two",
    );
    assert!(first_commit.mutation_work().is_some());
    assert!(second_commit.mutation_work().is_some());

    let first_outcome = live.poll(&owner, &request_scope());
    let BankAccountActivityLiveOutcome::Delivered(first) = first_outcome else {
        panic!("first exact activity cause must deliver")
    };
    assert_eq!(live.buffered_cause_count(), 1);
    let second_outcome = live.poll(&owner, &request_scope());
    let BankAccountActivityLiveOutcome::Delivered(second) = second_outcome else {
        panic!("second exact activity cause must deliver")
    };
    assert_eq!(live.buffered_cause_count(), 0);
    assert_eq!(
        first.result().entries()[0].account_sequence().get() + 1,
        second.result().entries()[0].account_sequence().get()
    );
    assert_eq!(first.result().entries().len(), 1);
    assert_eq!(second.result().entries().len(), 1);
    assert!(first.receipt().inspect().terminal_resources_released());
    assert!(second.receipt().inspect().terminal_resources_released());
}

#[test]
fn retained_commit_source_reports_exact_consumer_overflow() {
    let fixture = ordinary_read_world("live-source-overflow", 0);
    let owner = fixture.authenticate(OWNER);
    let mut live = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .subscribe(live_controls())
        .expect("authorized activity lease should open");

    for ordinal in 0..65 {
        commit_authorization_toggle(&fixture, &owner, ordinal);
    }

    let BankAccountActivityLiveOutcome::Overflow(overflow) = live.poll(&owner, &request_scope())
    else {
        panic!("a consumer older than the retained source must receive typed overflow");
    };
    assert_eq!(overflow.missed_commit_batches(), 1);
}
