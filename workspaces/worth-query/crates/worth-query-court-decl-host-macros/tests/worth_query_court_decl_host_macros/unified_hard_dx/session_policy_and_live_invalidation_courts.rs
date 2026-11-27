//! Courts for session execution-policy enforcement, watch result ceilings,
//! and typed live-subscription / watch invalidation on refusal.
use std::num::NonZeroUsize;
use std::time::Duration;

use worth_foundational::facade::{CanonicalDate, ExecutionBudget};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAdmissionClock, WorthQueryAdmissionTimeController,
    },
    application_entry::{
        WorthQueryApplicationLiveOpenRequestDenial, WorthQueryApplicationRequestMutationDenial,
        WorthQueryApplicationRequestQueryDenial,
    },
    application_installation::ApplicationBuildIdentity,
    primary_graph::{WorthQueryApplicationOneShotDenialKind, WorthQueryManagedIndexDenialCause},
    WorthQueryApplicationHome, WorthQueryHomeConfig, WorthQuerySessionRunDenial,
    WorthQueryTypedLiveRefreshDenial, WorthQueryTypedManagedIndexRefreshDenial,
};

use super::actions::{RescheduleTask, SetTaskState};
use super::program::{owner, seed, DaymarkHome, DaymarkRuntime};
use super::queries::{FindTasksByTitle, LiveReadyTasks, ReadyTasks};
use super::title_correction::RenameTask;

fn open_memory_home(clock: &WorthQueryAdmissionClock) -> DaymarkHome {
    let build = ApplicationBuildIdentity::parse("5a5a5a5a-5a5a-5a5a-5a5a-5a5a5a5a5a5a").unwrap();
    WorthQueryApplicationHome::open_memory(
        WorthQueryHomeConfig::new(build, (DaymarkRuntime::new(),))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| seed(graph, installed)),
    )
    .expect("open_memory succeeds")
}

#[test]
fn zero_work_session_fails_closed_across_all_lanes() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open_memory_home(&clock);
    let me = owner(&home, &clock);
    let funded = home.session(&me);
    let _idx = funded
        .ensure_index_ready(FindTasksByTitle {
            project: 1,
            prefix: String::new(),
        })
        .expect("funded session readies managed index");

    let zero_work =
        home.session(&me)
            .with_budget(ExecutionBudget::new(NonZeroUsize::MIN, None, Some(0)));

    let run_err = zero_work
        .run(SetTaskState {
            task: 1,
            state: "DONE".to_owned(),
        })
        .expect_err("zero-work session must refuse run on a funded Home");
    assert!(
        matches!(
            run_err,
            WorthQuerySessionRunDenial::Request(
                WorthQueryApplicationRequestMutationDenial::ExecutionRequest(_)
            )
        ),
        "expected ExecutionRequest refusal on zero-work run, got {run_err:?}"
    );

    let query_err = zero_work
        .query(ReadyTasks {
            project: 1,
            priority: None,
            due_before: None,
        })
        .expect_err("zero-work session must refuse query on a funded Home");
    assert!(
        matches!(
            query_err,
            WorthQueryApplicationRequestQueryDenial::ExecutionRequest(_)
        ),
        "expected ExecutionRequest refusal on zero-work query, got {query_err:?}"
    );

    let sub_err = zero_work
        .subscribe(LiveReadyTasks {
            project: 1,
            priority: None,
            due_before: None,
        })
        .err()
        .expect("zero-work session must refuse subscribe on a funded Home");
    assert!(
        matches!(
            sub_err,
            WorthQueryApplicationLiveOpenRequestDenial::ExecutionRequest(_)
        ),
        "expected ExecutionRequest refusal on zero-work subscribe, got {sub_err:?}"
    );

    let watch_err = zero_work
        .watch(FindTasksByTitle {
            project: 1,
            prefix: "Draft".to_owned(),
        })
        .err()
        .expect("zero-work session must refuse watch on a funded Home");
    assert!(
        matches!(
            watch_err,
            WorthQueryApplicationRequestQueryDenial::ExecutionRequest(_)
        ),
        "expected ExecutionRequest refusal on zero-work watch, got {watch_err:?}"
    );

    let ready_err = zero_work
        .ensure_index_ready(FindTasksByTitle {
            project: 1,
            prefix: String::new(),
        })
        .err()
        .expect("zero-work session must refuse ensure_index_ready on a funded Home");
    assert!(
        matches!(
            ready_err,
            WorthQueryApplicationRequestQueryDenial::ExecutionRequest(_)
        ),
        "expected ExecutionRequest refusal on zero-work ensure_index_ready, got {ready_err:?}"
    );

    // Home remained untouched (Task 1 is still OPEN) and funded session works.
    let rows = funded
        .query(ReadyTasks {
            project: 1,
            priority: None,
            due_before: None,
        })
        .expect("funded session still succeeds");
    assert_eq!(
        rows.iter().map(|r| r.task_key).collect::<Vec<_>>(),
        vec![1, 3]
    );
}

#[test]
fn watch_refresh_enforces_result_limit_and_invalidates_stale_rows() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open_memory_home(&clock);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let _idx = session
        .ensure_index_ready(FindTasksByTitle {
            project: 1,
            prefix: String::new(),
        })
        .unwrap();

    // "Draft" matches 2 rows (Task 1 & Task 2); opening with limit 1 refuses.
    let one = NonZeroUsize::new(1).unwrap();
    let open_limit_err = session
        .clone()
        .with_result_limit(one)
        .watch(FindTasksByTitle {
            project: 1,
            prefix: "Draft".to_owned(),
        })
        .err()
        .expect("limit 1 must refuse 2-row managed-index watch open");
    let is_result_limit_exceeded = |denial: &WorthQueryApplicationRequestQueryDenial| match denial {
        WorthQueryApplicationRequestQueryDenial::Limit(_) => true,
        WorthQueryApplicationRequestQueryDenial::ManagedIndex(index) => matches!(
            index.cause(),
            WorthQueryManagedIndexDenialCause::QueryRead(one_shot)
                if one_shot.kind() == WorthQueryApplicationOneShotDenialKind::ResultLimitExceeded
        ),
        _ => false,
    };
    assert!(
        is_result_limit_exceeded(&open_limit_err),
        "expected ResultLimitExceeded/Limit denial on open, got {open_limit_err:?}"
    );

    // "Review" initially matches 1 row (Task 3: "Review docs"); opening with
    // limit 1 succeeds, ordinary refresh preserves limit 1 without ForeignContext,
    // and renaming Task 1 to "Review spec" (2 matching rows) causes refresh to
    // enforce the preserved limit 1 and clear stale cached rows!
    let mut watch = session
        .clone()
        .with_result_limit(one)
        .watch(FindTasksByTitle {
            project: 1,
            prefix: "Review".to_owned(),
        })
        .ok()
        .expect("limit 1 watch open succeeds for single matching row");
    assert!(watch.is_current());
    assert_eq!(watch.current().unwrap().len(), 1);

    let _ = session
        .run(SetTaskState {
            task: 3,
            state: "REVIEW".to_owned(),
        })
        .unwrap();
    let updated = watch
        .refresh(&session)
        .expect(
            "refresh with unlimited session preserves watch result_limit without ForeignContext",
        )
        .expect("state edit wakes watch with 1 row");
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].state, "REVIEW");

    let _ = session
        .run(RenameTask {
            task: 1,
            title: "Review spec".to_owned(),
        })
        .unwrap();
    let refresh_err = watch
        .refresh(&session)
        .expect_err("2 matching rows must breach watch's preserved result_limit=1 on refresh");
    assert!(
        matches!(
            refresh_err,
            WorthQueryTypedManagedIndexRefreshDenial::Query(ref query_err)
                if is_result_limit_exceeded(query_err)
        ),
        "expected ResultLimitExceeded/Limit denial on watch refresh, got {refresh_err:?}"
    );
    assert!(!watch.is_current());
    assert!(watch.current().is_none());
    assert!(watch.last_notification().is_none());
    watch.close().unwrap();
}

#[test]
fn live_subscription_overflow_and_result_limit_clear_stale_rows() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open_memory_home(&clock);
    let me = owner(&home, &clock);
    let session = home.session(&me);

    // 1. Queue overflow invalidates cached rows immediately.
    let mut sub = session
        .clone()
        .with_live_limits(1, 64)
        .subscribe(LiveReadyTasks {
            project: 1,
            priority: None,
            due_before: None,
        })
        .unwrap();
    assert!(sub.is_current());
    assert_eq!(sub.current().unwrap().len(), 2);

    let _ = session
        .run(SetTaskState {
            task: 1,
            state: "DONE".to_owned(),
        })
        .unwrap();
    let _ = session
        .run(SetTaskState {
            task: 3,
            state: "DONE".to_owned(),
        })
        .unwrap();
    let overflow = sub
        .refresh(&session)
        .expect_err("2 invalidating commits into capacity-1 live queue must overflow");
    assert!(
        matches!(overflow, WorthQueryTypedLiveRefreshDenial::Overflow(_)),
        "expected Overflow denial, got {overflow:?}"
    );
    assert!(!sub.is_current());
    assert!(sub.current().is_none());
    assert!(sub.current_snapshot().is_none());
    assert!(sub.last_publication().is_none());
    sub.close().unwrap();

    // 2. Result-limit breach on refresh also invalidates cached rows immediately.
    let cutoff = CanonicalDate::parse_iso("2026-11-26").unwrap();
    let mut limited = session
        .clone()
        .with_live_limits(64, 1)
        .subscribe(LiveReadyTasks {
            project: 1,
            priority: None,
            due_before: Some(cutoff),
        })
        .unwrap();
    assert!(limited.is_current());
    assert_eq!(
        limited
            .current()
            .unwrap()
            .iter()
            .map(|r| r.task_key)
            .collect::<Vec<_>>(),
        vec![2]
    );
    // Move Task 3's due date into the cutoff window first (still DONE), then
    // reopen it so a single refresh sees 2 matching rows > maximum_results=1.
    let _ = session
        .run(RescheduleTask {
            task: 3,
            due_date: Some(cutoff),
        })
        .unwrap();
    let _ = limited.refresh(&session).unwrap();
    let _ = session
        .run(SetTaskState {
            task: 3,
            state: "OPEN".to_owned(),
        })
        .unwrap();
    let limit_err = limited
        .refresh(&session)
        .expect_err("2 matching rows must breach maximum_results=1 on refresh");
    assert!(
        matches!(
            limit_err,
            WorthQueryTypedLiveRefreshDenial::ResultLimitExceeded
        ),
        "expected ResultLimitExceeded, got {limit_err:?}"
    );
    assert!(!limited.is_current());
    assert!(limited.current().is_none());
    assert!(limited.current_snapshot().is_none());
    limited.close().unwrap();
}

#[test]
fn live_subscription_expired_principal_clears_stale_rows() {
    let time = WorthQueryAdmissionTimeController::new().expect("controlled clock initializes");
    let _ = time.hold();
    let clock = time.clock();
    let home = open_memory_home(&clock);
    let expiring_me = owner(&home, &clock);
    let expiring_session = home.session(&expiring_me);

    let mut sub = expiring_session
        .subscribe(LiveReadyTasks {
            project: 1,
            priority: None,
            due_before: None,
        })
        .unwrap();
    assert!(sub.is_current());
    assert_eq!(sub.current().unwrap().len(), 2);

    // Advance past the principal's 600s authentication lifetime, then commit an
    // invalidating edit using a freshly authenticated principal.
    time.advance(Duration::from_secs(601))
        .expect("controlled clock advances");
    let fresh_me = owner(&home, &clock);
    let _ = home
        .session(&fresh_me)
        .run(SetTaskState {
            task: 1,
            state: "DONE".to_owned(),
        })
        .unwrap();

    // Refreshing with the expired principal must refuse AND clear cached rows!
    let auth_err = sub
        .refresh(&expiring_session)
        .expect_err("expired principal must refuse live subscription refresh");
    assert!(
        matches!(auth_err, WorthQueryTypedLiveRefreshDenial::SessionExpired),
        "expected an expired-session refusal, got {auth_err:?}"
    );
    assert!(!sub.is_current());
    assert!(sub.current().is_none());
    assert!(sub.current_snapshot().is_none());
    assert!(sub.last_publication().is_none());
    sub.close().unwrap();
}
