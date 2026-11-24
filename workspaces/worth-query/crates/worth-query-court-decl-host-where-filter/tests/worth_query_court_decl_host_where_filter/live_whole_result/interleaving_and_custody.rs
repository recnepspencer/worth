//! Dedicated courts for whole-result live subscription concurrency, authority
//! boundaries, per-lease buffer limits, tight-budget refresh footprint capture,
//! and immediate source-custody detachment on terminal outcomes.
use super::access::{InspectBoard, ReadBoard, ReadBoardAccess};
use super::queries::{
    GovTodayIntent, ReadyBinding, ReadyIntent, ReadyLiveQuery, TaskRow, TodayIntent,
};
use super::schema::{Project, ProjectKey};
use super::world::{install, live_limits, run_mutate, FUTURE_DUE_DAYS, TODAY_CUTOFF_DAYS};
use super::{keys_of, mutate_op, seed};
use worth_query_decl::facade::application_query::ApplicationQueryParameterSet;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationLiveLimits, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationLiveCloseOutcome, WorthQueryApplicationLiveControls,
    WorthQueryApplicationLiveOpenDenialKind, WorthQueryApplicationWholeResultLiveOutcome,
    WorthQueryPrincipalResolutionMode,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

/// Blocker 1: First-subscriber interleaving — if World commits & retires an
/// unowned partition `B` between `select(A)` and `open_application_query_live_whole_result`,
/// open MUST refuse (`ProviderVersionUnavailable`), NEVER returning a lease
/// that silently lost commit `B`.
#[test]
fn first_subscriber_interleaving_refuses_stale_open_instead_of_losing_commit() {
    let host = install(
        &[
            seed(1, "p1", "open", Some(FUTURE_DUE_DAYS)),
            seed(2, "p1", "open", Some(TODAY_CUTOFF_DAYS)),
        ],
        &[(1, 2)],
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let binding = host
        .installed_schema()
        .installed_query_binding::<ReadyBinding>()
        .expect("ready binding installs");

    // 1. Select branch at commit A while zero subscribers exist.
    let (selected_a, resolved_principal, scope_entity) = host
        .runtime()
        .with_application_advancement(&scope, |phase| {
            let selected_a = host
                .runtime()
                .on_branch(host.runtime().current_world())
                .select(&phase)
                .expect("select at commit A");
            let resolved_principal = selected_a
                .resolve_authenticated_principal(
                    &phase,
                    binding.principal_binding(),
                    &principal,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .expect("principal resolves at A");
            let scope_entity = selected_a
                .resolve_entity(
                    &phase,
                    ProjectKey::at("ProjectKey"),
                    "p1".to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .expect("project p1 resolves at A");
            (selected_a, resolved_principal, scope_entity)
        })
        .expect("selection advancement runs");

    // 2. Independent advancement commits B and retires its unowned partition BEFORE open_funded runs!
    run_mutate(&host, mutate_op("p1", 0, 2, 0, 0, "done"), 1);

    // 3. Attempting to open from stale selection A in a new advancement MUST refuse cleanly.
    let controls = WorthQueryApplicationLiveControls::bounded(scope.clone(), 16, 64)
        .expect("valid live controls");
    let open_denial = host
        .runtime()
        .with_application_advancement(&scope, |phase| {
            selected_a
                .open_application_query_live_whole_result::<
                    ReadyLiveQuery,
                    super::queries::EmptyParams,
                    TaskRow,
                    super::schema::Principal,
                    u64,
                    Project,
                >(
                    &phase,
                    binding.into_query(),
                    &resolved_principal,
                    scope_entity,
                    ApplicationQueryParameterSet::new(),
                    controls,
                )
                .err()
                .expect("stale first-subscriber open after commit B must refuse")
        })
        .expect("open advancement runs");
    assert_eq!(
        open_denial.kind(),
        WorthQueryApplicationLiveOpenDenialKind::ProviderVersionUnavailable,
        "expected ProviderVersionUnavailable on first-subscriber interleaving, got {open_denial:?}"
    );

    // 4. Opening freshly at commit B observes B immediately and catches commit C.
    let mut fresh = host
        .runtime()
        .request(&principal, &scope)
        .query(ReadyIntent {
            project: "p1".into(),
        })
        .subscribe_whole_result(live_limits())
        .expect("fresh open at commit B succeeds");
    assert_eq!(
        fresh
            .initial()
            .unwrap()
            .rows()
            .iter()
            .map(|r| r.key)
            .collect::<Vec<_>>(),
        vec![1]
    );
    run_mutate(&host, mutate_op("p1", 0, 2, 0, 0, "open"), 2);
    assert_eq!(
        keys_of(
            fresh
                .next(&host.runtime().request(&principal, &scope))
                .unwrap()
        ),
        vec![2]
    );
    assert!(matches!(
        fresh.close(),
        WorthQueryApplicationLiveCloseOutcome::Completed(_)
    ));
}

/// Blockers 4 & 5: Per-lease `buffer_capacity` triggers `Overflow { missed }`
/// once unread commits exceed `buffer_capacity`, and immediately detaches the
/// handle from the live delivery source WITHOUT waiting for `close()` or `drop()`.
#[test]
fn buffer_capacity_overflow_enforces_limit_and_detaches_source_custody_immediately() {
    let host = install(
        &[
            seed(1, "p1", "open", Some(FUTURE_DUE_DAYS)),
            seed(2, "p1", "open", Some(FUTURE_DUE_DAYS)),
            seed(3, "p1", "open", Some(FUTURE_DUE_DAYS)),
        ],
        &[],
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let mut sub = host
        .runtime()
        .request(&principal, &scope)
        .query(TodayIntent {
            project: "p1".into(),
            cutoff_days: TODAY_CUTOFF_DAYS,
        })
        .subscribe_whole_result(WorthQueryApplicationLiveLimits::bounded(1, 64))
        .expect("bounded-1 whole-result sub opens");

    // Lag = 1 <= buffer_capacity(1): delivers cleanly.
    run_mutate(&host, mutate_op("p1", 1, 1, 0, TODAY_CUTOFF_DAYS, ""), 1);
    assert_eq!(
        keys_of(
            sub.next(&host.runtime().request(&principal, &scope))
                .unwrap()
        ),
        vec![1]
    );

    // Lag = 2 > buffer_capacity(1): MUST return Overflow { missed: 1 }.
    run_mutate(&host, mutate_op("p1", 1, 2, 0, TODAY_CUTOFF_DAYS, ""), 2);
    run_mutate(&host, mutate_op("p1", 1, 3, 0, TODAY_CUTOFF_DAYS, ""), 3);
    match sub
        .next(&host.runtime().request(&principal, &scope))
        .unwrap()
    {
        WorthQueryApplicationWholeResultLiveOutcome::Overflow(overflow) => {
            assert_eq!(overflow.missed_commit_batches(), 1);
        }
        other => panic!("expected Overflow(1), got {other:?}"),
    }

    // Blocker 4: WITHOUT closing or dropping `sub`, source MUST already have 0 consumers!
    let close_receipt = host.runtime().close_live_delivery();
    assert_eq!(
        close_receipt.remaining_live_consumers(),
        0,
        "overflowed handle must detach active source custody immediately before Drop"
    );
    assert!(matches!(
        sub.next(&host.runtime().request(&principal, &scope))
            .unwrap(),
        WorthQueryApplicationWholeResultLiveOutcome::Closed
    ));
    drop(sub);
}

/// Simultaneous ring eviction (64-batch ring) + tighter per-lease `buffer_capacity = 1`:
/// after 65 unread commits, missed count MUST reflect the stricter boundary (`64`),
/// never the weaker ring eviction count (`1`).
#[test]
fn simultaneous_ring_and_lease_buffer_overflow_reports_stricter_missed_count() {
    let host = install(&[seed(1, "p1", "open", Some(FUTURE_DUE_DAYS))], &[]);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let mut sub = host
        .runtime()
        .request(&principal, &scope)
        .query(TodayIntent {
            project: "p1".into(),
            cutoff_days: TODAY_CUTOFF_DAYS,
        })
        .subscribe_whole_result(WorthQueryApplicationLiveLimits::bounded(1, 64))
        .expect("bounded-1 sub opens");

    for seq in 1..=65_u64 {
        let next_due = if seq.is_multiple_of(2) {
            FUTURE_DUE_DAYS
        } else {
            TODAY_CUTOFF_DAYS
        };
        run_mutate(&host, mutate_op("p1", 1, 1, 0, next_due, ""), seq);
    }
    match sub
        .next(&host.runtime().request(&principal, &scope))
        .unwrap()
    {
        WorthQueryApplicationWholeResultLiveOutcome::Overflow(overflow) => {
            assert_eq!(
                overflow.missed_commit_batches(),
                64,
                "65 unread commits with buffer_capacity=1 and 64-batch ring must report stricter missed=64"
            );
        }
        other => panic!("expected Overflow(64), got {other:?}"),
    }
    assert_eq!(
        host.runtime()
            .close_live_delivery()
            .remaining_live_consumers(),
        0
    );
    drop(sub);
}

/// Blockers 2 & 4: Revocation before non-delivery (`Pending` and `Overflow`)
/// fails closed with `AuthorizationDenied` (never returning `Pending` or
/// leaking `Overflow { missed }`) and immediately detaches source custody.
#[test]
fn revocation_before_pending_and_overflow_fails_closed_and_detaches_custody() {
    // Sub-case A: revocation with zero task edits (would otherwise be Pending).
    {
        let host = install(&[seed(1, "p1", "open", Some(TODAY_CUTOFF_DAYS))], &[]);
        let scope = request_scope();
        let subscriber = authenticate_operator(host.installed_schema(), &scope);
        let mut sub = host
            .runtime()
            .request(&subscriber, &scope)
            .query(GovTodayIntent {
                project: "p1".into(),
                cutoff_days: TODAY_CUTOFF_DAYS,
            })
            .subscribe_whole_result_governed(
                ReadBoard::reference(),
                InspectBoard::reference(),
                ReadBoardAccess {
                    project: "p1".into(),
                },
                live_limits(),
            )
            .expect("governed sub opens");

        run_mutate(&host, mutate_op("p1", 6, 0, 2, 0, "grant-p1"), 1);
        assert!(matches!(
            sub.next(&host.runtime().request(&subscriber, &scope))
                .unwrap(),
            WorthQueryApplicationWholeResultLiveOutcome::AuthorizationDenied(_)
        ));
        let receipt = host.runtime().close_live_delivery();
        assert_eq!(
            receipt.remaining_live_consumers(),
            0,
            "revoked handle must detach active source custody before Drop"
        );
        drop(sub);
    }

    // Sub-case B: 2 commits exceed buffer_capacity(1), THEN grant revoked before next().
    // Must fail closed with AuthorizationDenied and NEVER leak Overflow { missed }.
    {
        let host = install(
            &[
                seed(1, "p1", "open", Some(FUTURE_DUE_DAYS)),
                seed(2, "p1", "open", Some(FUTURE_DUE_DAYS)),
            ],
            &[],
        );
        let scope = request_scope();
        let subscriber = authenticate_operator(host.installed_schema(), &scope);
        let mut sub = host
            .runtime()
            .request(&subscriber, &scope)
            .query(GovTodayIntent {
                project: "p1".into(),
                cutoff_days: TODAY_CUTOFF_DAYS,
            })
            .subscribe_whole_result_governed(
                ReadBoard::reference(),
                InspectBoard::reference(),
                ReadBoardAccess {
                    project: "p1".into(),
                },
                WorthQueryApplicationLiveLimits::bounded(1, 64),
            )
            .expect("bounded-1 governed sub opens");

        run_mutate(&host, mutate_op("p1", 1, 1, 0, TODAY_CUTOFF_DAYS, ""), 1);
        run_mutate(&host, mutate_op("p1", 1, 2, 0, TODAY_CUTOFF_DAYS, ""), 2);
        run_mutate(&host, mutate_op("p1", 6, 0, 2, 0, "grant-p1"), 3);

        assert!(matches!(
            sub.next(&host.runtime().request(&subscriber, &scope))
                .unwrap(),
            WorthQueryApplicationWholeResultLiveOutcome::AuthorizationDenied(_)
        ));
        let receipt = host.runtime().close_live_delivery();
        assert_eq!(receipt.remaining_live_consumers(), 0);
        drop(sub);
    }
}
