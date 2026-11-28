//! An expired session proof is a distinct, retryable `SessionExpired`
//! refusal (never a missing permission), a live subscription survives a
//! refresh with an expired proof, and a self-renewing local session stays
//! valid for far longer than one proof lifetime.
use std::time::Duration;

use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAdmissionTimeController, WorthQueryLocalProcessPrincipal,
    },
    WorthQueryRefusalCategory as Category, WorthQueryTypedLiveRefreshDenial,
    WorthQueryUserFacingRefusal,
};

use super::actions::SetTaskState;
use super::program::{owner, owner_identity, HostAudience, LocalProcessMethod};
use super::queries::{FindTasksByTitle, LiveReadyTasks, ReadyTasks};
use super::schema::TaskRow;
use super::serial_policy_and_custody_boundary_courts::open_memory_home_with_policy;

fn ready() -> ReadyTasks {
    ReadyTasks {
        project: 1,
        priority: None,
        due_before: None,
    }
}

fn live_ready() -> LiveReadyTasks {
    LiveReadyTasks {
        project: 1,
        priority: None,
        due_before: None,
    }
}

fn keys(rows: &[TaskRow]) -> Vec<u64> {
    rows.iter().map(|row| row.task_key).collect()
}

fn assert_session_expired(refusal: &dyn WorthQueryUserFacingRefusal) {
    let summary = refusal.user_summary();
    assert_eq!(summary.category(), Category::SessionExpired);
    assert!(summary.retryable(), "an expired proof is retryable");
    assert_ne!(
        summary.user_message(),
        Category::PermissionDenied.default_message()
    );
}

fn controlled_clock() -> WorthQueryAdmissionTimeController {
    let time = WorthQueryAdmissionTimeController::new().expect("controlled clock initializes");
    let _ = time.hold();
    time
}

#[test]
fn an_expired_proof_refuses_reads_writes_and_live_opens_as_session_expired() {
    let time = controlled_clock();
    let clock = time.clock();
    let home = open_memory_home_with_policy(&clock, None);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    time.advance(Duration::from_secs(601)).unwrap();

    let write = session
        .run(SetTaskState {
            task: 1,
            state: "DONE".to_owned(),
        })
        .unwrap_err();
    assert_session_expired(&write);
    assert_session_expired(&session.query(ready()).unwrap_err());
    assert_session_expired(&session.subscribe(live_ready()).err().unwrap());
    let snapshot = session.snapshot().unwrap();
    assert_session_expired(&snapshot.query(ready()).unwrap_err());
}

#[test]
fn a_live_subscription_survives_an_expired_refresh_and_resumes_with_a_fresh_session() {
    let time = controlled_clock();
    let clock = time.clock();
    let home = open_memory_home_with_policy(&clock, None);
    let expiring = owner(&home, &clock);
    let mut unchanged = home.session(&expiring).subscribe(live_ready()).unwrap();
    let mut changed = home.session(&expiring).subscribe(live_ready()).unwrap();
    time.advance(Duration::from_secs(601)).unwrap();

    let fresh = owner(&home, &clock);
    for live in [&mut unchanged, &mut changed] {
        let refused = live.refresh(&home.session(&expiring)).unwrap_err();
        assert!(matches!(
            refused,
            WorthQueryTypedLiveRefreshDenial::SessionExpired
        ));
        assert_session_expired(&refused);
        assert!(!live.is_current(), "an unconfirmed snapshot is withheld");
    }

    // With no commit since, a fresh poll confirms the withheld snapshot.
    let confirmed = unchanged.refresh(&home.session(&fresh)).unwrap();
    assert_eq!(keys(confirmed.expect("withheld rows return")), vec![1, 3]);
    assert!(unchanged.refresh(&home.session(&fresh)).unwrap().is_none());

    // A commit while refused is delivered to the next fresh poll.
    home.session(&fresh)
        .run(SetTaskState {
            task: 1,
            state: "DONE".to_owned(),
        })
        .unwrap();
    let delivered = changed.refresh(&home.session(&fresh)).unwrap();
    assert_eq!(
        keys(delivered.expect("the commit is delivered")),
        vec![2, 3]
    );
    unchanged.close().unwrap();
    changed.close().unwrap();
}

#[test]
fn a_local_session_renews_its_proof_far_past_one_lifetime() {
    let time = controlled_clock();
    let clock = time.clock();
    let home = open_memory_home_with_policy(&clock, None);
    let local = WorthQueryLocalProcessPrincipal::new(
        owner_identity(),
        HostAudience,
        LocalProcessMethod,
        Duration::from_secs(300),
    )
    .unwrap();
    let session = home.local_session(local, &clock).unwrap();
    assert!(session.renews_principal());
    let fixed = home.session(session.principal().clone());
    let mut live = session.subscribe(live_ready()).unwrap();

    // Twenty polls 250 s apart span 5000 s, more than sixteen proof lifetimes.
    for poll in 0..20_u64 {
        time.advance(Duration::from_secs(250)).unwrap();
        assert_eq!(keys(&session.query(ready()).unwrap()), vec![1, 3]);
        assert!(live.refresh(&session).unwrap().is_none(), "poll {poll}");
    }
    session
        .run(SetTaskState {
            task: 1,
            state: "DONE".to_owned(),
        })
        .unwrap();
    assert_eq!(keys(live.refresh(&session).unwrap().unwrap()), vec![2, 3]);

    // The creation proof itself lapsed long ago.
    assert_session_expired(&fixed.query(ready()).unwrap_err());
    live.close().unwrap();
}

#[test]
fn an_expired_proof_on_managed_index_reads_is_session_expired() {
    let time = controlled_clock();
    let clock = time.clock();
    let home = open_memory_home_with_policy(&clock, None);
    let me = owner(&home, &clock);
    let indexed = FindTasksByTitle {
        project: 1,
        prefix: "Draft".to_owned(),
    };
    let mut watch = home.session(&me).watch(indexed.clone()).unwrap();
    time.advance(Duration::from_secs(601)).unwrap();

    let session = home.session(&me);
    assert_session_expired(&watch.refresh(&session).err().unwrap());
    assert_session_expired(&session.query_indexed(indexed.clone()).unwrap_err());
    assert_session_expired(&session.ensure_index_ready(indexed).err().unwrap());
}
