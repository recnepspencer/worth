//! W8B-2: `committed_basis_version()` changes on every commit and nothing else;
//! `on_commit` wakes after commits by any session to any entity, isolates a
//! panicking callback, unregisters on guard drop, and never blocks retirement.
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::time::Duration;

use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock, WorthQueryCommitWatchDenial,
    DEFAULT_HOST_EXECUTION_POLICY,
};

use super::actions::{RenameProject, SetTaskState};
use super::courts::{open_daymark, temp_home_path};
use super::program::{owner, DaymarkHome};
use super::queries::ReadyTasks;

fn ready() -> ReadyTasks {
    ReadyTasks {
        project: 1,
        priority: None,
        due_before: None,
    }
}

fn set_state(home: &DaymarkHome, clock: &WorthQueryAdmissionClock, task: u64, state: &str) {
    home.session(owner(home, clock))
        .run(SetTaskState {
            task,
            state: state.to_owned(),
        })
        .unwrap();
}

fn notified(rx: &Receiver<()>) {
    rx.recv_timeout(Duration::from_secs(30))
        .expect("the commit callback fires after the commit");
}

#[test]
fn the_committed_basis_version_and_on_commit_follow_every_commit_to_the_home() {
    let path = temp_home_path("commit-watch");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);
    let me = owner(&home, &clock);
    let session = home.session(&me);

    // Reads do not move the version; the owned session reports the same one.
    let before = home.committed_basis_version();
    session.query(ready()).unwrap();
    assert_eq!(home.committed_basis_version(), before);
    let owned = home.session(owner(&home, &clock)).into_owned();
    assert_eq!(owned.committed_basis_version(), before);

    let (tx, rx) = channel();
    let guard = home
        .on_commit(move || tx.send(()).unwrap_or_default())
        .unwrap();
    set_state(&home, &clock, 1, "DONE");
    notified(&rx);
    let after_task = home.committed_basis_version();
    assert!(after_task.changed_since(before));
    assert_eq!(owned.committed_basis_version(), after_task);

    // An out-of-band commit to another entity kind by another session wakes
    // the same watch, which no per-query subscription would have seen.
    owned
        .run(RenameProject {
            project: 2,
            label: "Later".to_owned(),
        })
        .unwrap();
    notified(&rx);
    assert!(home.committed_basis_version().changed_since(after_task));

    // A panicking callback is contained: the others keep firing, and so does
    // the notifier for later commits.
    let panicking = owned
        .on_commit(|| panic!("a bug inside a commit callback"))
        .unwrap();
    set_state(&home, &clock, 2, "DONE");
    notified(&rx);
    set_state(&home, &clock, 3, "DONE");
    notified(&rx);
    drop(panicking);

    // Dropping a guard unregisters it: once a later watch has seen a later
    // commit, the dropped callback (and its sender) is gone for good.
    drop(guard);
    let (later_tx, later_rx) = channel();
    let later = home
        .on_commit(move || later_tx.send(()).unwrap_or_default())
        .unwrap();
    set_state(&home, &clock, 3, "OPEN");
    notified(&later_rx);
    loop {
        match rx.try_recv() {
            Ok(()) => continue,
            Err(TryRecvError::Disconnected) => break,
            Err(TryRecvError::Empty) => panic!("a dropped guard's callback is still registered"),
        }
    }

    // Retirement neither waits for the notifier nor for the live guard.
    drop((session, me));
    if let Err((_, stop)) = home
        .into_retirement()
        .try_retire(DEFAULT_HOST_EXECUTION_POLICY)
    {
        panic!("a commit watch never holds the runtime: {stop:?}");
    }
    assert_eq!(
        owned.on_commit(|| ()).unwrap_err(),
        WorthQueryCommitWatchDenial::HomeClosed
    );
    drop((later, owned));
    std::fs::remove_dir_all(path).unwrap();
}
