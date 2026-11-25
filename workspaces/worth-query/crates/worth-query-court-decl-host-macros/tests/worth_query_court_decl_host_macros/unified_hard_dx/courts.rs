//! End-to-end Daymark acceptance courts exercising Target DX #1, #2, #3, #4, #5, #6 & #8.
use std::path::{Path, PathBuf};

use worth_foundational::facade::{CanonicalDate, CanonicalTemporalTextDenial};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_installation::ApplicationBuildIdentity, WorthQueryApplicationHome,
    WorthQueryHomeConfig, WorthQueryHomeOpenDisposition,
};

use super::actions::{RenameProject, RescheduleTask, SetTaskState};
use super::program::{owner, seed, DaymarkHome, DaymarkRuntime};
use super::queries::{BlockedTasks, FindTasksByTitle, LiveReadyTasks, ReadyTasks};

pub(super) fn build_id() -> ApplicationBuildIdentity {
    ApplicationBuildIdentity::parse("5a5a5a5a-5a5a-5a5a-5a5a-5a5a5a5a5a5a").unwrap()
}

pub(super) fn temp_home_path(label: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("unified-dx-{label}"));
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    path
}

pub(super) fn open_daymark(path: &Path, clock: &WorthQueryAdmissionClock) -> DaymarkHome {
    WorthQueryApplicationHome::open_directory(
        path,
        WorthQueryHomeConfig::new(build_id(), (DaymarkRuntime::new(),))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| seed(graph, installed)),
    )
    .expect("open_directory succeeds")
}

#[test]
fn target_dx_1_2_3_5_8_unified_query_session_directory_open_managed_index_and_temporal() {
    let base = CanonicalDate::parse_iso("2026-11-24").unwrap();
    assert_eq!(base.to_string(), "2026-11-24");
    assert_eq!("2026-11-24".parse::<CanonicalDate>().unwrap(), base);
    let plus_three = base.add_days(3).unwrap();
    assert_eq!(plus_three.to_string(), "2026-11-27");
    let start_ts = base.start_of_day_utc().unwrap();
    assert_eq!(start_ts.to_string(), "2026-11-24T00:00:00Z");
    assert_eq!(
        CanonicalDate::parse_iso("2026-02-30"),
        Err(CanonicalTemporalTextDenial::InvalidDate)
    );

    let path = temp_home_path("unified-query");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);
    assert_eq!(
        home.open_disposition(),
        WorthQueryHomeOpenDisposition::Created
    );
    {
        let me = owner(&home, &clock);
        let session = home.session(&me);

        // ReadyTasks: state="OPEN", no OPEN BlockedBy, optional priority & due_before.
        // Seeded Project 1 tasks:
        //   1: OPEN, prio 1, due 2026-11-25, unblocked
        //   2: OPEN, prio 1, due 2026-11-26, BLOCKED BY 1
        //   3: OPEN, prio 2, due 2026-11-29, unblocked
        //   4: DONE, prio 1, due 2026-11-24, unblocked
        let all_ready = session
            .query(ReadyTasks {
                project: 1,
                priority: None,
                due_before: None,
            })
            .unwrap();
        let keys: Vec<u64> = all_ready.iter().map(|r| r.task_key).collect();
        assert_eq!(keys, vec![1, 3]);

        // Optional priority filter
        let p1_ready = session
            .query(ReadyTasks {
                project: 1,
                priority: Some(1),
                due_before: None,
            })
            .unwrap();
        assert_eq!(
            p1_ready.iter().map(|r| r.task_key).collect::<Vec<_>>(),
            vec![1]
        );

        // Optional due_before <= comparison filter
        let due_soon = session
            .query(ReadyTasks {
                project: 1,
                priority: None,
                due_before: Some(plus_three),
            })
            .unwrap();
        assert_eq!(
            due_soon.iter().map(|r| r.task_key).collect::<Vec<_>>(),
            vec![1]
        );

        // Companion where_related_exists finds Task 2 blocked by open Task 1
        let blocked = session.query(BlockedTasks { project: 1 }).unwrap();
        assert_eq!(
            blocked.iter().map(|r| r.task_key).collect::<Vec<_>>(),
            vec![2]
        );

        // Managed index cold readiness + typed watch
        let _idx = session
            .ensure_index_ready(FindTasksByTitle {
                project: 1,
                prefix: String::new(),
            })
            .unwrap();
        let watch = session
            .watch(FindTasksByTitle {
                project: 1,
                prefix: "Draft".to_owned(),
            })
            .unwrap();
        assert_eq!(
            watch
                .current()
                .expect("watch snapshot is current")
                .iter()
                .map(|r| r.task_key)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        watch.close().unwrap();

        // Completing blocker Task 1 unblocks Task 2!
        let _ = session
            .run(SetTaskState {
                task: 1,
                state: "DONE".to_owned(),
            })
            .unwrap();
        let after_unblock = session
            .query(ReadyTasks {
                project: 1,
                priority: None,
                due_before: None,
            })
            .unwrap();
        assert_eq!(
            after_unblock.iter().map(|r| r.task_key).collect::<Vec<_>>(),
            vec![2, 3]
        );
    }
    home.close_clean().unwrap();

    // Reopen existing Directory Home -> Opened disposition + persisted state + cold index ready!
    let home = open_daymark(&path, &clock);
    assert_eq!(
        home.open_disposition(),
        WorthQueryHomeOpenDisposition::Opened
    );
    {
        let me = owner(&home, &clock);
        let session = home.session(&me);
        let ready = session
            .query(ReadyTasks {
                project: 1,
                priority: None,
                due_before: None,
            })
            .unwrap();
        assert_eq!(
            ready.iter().map(|r| r.task_key).collect::<Vec<_>>(),
            vec![2, 3]
        );
        let _idx = session
            .ensure_index_ready(FindTasksByTitle {
                project: 1,
                prefix: String::new(),
            })
            .unwrap();
        let matched = session
            .query(FindTasksByTitle {
                project: 1,
                prefix: "Draft".to_owned(),
            })
            .unwrap();
        assert_eq!(
            matched.iter().map(|r| r.task_key).collect::<Vec<_>>(),
            vec![1, 2]
        );
    }
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn target_dx_6_whole_result_live_subscription_wakes_on_blocker_and_due_date_not_unrelated() {
    let base = CanonicalDate::parse_iso("2026-11-24").unwrap();
    let cutoff = base.add_days(6).unwrap(); // 2026-11-30
    let path = temp_home_path("whole-result-live");
    let clock = WorthQueryAdmissionClock::system();
    let home = open_daymark(&path, &clock);
    {
        let me = owner(&home, &clock);
        let session = home.session(&me);
        let mut sub = session
            .subscribe(LiveReadyTasks {
                project: 1,
                priority: None,
                due_before: Some(cutoff),
            })
            .unwrap();
        assert_eq!(
            sub.current()
                .expect("initial live snapshot is current")
                .iter()
                .map(|r| r.task_key)
                .collect::<Vec<_>>(),
            vec![1, 3]
        );

        // Unrelated edit on Project 2 must NOT invalidate Project 1 subscription -> Ok(None) (Pending).
        let _ = session
            .run(RenameProject {
                project: 2,
                label: "Archived".to_owned(),
            })
            .unwrap();
        assert!(sub.refresh(&session).unwrap().is_none());

        // Closing blocker Task 1 unblocks Task 2 and removes Task 1 -> wakes with [2, 3]!
        let _ = session
            .run(SetTaskState {
                task: 1,
                state: "DONE".to_owned(),
            })
            .unwrap();
        let updated = sub
            .refresh(&session)
            .unwrap()
            .expect("blocker completion must wake whole-result live subscription");
        assert_eq!(
            updated.iter().map(|r| r.task_key).collect::<Vec<_>>(),
            vec![2, 3]
        );

        // Pushing Task 3's due date past cutoff -> wakes and removes Task 3 -> [2]!
        let _ = session
            .run(RescheduleTask {
                task: 3,
                due_date: Some(cutoff.add_days(5).unwrap()),
            })
            .unwrap();
        let after_reschedule = sub
            .refresh(&session)
            .unwrap()
            .expect("due-date edit must wake whole-result live subscription");
        assert_eq!(
            after_reschedule
                .iter()
                .map(|r| r.task_key)
                .collect::<Vec<_>>(),
            vec![2]
        );
        assert_eq!(
            sub.current()
                .expect("refreshed live snapshot is current")
                .iter()
                .map(|r| r.task_key)
                .collect::<Vec<_>>(),
            vec![2]
        );
        sub.close().unwrap();
    }
    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}
