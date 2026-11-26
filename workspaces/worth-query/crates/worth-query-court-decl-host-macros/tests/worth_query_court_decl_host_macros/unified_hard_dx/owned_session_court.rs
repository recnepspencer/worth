//! A GUI-style value holds its Home, an owned `'static` session and an owned
//! live subscription with no self-reference, and retires the Home while the
//! session and subscription are still open; both then refuse as closed.
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock, WorthQueryOwnedLiveSubscription,
    WorthQueryOwnedSession, WorthQueryOwnedSessionDenial, WorthQueryRefusalCategory as Category,
    WorthQueryTypedLiveRefreshDenial, WorthQueryUserFacingRefusal, DEFAULT_HOST_EXECUTION_POLICY,
};

use super::actions::SetTaskState;
use super::courts::{open_daymark, temp_home_path};
use super::program::{owner, DaymarkHome, DaymarkProgram};
use super::queries::{LiveReadyTasks, ReadyTasks};
use super::schema::{DaymarkSchema, TaskRow};

/// What an egui or Tauri application state holds: three plain fields.
struct App {
    session: WorthQueryOwnedSession<DaymarkSchema, DaymarkProgram>,
    ready: WorthQueryOwnedLiveSubscription<DaymarkSchema, LiveReadyTasks>,
    home: DaymarkHome,
}

/// Tauri-style shared state needs the owned session to cross threads.
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<WorthQueryOwnedSession<DaymarkSchema, DaymarkProgram>>();
};

fn keys(rows: &[TaskRow]) -> Vec<u64> {
    rows.iter().map(|row| row.task_key).collect()
}

fn set_state(task: u64, state: &str) -> SetTaskState {
    SetTaskState {
        task,
        state: state.to_owned(),
    }
}

#[test]
fn an_app_retires_its_home_with_an_owned_session_and_live_subscription_open() {
    let path = temp_home_path("owned-session");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);
    let session = home.session(owner(&home, &clock)).into_owned();
    let ready = session
        .subscribe(LiveReadyTasks {
            project: 1,
            priority: None,
            due_before: None,
        })
        .unwrap();
    let mut app = App {
        session,
        ready,
        home,
    };
    assert!(app.session.is_attached());
    assert_eq!(keys(app.ready.current().unwrap()), vec![1, 3]);

    // Clones share one hold; each runs through the owned session.
    let worker = app.session.clone();
    worker.run(set_state(1, "DONE")).unwrap();
    let rows = app.ready.refresh_owned(&app.session).unwrap().unwrap();
    assert_eq!(keys(rows), vec![2, 3]);
    let priority_one = app
        .session
        .with_session(|session| {
            session.query(ReadyTasks {
                project: 1,
                priority: Some(1),
                due_before: None,
            })
        })
        .unwrap()
        .unwrap();
    assert_eq!(keys(&priority_one), vec![2]);

    // Retire with the session, its clone and the subscription all open.
    let App {
        session,
        mut ready,
        home,
    } = app;
    if let Err((_, stop)) = home
        .into_retirement()
        .try_retire(DEFAULT_HOST_EXECUTION_POLICY)
    {
        panic!("retirement releases open owned handles: {stop:?}");
    }
    assert!(!session.is_attached());
    let closed = worker.run(set_state(2, "DONE")).unwrap_err();
    assert!(matches!(closed, WorthQueryOwnedSessionDenial::HomeClosed));
    assert_eq!(closed.refusal_category(), Category::NotFound);
    assert!(matches!(
        session.query(ReadyTasks {
            project: 1,
            priority: None,
            due_before: None,
        }),
        Err(WorthQueryOwnedSessionDenial::HomeClosed)
    ));
    assert!(matches!(
        ready.refresh_owned(&session),
        Err(WorthQueryTypedLiveRefreshDenial::Closed)
    ));
    assert!(!ready.is_current());
    ready.close().unwrap();
    drop((session, worker));

    // The Directory reopens cleanly with the committed change.
    let reopened = open_daymark(&path, &clock);
    let me = owner(&reopened, &clock);
    let rows = reopened
        .session(&me)
        .query(ReadyTasks {
            project: 1,
            priority: None,
            due_before: None,
        })
        .unwrap();
    assert_eq!(keys(&rows), vec![2, 3]);
    drop(me);
    if let Err((_, stop)) = reopened
        .into_retirement()
        .try_retire(DEFAULT_HOST_EXECUTION_POLICY)
    {
        panic!("the reopened Home retires: {stop:?}");
    }
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn a_detached_refresh_closes_a_subscription_pinned_to_another_thread_so_retirement_completes() {
    use std::sync::mpsc::channel;
    let path = temp_home_path("owned-cross-thread");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);
    let session = home.session(owner(&home, &clock)).into_owned();
    let (opened_tx, opened_rx) = channel();
    let (poll_tx, poll_rx) = channel::<()>();
    let (closed_tx, closed_rx) = channel();
    let worker = std::thread::spawn({
        let session = session.clone();
        move || {
            let mut ready = session
                .subscribe(LiveReadyTasks {
                    project: 1,
                    priority: None,
                    due_before: None,
                })
                .unwrap();
            opened_tx.send(()).unwrap();
            poll_rx.recv().unwrap();
            let refused = ready.refresh_owned(&session);
            closed_tx
                .send(matches!(
                    refused,
                    Err(WorthQueryTypedLiveRefreshDenial::Closed)
                ))
                .unwrap();
            poll_rx.recv().unwrap();
            drop(ready);
        }
    });
    opened_rx.recv().unwrap();

    // The subscription's lease is pinned to the worker thread, so retiring
    // here releases the session but must wait for the subscription.
    let retiring = match home
        .into_retirement()
        .try_retire(DEFAULT_HOST_EXECUTION_POLICY)
    {
        Ok(()) => panic!("a lease pinned to another thread still holds the runtime"),
        Err((retiring, _)) => retiring,
    };
    assert!(!session.is_attached());
    poll_tx.send(()).unwrap();
    assert!(
        closed_rx.recv().unwrap(),
        "a detached refresh refuses Closed"
    );
    if let Err((_, stop)) = retiring.try_retire(DEFAULT_HOST_EXECUTION_POLICY) {
        panic!("the closed lease no longer holds the runtime: {stop:?}");
    }
    poll_tx.send(()).unwrap();
    worker.join().unwrap();
    drop(session);
    std::fs::remove_dir_all(path).unwrap();
}
