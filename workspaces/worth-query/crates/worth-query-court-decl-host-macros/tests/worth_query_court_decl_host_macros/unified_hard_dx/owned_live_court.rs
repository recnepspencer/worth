//! A desktop-style value owns its Home and an owned live subscription side by
//! side, receives another session's change, and retires once it lets go.
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock, WorthQueryOwnedLiveSubscription,
    DEFAULT_HOST_EXECUTION_POLICY,
};

use super::actions::SetTaskState;
use super::courts::{open_daymark, temp_home_path};
use super::program::{owner, DaymarkHome};
use super::queries::LiveReadyTasks;
use super::schema::DaymarkSchema;

/// What a GUI `App` holds: no borrow ties the subscription to the Home.
struct Board {
    ready: Option<WorthQueryOwnedLiveSubscription<DaymarkSchema, LiveReadyTasks>>,
    home: DaymarkHome,
}

fn ready_tasks() -> LiveReadyTasks {
    LiveReadyTasks {
        project: 1,
        priority: None,
        due_before: None,
    }
}

fn keys(rows: &[super::schema::TaskRow]) -> Vec<u64> {
    rows.iter().map(|row| row.task_key).collect()
}

#[test]
fn a_struct_owns_its_home_and_an_owned_live_subscription_then_retires() {
    let path = temp_home_path("owned-live");
    let clock = WorthQueryAdmissionClock::system();
    let mut board = Board {
        ready: None,
        home: open_daymark(&path, &clock),
    };
    let me = owner(&board.home, &clock);
    board.ready = Some(
        board
            .home
            .session(&me)
            .subscribe_owned(ready_tasks())
            .unwrap(),
    );
    let closed = board
        .home
        .session(&me)
        .subscribe_owned(ready_tasks())
        .unwrap();
    closed.close().unwrap();
    assert_eq!(
        keys(board.ready.as_ref().unwrap().current().unwrap()),
        vec![1, 3]
    );

    // Another session closes blocker Task 1; the owned subscription wakes.
    board
        .home
        .session(owner(&board.home, &clock))
        .run(SetTaskState {
            task: 1,
            state: "DONE".to_owned(),
        })
        .unwrap();
    let session = board.home.session(&me);
    let live = board.ready.as_mut().unwrap();
    assert_eq!(keys(live.refresh(&session).unwrap().unwrap()), vec![2, 3]);
    assert!(live.refresh(&session).unwrap().is_none());
    drop(session);
    drop(me);

    // While the subscription is held the Home keeps its runtime; once the
    // subscription drops, the same retirement completes.
    let Board { ready, home } = board;
    let retiring = match home
        .into_retirement()
        .try_retire(DEFAULT_HOST_EXECUTION_POLICY)
    {
        Ok(()) => panic!("a held owned subscription keeps the Home's runtime open"),
        Err((retiring, _stop)) => retiring,
    };
    drop(ready);
    if let Err((_, stop)) = retiring.try_retire(DEFAULT_HOST_EXECUTION_POLICY) {
        panic!("the Home retires once the owned subscription drops: {stop:?}");
    }
    std::fs::remove_dir_all(path).unwrap();
}
