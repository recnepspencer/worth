//! W8B-1: an owned session's `spawn_*` calls return at once and complete on
//! the session's call worker, in order; they wake their caller, contain a
//! panicking call, refuse a full queue, and never block a Home's retirement.
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;

use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock, WorthQueryBackgroundDenial,
    DEFAULT_BACKGROUND_QUEUE_CAPACITY, DEFAULT_HOST_EXECUTION_POLICY,
};

use super::actions::SetTaskState;
use super::courts::{open_daymark, temp_home_path};
use super::program::owner;
use super::queries::ReadyTasks;
use super::schema::TaskRow;

fn keys(rows: &[TaskRow]) -> Vec<u64> {
    rows.iter().map(|row| row.task_key).collect()
}

fn ready() -> ReadyTasks {
    ReadyTasks {
        project: 1,
        priority: None,
        due_before: None,
    }
}

fn set_state(task: u64, state: &str) -> SetTaskState {
    SetTaskState {
        task,
        state: state.to_owned(),
    }
}

#[test]
fn spawned_calls_run_in_order_off_thread_wake_their_caller_and_contain_a_panic() {
    let path = temp_home_path("background-calls");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);
    let session = home.session(owner(&home, &clock)).into_owned();

    // A gate call holds the worker so the queued commit is observably pending.
    let (entered_tx, entered_rx) = channel();
    let (open_tx, open_rx) = channel::<()>();
    let mut gate = session.spawn_with_session(move |_| {
        entered_tx.send(()).unwrap();
        open_rx.recv().unwrap();
    });
    entered_rx.recv().unwrap();
    let (woken_tx, woken_rx) = channel();
    let mut commit = session
        .spawn_run(set_state(1, "DONE"))
        .with_waker(move || woken_tx.send(()).unwrap());
    assert!(!gate.is_ready() && gate.try_take().is_none());
    assert!(!commit.is_ready() && commit.try_take().is_none());
    assert!(woken_rx.try_recv().is_err(), "not woken while queued");
    open_tx.send(()).unwrap();
    gate.wait().unwrap();
    woken_rx
        .recv_timeout(Duration::from_secs(30))
        .expect("the waker fires when the commit completes");
    let committed = commit
        .wait_timeout(Duration::from_secs(30))
        .expect("a woken call is ready")
        .unwrap();
    drop(committed.into_parts());
    assert!(woken_rx.try_recv().is_err(), "woken exactly once");
    assert!(commit.try_take().is_none());
    assert!(matches!(
        commit.wait(),
        Err(WorthQueryBackgroundDenial::AlreadyTaken)
    ));

    // A read queued after the commit sees it; a waker added late fires at once.
    let late = Arc::new(AtomicUsize::new(0));
    let read = session.spawn_query(ready());
    let rows = read.wait().unwrap();
    assert_eq!(keys(&rows), vec![2, 3]);
    let done = session.spawn_with_session(|_| 7_u8);
    while !done.is_ready() {
        std::thread::yield_now();
    }
    let done = done.with_waker({
        let late = Arc::clone(&late);
        move || {
            late.fetch_add(1, Ordering::SeqCst);
        }
    });
    assert_eq!(late.load(Ordering::SeqCst), 1);
    assert_eq!(done.wait().unwrap(), 7);

    // A typed refusal stays typed; a panicking call is contained and the
    // worker keeps serving.
    let missing = session.spawn_run(set_state(99, "DONE")).wait();
    assert!(
        matches!(missing, Err(WorthQueryBackgroundDenial::Denied(_))),
        "{missing:?}"
    );
    let panicked = session
        .spawn_with_session(|_| -> u8 { panic!("a bug inside a background call") })
        .wait();
    assert!(matches!(
        panicked,
        Err(WorthQueryBackgroundDenial::Panicked)
    ));
    assert_eq!(
        keys(&session.spawn_query(ready()).wait().unwrap()),
        vec![2, 3]
    );

    drop(session);
    if let Err((_, stop)) = home
        .into_retirement()
        .try_retire(DEFAULT_HOST_EXECUTION_POLICY)
    {
        panic!("the call worker holds no runtime: {stop:?}");
    }
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn a_full_queue_refuses_busy_and_retirement_never_waits_on_the_worker() {
    let path = temp_home_path("background-retire");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);
    let session = home.session(owner(&home, &clock)).into_owned();

    // The running call holds a read; behind it the queue fills exactly.
    let (entered_tx, entered_rx) = channel();
    let (open_tx, open_rx) = channel::<()>();
    let running = session.spawn_with_session(move |session| {
        entered_tx.send(()).unwrap();
        open_rx.recv().unwrap();
        session.query(ready()).map(|rows| rows.len())
    });
    entered_rx.recv().unwrap();
    let queued = (0..DEFAULT_BACKGROUND_QUEUE_CAPACITY)
        .map(|_| session.spawn_query(ready()))
        .collect::<Vec<_>>();
    let mut busy = session.spawn_query(ready());
    assert!(busy.is_ready());
    assert!(matches!(
        busy.try_take(),
        Some(Err(WorthQueryBackgroundDenial::Busy))
    ));

    // Retiring while a call runs refuses (that call still holds the runtime)
    // instead of blocking, and releases the session for every later call.
    let retiring = match home
        .into_retirement()
        .try_retire(DEFAULT_HOST_EXECUTION_POLICY)
    {
        Ok(()) => panic!("a running call still holds the runtime"),
        Err((retiring, _)) => retiring,
    };
    assert!(!session.is_attached());
    open_tx.send(()).unwrap();
    assert_eq!(
        running.wait().unwrap().unwrap(),
        2,
        "the running call finished"
    );
    for call in queued {
        assert!(matches!(
            call.wait(),
            Err(WorthQueryBackgroundDenial::HomeClosed)
        ));
    }
    if let Err((_, stop)) = retiring.try_retire(DEFAULT_HOST_EXECUTION_POLICY) {
        panic!("no call holds the runtime any more: {stop:?}");
    }
    assert!(matches!(
        session.spawn_run(set_state(2, "DONE")).wait(),
        Err(WorthQueryBackgroundDenial::HomeClosed)
    ));
    drop(session);
    std::fs::remove_dir_all(path).unwrap();
}
