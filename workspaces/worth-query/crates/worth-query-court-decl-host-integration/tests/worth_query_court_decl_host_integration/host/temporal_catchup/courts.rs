//! Focused Directory Home & MemoryOnly integration courts for desktop temporal catch-up.

use worth_query_host::facade::{
    domain::WorthQueryClockCoordinate,
    temporal_contribution::{WorthQueryTemporalCatchUpPolicy, WorthQueryTemporalCatchUpStatus},
};

use super::{
    adapters::{AuthControl, FiredOccurrence, ReminderFireLog},
    program::*,
    schema::MutateReminderInput,
    with_cancelled_request, with_leased_request,
};

#[test]
fn desktop_temporal_reopen_catch_up_fires_missed_recurrences_and_completes_idempotently() {
    let dir = unique_temp_dir("reopen-recurrence");
    let fire_log = ReminderFireLog::default();
    let auth = AuthControl::valid();

    with_leased_request(512 * 1024 * 1024, None, |execution| {
        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        mutate_reminder(
            &home,
            MutateReminderInput {
                identity: "i1".to_owned(),
                revision: 1,
                due: 10,
                interval: 10,
                max_fires: 3,
                fired_count: 0,
                lifecycle: "active".to_owned(),
            },
            1,
        );
        let early = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(5),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .unwrap();
        assert_eq!(early.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
        assert_eq!(early.fired(), 0);
        assert_eq!(early.skipped(), 0);
        assert!(!early.remaining());
        assert!(fire_log.snapshot().is_empty());
        drop(early);

        home.close(execution)
            .expect("directory home closes cleanly before simulated offline gap")
            .reclaim();

        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        let caught_up = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(35),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(8).unwrap(),
            )
            .unwrap();
        assert_eq!(
            caught_up.status(),
            &WorthQueryTemporalCatchUpStatus::CaughtUp
        );
        assert_eq!(caught_up.fired(), 3);
        assert!(!caught_up.remaining());

        assert_eq!(
            fire_log.snapshot(),
            vec![
                FiredOccurrence {
                    id: "i1".to_owned(),
                    rev: 1,
                    due: 10,
                    fired_count_after: 1
                },
                FiredOccurrence {
                    id: "i1".to_owned(),
                    rev: 2,
                    due: 20,
                    fired_count_after: 2
                },
                FiredOccurrence {
                    id: "i1".to_owned(),
                    rev: 3,
                    due: 30,
                    fired_count_after: 3
                },
            ]
        );
        let rows = read_reminders(&home);
        assert_eq!(rows["i1"].lifecycle, "completed");
        assert_eq!(rows["i1"].revision, 4);
        assert_eq!(rows["i1"].fired_count, 3);

        let repump = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(50),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .unwrap();
        assert_eq!(repump.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
        assert_eq!(repump.fired(), 0);
        assert_eq!(fire_log.snapshot().len(), 3);

        home.close(execution).unwrap().reclaim();

        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        let second = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(100),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .unwrap();
        assert_eq!(second.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
        assert_eq!(second.fired(), 0);
        assert_eq!(fire_log.snapshot().len(), 3);
        home.close(execution).unwrap().reclaim();
    });
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn desktop_temporal_catch_up_handles_step_limit_cancellation_and_budget_exhaustion() {
    let dir = unique_temp_dir("partial-batch");
    let fire_log = ReminderFireLog::default();
    let auth = AuthControl::valid();

    with_leased_request(512 * 1024 * 1024, None, |execution| {
        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        mutate_reminder(
            &home,
            MutateReminderInput {
                identity: "i1".to_owned(),
                revision: 1,
                due: 10,
                interval: 10,
                max_fires: 4,
                fired_count: 0,
                lifecycle: "active".to_owned(),
            },
            1,
        );
        home.close(execution).unwrap().reclaim();

        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        let partial = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(50),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
            )
            .unwrap();
        assert_eq!(
            partial.status(),
            &WorthQueryTemporalCatchUpStatus::StepLimitReached,
            "partial report: {partial:?}, rows: {:?}, fires: {:?}",
            read_reminders(&home),
            fire_log.snapshot(),
        );
        assert_eq!(partial.fired(), 2);
        assert!(partial.remaining());
        assert_eq!(read_reminders(&home)["i1"].fired_count, 2);

        with_cancelled_request(|cancelled_exec| {
            let cancelled = home
                .pump_temporal::<DesktopReminderConditional>(
                    cancelled_exec,
                    WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
                )
                .unwrap();
            assert_eq!(
                cancelled.status(),
                &WorthQueryTemporalCatchUpStatus::Cancelled
            );
            assert_eq!(cancelled.fired(), 0);
        });
        assert_eq!(fire_log.snapshot().len(), 2);

        with_leased_request(128 * 1024 * 1024, Some(1), |tiny_exec| {
            let exhausted = home
                .pump_temporal::<DesktopReminderConditional>(
                    tiny_exec,
                    WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
                )
                .unwrap();
            assert_eq!(
                exhausted.status(),
                &WorthQueryTemporalCatchUpStatus::BudgetExhausted
            );
            assert_eq!(exhausted.fired(), 0);
        });
        assert_eq!(read_reminders(&home)["i1"].fired_count, 2);
        assert_eq!(fire_log.snapshot().len(), 2);

        home.close(execution).unwrap().reclaim();
        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        let resumed = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(50),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(8).unwrap(),
            )
            .unwrap();
        assert_eq!(resumed.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
        assert_eq!(resumed.fired(), 2);
        assert_eq!(
            fire_log
                .snapshot()
                .iter()
                .map(|entry| (entry.rev, entry.due))
                .collect::<Vec<_>>(),
            vec![(1, 10), (2, 20), (3, 30), (4, 40)]
        );
        assert_eq!(read_reminders(&home)["i1"].lifecycle, "completed");
        home.close(execution).unwrap().reclaim();
    });
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn desktop_temporal_catch_up_refuses_expired_principal_and_resumes_after_refresh() {
    let fire_log = ReminderFireLog::default();
    let auth = AuthControl::valid();
    with_leased_request(512 * 1024 * 1024, None, |execution| {
        let home = open_memory_home(fire_log.clone(), auth.clone(), execution);
        mutate_reminder(
            &home,
            MutateReminderInput {
                identity: "i1".to_owned(),
                revision: 1,
                due: 10,
                interval: 10,
                max_fires: 2,
                fired_count: 0,
                lifecycle: "active".to_owned(),
            },
            1,
        );
        auth.set_valid(false);
        let refused = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(25),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .unwrap();
        assert_eq!(
            refused.status(),
            &WorthQueryTemporalCatchUpStatus::AuthorizationRefused
        );
        assert_eq!(refused.fired(), 0);
        assert!(fire_log.snapshot().is_empty());
        assert_eq!(read_reminders(&home)["i1"].lifecycle, "active");

        auth.set_valid(true);
        let resumed = home
            .pump_temporal::<DesktopReminderConditional>(
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(6).unwrap(),
            )
            .unwrap();
        assert_eq!(resumed.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
        assert_eq!(resumed.fired(), 2);
        assert_eq!(fire_log.snapshot().len(), 2);
        assert_eq!(read_reminders(&home)["i1"].lifecycle, "completed");
    });
}

#[test]
fn desktop_temporal_cancelled_intent_never_fires_across_directory_reopen() {
    let dir = unique_temp_dir("cancelled-reopen");
    let fire_log = ReminderFireLog::default();
    let auth = AuthControl::valid();
    with_leased_request(512 * 1024 * 1024, None, |execution| {
        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        mutate_reminder(
            &home,
            MutateReminderInput {
                identity: "i1".to_owned(),
                revision: 1,
                due: 10,
                interval: 5,
                max_fires: 3,
                fired_count: 0,
                lifecycle: "active".to_owned(),
            },
            1,
        );
        mutate_reminder(
            &home,
            MutateReminderInput {
                identity: "i1".to_owned(),
                revision: 2,
                due: 10,
                interval: 5,
                max_fires: 3,
                fired_count: 0,
                lifecycle: "cancelled".to_owned(),
            },
            2,
        );
        home.close(execution).unwrap().reclaim();

        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        let report = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(50),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .unwrap();
        assert_eq!(report.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
        assert_eq!(report.fired(), 0);
        assert!(fire_log.snapshot().is_empty());
        assert_eq!(read_reminders(&home)["i1"].lifecycle, "cancelled");
        home.close(execution).unwrap().reclaim();
    });
    let _ = std::fs::remove_dir_all(&dir);
}
