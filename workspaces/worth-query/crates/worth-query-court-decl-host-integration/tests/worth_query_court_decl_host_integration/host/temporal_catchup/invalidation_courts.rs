//! Integration courts for foreign clock-handle rejection and granular
//! invalidation delivery preservation across catch-up and post-progress stops
//! (Sol Round 1 blockers 4 & 6 and Round 2 finding 5).

use worth_query_host::facade::{
    domain::{
        WorthQueryClockCoordinate, WorthQueryClockSourceIdentity, WorthQueryClockTimelineIdentity,
    },
    temporal_contribution::{
        pump_temporal_catch_up_until, WorthQueryDesktopClockSource,
        WorthQueryDesktopTemporalHomeBinding, WorthQueryTemporalCatchUpDenial,
        WorthQueryTemporalCatchUpPolicy, WorthQueryTemporalCatchUpStatus,
    },
};

use super::{
    adapters::{AuthControl, DesktopReminderClock, InvokerControls, ReminderFireLog},
    program::{
        installed, mutate_reminder, mutate_reminder_returning_change, open_memory_home,
        open_memory_home_with_controls, read_reminders, DesktopReminderConditional,
    },
    schema::MutateReminderInput,
    with_leased_request,
};

fn active_reminder(
    identity: &str,
    revision: u64,
    due: u64,
    interval: u64,
    max_fires: u64,
) -> MutateReminderInput {
    MutateReminderInput {
        identity: identity.to_owned(),
        revision,
        due,
        interval,
        max_fires,
        fired_count: 0,
        lifecycle: "active".to_owned(),
    }
}

#[test]
fn desktop_temporal_foreign_clock_handle_same_marker_type_is_refused() {
    let (fire_log, auth) = (ReminderFireLog::default(), AuthControl::valid());
    with_leased_request(512 * 1024 * 1024, None, |execution| {
        let home = open_memory_home(fire_log.clone(), auth.clone(), execution);
        mutate_reminder(&home, active_reminder("i1", 1, 10, 0, 1), 1);
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
        assert!(matches!(
            refused,
            Err(WorthQueryTemporalCatchUpDenial::ForeignClockHandle)
        ));
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
    let (fire_log, auth, controls) = (
        ReminderFireLog::default(),
        AuthControl::valid(),
        InvokerControls::default(),
    );
    with_leased_request(512 * 1024 * 1024, None, |execution| {
        let home = open_memory_home_with_controls(
            fire_log.clone(),
            auth.clone(),
            controls.clone(),
            None,
            execution,
        );
        mutate_reminder(&home, active_reminder("i1", 1, 10, 10, 2), 1);
        mutate_reminder(&home, active_reminder("i2", 1, 100, 0, 1), 2);
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
        mutate_reminder(&home, active_reminder("i2", 2, 100, 0, 1), 3);
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
        assert!(!batches.is_empty());
        let installation = home.primary_graph().granular_invalidation_installation();
        assert!(batches.iter().all(|batch| installation.admits_batch(batch)));

        controls.set_next_gate_write(Some("updated-2".to_owned()));
        mutate_reminder(&home, active_reminder("i2", 3, 100, 0, 1), 4);
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
    let (fire_log, auth, controls) = (
        ReminderFireLog::default(),
        AuthControl::valid(),
        InvokerControls::default(),
    );
    with_leased_request(512 * 1024 * 1024, None, |execution| {
        let home = open_memory_home_with_controls(
            fire_log.clone(),
            auth.clone(),
            controls.clone(),
            None,
            execution,
        );
        mutate_reminder(&home, active_reminder("i1", 1, 10, 10, 2), 1);
        mutate_reminder(&home, active_reminder("i2", 1, 100, 0, 1), 2);
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
        mutate_reminder(&home, active_reminder("i2", 2, 100, 0, 1), 3);
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
        assert!(!batches.is_empty());
        let installation = home.primary_graph().granular_invalidation_installation();
        assert!(batches.iter().all(|batch| installation.admits_batch(batch)));
    });
}

#[test]
fn desktop_temporal_direct_delivery_carries_across_same_branch_successor() {
    let (fire_log, auth, controls) = (
        ReminderFireLog::default(),
        AuthControl::valid(),
        InvokerControls::default(),
    );
    with_leased_request(128 * 1024 * 1024, None, |execution| {
        let home = open_memory_home_with_controls(
            fire_log.clone(),
            auth.clone(),
            controls.clone(),
            None,
            execution,
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

        // Mutate a gate-watched commit and directly deliver its performed
        // relational change to the conditional binding (populating `direct`).
        controls.set_next_gate_write(Some("direct-carry-gate".to_owned()));
        let change = mutate_reminder_returning_change(&home, active_reminder("i1", 1, 10, 0, 1), 1)
            .expect("committed gate mutation retains performed relational change");
        controls.set_next_gate_write(None);
        let binding = installed(&home);
        let selected = home
            .runtime()
            .runtime()
            .with_execution_request_advancement(*execution, |phase| {
                home.runtime()
                    .runtime()
                    .on_branch(home.runtime().current_world())
                    .select(&phase)
            })
            .unwrap()
            .unwrap();
        let delivered = home
            .runtime()
            .runtime()
            .with_execution_request_advancement(*execution, |phase| {
                selected.deliver_relational_change_to_conditional(
                    &phase,
                    binding.clock_handle(),
                    0,
                    change,
                )
            })
            .unwrap()
            .unwrap();
        assert!(matches!(
            delivered,
            worth_query_host::facade::runtime::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(_)
        ));
        assert_eq!(
            home.runtime()
                .runtime()
                .inspect_conditional_runtime()
                .retained_direct_delivery_count(),
            1
        );
        drop((delivered, selected, binding));

        // Advance branch head via a non-gate mutation BEFORE any clock
        // observation so a same-branch successor inherits the commit cursor
        // before the gate commit and preserves direct preperformed evidence.
        mutate_reminder(&home, active_reminder("i2", 1, 50, 0, 1), 2);
        let mut streamed = Vec::new();
        let observed = home
            .catch_up_temporal_observing::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(15),
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
        assert_eq!(streamed.len(), 1);
        assert_eq!(streamed[0].observation().direct_truth_delivery_count(), 1);
        assert_eq!(streamed[0].bridge_deliveries().len(), 1);
        assert_eq!(
            home.runtime()
                .runtime()
                .inspect_conditional_runtime()
                .retained_direct_delivery_count(),
            0
        );
    });
}
