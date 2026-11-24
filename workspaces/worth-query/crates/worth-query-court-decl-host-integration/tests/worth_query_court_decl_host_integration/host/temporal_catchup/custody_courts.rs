//! Integration courts for foreign clock-handle rejection, live continuation
//! preservation across fresh branch selection, and granular invalidation
//! delivery preservation (Sol Round 1 blockers 4, 5 & 6).

use worth_query_host::facade::{
    domain::{
        WorthQueryClockCoordinate, WorthQueryClockSourceIdentity, WorthQueryClockTimelineIdentity,
    },
    temporal_contribution::{
        pump_temporal_catch_up_until, WorthQueryDesktopClockSource,
        WorthQueryTemporalCatchUpDenial, WorthQueryTemporalCatchUpPolicy,
        WorthQueryTemporalCatchUpStatus,
    },
};

use super::{
    adapters::{AuthControl, DesktopReminderClock, InvokerControls, ReminderFireLog},
    program::{
        installed, mutate_reminder, open_directory_home, open_memory_home,
        open_memory_home_with_controls, read_reminders, unique_temp_dir,
        DesktopReminderConditional,
    },
    schema::MutateReminderInput,
    with_leased_request,
};

#[test]
fn desktop_temporal_foreign_clock_handle_same_marker_type_is_refused() {
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
                interval: 0,
                max_fires: 1,
                fired_count: 0,
                lifecycle: "active".to_owned(),
            },
            1,
        );
        let (_foreign_source, foreign_handle) =
            WorthQueryDesktopClockSource::<DesktopReminderClock>::new(
                WorthQueryClockSourceIdentity::declare(
                    "worth.query.court.desktop-reminder.clock-source",
                )
                .unwrap(),
                WorthQueryClockTimelineIdentity::declare(
                    "worth.query.court.desktop-reminder.timeline",
                )
                .unwrap(),
                WorthQueryClockCoordinate::from_nanoseconds(0),
            );
        let binding = installed(&home);
        let refused = pump_temporal_catch_up_until(
            home.runtime(),
            home.runtime().current_world(),
            binding.as_ref(),
            &foreign_handle,
            WorthQueryClockCoordinate::from_nanoseconds(15),
            execution,
            WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
        );
        assert!(
            matches!(
                refused,
                Err(WorthQueryTemporalCatchUpDenial::ForeignClockHandle)
            ),
            "foreign clock handle of same marker type must be refused: {refused:?}"
        );
        assert_eq!(foreign_handle.current_time().nanoseconds(), 0);
        assert_eq!(read_reminders(&home)["i1"].fired_count, 0);
        drop(binding);

        let legit = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(15),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .unwrap();
        assert_eq!(legit.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
        assert_eq!(legit.fired(), 1);
    });
}

#[test]
fn desktop_temporal_live_continuation_survives_fresh_branch_selection() {
    let dir = unique_temp_dir("live-continuation-branch-advance");
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
                interval: 0,
                max_fires: 1,
                fired_count: 0,
                lifecycle: "active".to_owned(),
            },
            1,
        );
        let rejection = home
            .runtime()
            .runtime()
            .reject_next_commit_before_transaction_for_test()
            .unwrap();
        home.runtime().runtime().lose_next_commit_answer_for_test();
        let report = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(15),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
            )
            .unwrap();
        drop(rejection);
        assert_eq!(
            report.status(),
            &WorthQueryTemporalCatchUpStatus::BlockedByRetainedDecision
        );
        assert_eq!(report.indeterminate(), 1);
        let obligations_before = home.runtime().conditional_decision_obligations();
        assert_eq!(obligations_before.len(), 1);

        mutate_reminder(
            &home,
            MutateReminderInput {
                identity: "i2".to_owned(),
                revision: 1,
                due: 50,
                interval: 0,
                max_fires: 1,
                fired_count: 0,
                lifecycle: "active".to_owned(),
            },
            2,
        );
        auth.set_valid(false);
        let after_branch_move = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(16),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
            )
            .unwrap();
        assert_eq!(
            after_branch_move.status(),
            &WorthQueryTemporalCatchUpStatus::AuthorizationRefused
        );
        assert!(after_branch_move.remaining());
        assert_eq!(
            home.runtime().conditional_decision_obligations(),
            obligations_before,
            "fresh branch selection must not drop live continuation custody"
        );

        auth.set_valid(true);
        let resumed = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(20),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .unwrap();
        assert_eq!(resumed.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
        assert_eq!(resumed.fired(), 1);
        assert!(
            home.runtime().conditional_decision_obligations().is_empty(),
            "resolved continuation must discharge and clear obligations"
        );
        assert_eq!(read_reminders(&home)["i1"].lifecycle, "completed");
        drop((report, after_branch_move, resumed));
        home.close(execution).unwrap().reclaim();

        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        assert!(home.runtime().conditional_decision_obligations().is_empty());
        let after_reopen = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(30),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .unwrap();
        assert_eq!(
            after_reopen.status(),
            &WorthQueryTemporalCatchUpStatus::CaughtUp
        );
        assert_eq!(after_reopen.fired(), 0);
        drop(after_reopen);
        home.close(execution).unwrap().reclaim();
    });
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn desktop_temporal_catch_up_preserves_and_delivers_owed_granular_invalidations() {
    let fire_log = ReminderFireLog::default();
    let auth = AuthControl::valid();
    let controls = InvokerControls::default();
    with_leased_request(512 * 1024 * 1024, None, |execution| {
        let home = open_memory_home_with_controls(
            fire_log.clone(),
            auth.clone(),
            controls.clone(),
            None,
            execution,
        );
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
        mutate_reminder(
            &home,
            MutateReminderInput {
                identity: "i2".to_owned(),
                revision: 1,
                due: 100,
                interval: 0,
                max_fires: 1,
                fired_count: 0,
                lifecycle: "active".to_owned(),
            },
            2,
        );
        let baseline = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(5),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
            )
            .unwrap();
        assert_eq!(
            baseline.status(),
            &WorthQueryTemporalCatchUpStatus::CaughtUp
        );

        controls.set_next_gate_write(Some("updated-1".to_owned()));
        mutate_reminder(
            &home,
            MutateReminderInput {
                identity: "i2".to_owned(),
                revision: 2,
                due: 100,
                interval: 0,
                max_fires: 1,
                fired_count: 0,
                lifecycle: "active".to_owned(),
            },
            3,
        );
        controls.set_next_gate_write(None);
        let mut report = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(15),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .unwrap();
        assert_eq!(report.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
        assert_eq!(report.fired(), 1);
        let batches = report.take_granular_invalidation_batches();
        assert!(
            !batches.is_empty(),
            "catch_up_temporal must retain non-empty granular invalidation batches"
        );
        let installation = home.primary_graph().granular_invalidation_installation();
        assert!(batches.iter().all(|batch| installation.admits_batch(batch)));

        controls.set_next_gate_write(Some("updated-2".to_owned()));
        mutate_reminder(
            &home,
            MutateReminderInput {
                identity: "i2".to_owned(),
                revision: 3,
                due: 100,
                interval: 0,
                max_fires: 1,
                fired_count: 0,
                lifecycle: "active".to_owned(),
            },
            4,
        );
        controls.set_next_gate_write(None);
        let mut streamed = Vec::new();
        let observed = home
            .catch_up_temporal_observing::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(25),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
                |batch| streamed.push(batch),
            )
            .unwrap();
        assert_eq!(
            observed.status(),
            &WorthQueryTemporalCatchUpStatus::CaughtUp
        );
        assert_eq!(observed.fired(), 1);
        assert!(!streamed.is_empty());
        assert!(observed.granular_invalidation_batches().is_empty());
        assert!(streamed
            .iter()
            .all(|batch| installation.admits_batch(batch)));
    });
}
