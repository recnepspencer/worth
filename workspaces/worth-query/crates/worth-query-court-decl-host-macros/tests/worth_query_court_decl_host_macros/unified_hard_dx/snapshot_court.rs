//! `session.snapshot()` runs several typed queries against one pinned
//! committed basis while other sessions commit, and releases its pin on drop.
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock, DEFAULT_HOST_EXECUTION_POLICY,
};

use super::actions::SetTaskState;
use super::courts::{open_daymark, temp_home_path};
use super::program::owner;
use super::queries::{BlockedTasks, ReadyTasks};
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

#[test]
fn a_snapshot_reads_one_basis_across_a_concurrent_commit_and_releases_on_drop() {
    let path = temp_home_path("session-snapshot");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    {
        let snapshot = session.snapshot().unwrap();
        let pinned = snapshot.selected_commit().clone();
        assert_eq!(keys(&snapshot.query(ready()).unwrap()), vec![1, 3]);
        assert_eq!(
            keys(&snapshot.query(BlockedTasks { project: 1 }).unwrap()),
            vec![2]
        );

        // Another session closes Task 1, which unblocks Task 2.
        home.session(owner(&home, &clock))
            .run(SetTaskState {
                task: 1,
                state: "DONE".to_owned(),
            })
            .unwrap();

        // The snapshot keeps answering from its pinned basis, consistently.
        assert_eq!(snapshot.selected_commit(), &pinned);
        assert_eq!(keys(&snapshot.query(ready()).unwrap()), vec![1, 3]);
        assert_eq!(
            keys(&snapshot.query(BlockedTasks { project: 1 }).unwrap()),
            vec![2]
        );

        // The session itself, and a new snapshot, read the new commit.
        assert_eq!(keys(&session.query(ready()).unwrap()), vec![2, 3]);
        let current = session.snapshot().unwrap();
        assert_ne!(current.selected_commit(), &pinned);
        assert_eq!(current.branch_identity(), snapshot.branch_identity());
        assert!(current
            .query(BlockedTasks { project: 1 })
            .unwrap()
            .is_empty());
    }

    // Each dropped snapshot releases its pin: many in sequence never exhaust
    // retention, and the Home then retires with no basis still pinned.
    for _ in 0..256 {
        let snapshot = session.snapshot().unwrap();
        assert_eq!(keys(&snapshot.query(ready()).unwrap()), vec![2, 3]);
    }
    drop(session);
    drop(me);
    if let Err((_, stop)) = home
        .into_retirement()
        .try_retire(DEFAULT_HOST_EXECUTION_POLICY)
    {
        panic!("no released snapshot pins the Home: {stop:?}");
    }
    std::fs::remove_dir_all(path).unwrap();
}
