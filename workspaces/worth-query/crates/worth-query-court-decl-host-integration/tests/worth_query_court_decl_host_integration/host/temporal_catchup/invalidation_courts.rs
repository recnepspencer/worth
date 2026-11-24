//! Integration courts for foreign clock-handle rejection and granular
//! invalidation delivery preservation across catch-up and post-progress stops
//! (Sol Round 1 blockers 4 & 6 and Round 2 finding 5).

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
        installed, mutate_reminder, open_memory_home, open_memory_home_with_controls,
        read_reminders, DesktopReminderConditional,
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

#[test]
fn desktop_temporal_catch_up_preserves_earlier_batches_when_later_step_stops() {
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

        controls.set_next_gate_write(Some("updated-before-stop".to_owned()));
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
        controls.set_expire_auth_after_apply(Some(auth.clone()));

        let mut stopped = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(25),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .expect(
                "post-progress stop must return report preserving earlier progress and batches",
            );
        assert_eq!(
            stopped.status(),
            &WorthQueryTemporalCatchUpStatus::AuthorizationRefused
        );
        assert_eq!(stopped.fired(), 1);
        assert!(stopped.remaining());
        let batches = stopped.take_granular_invalidation_batches();
        assert!(
            !batches.is_empty(),
            "earlier step granular invalidation batches must not be lost when a later step stops"
        );
        let installation = home.primary_graph().granular_invalidation_installation();
        assert!(batches.iter().all(|batch| installation.admits_batch(batch)));
    });
}
