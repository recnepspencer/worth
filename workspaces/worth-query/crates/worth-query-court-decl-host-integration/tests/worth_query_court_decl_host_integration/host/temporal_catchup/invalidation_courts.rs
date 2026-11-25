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
        open_memory_home_with_controls, open_memory_home_with_observation_limit, read_reminders,
        DesktopReminderConditional,
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

fn catch_up(
    home: &super::program::ReminderHome,
    ns: u64,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    max_steps: usize,
) -> worth_query_host::facade::temporal_contribution::WorthQueryTemporalCatchUpReport<
    DesktopReminderClock,
> {
    home.catch_up_temporal::<DesktopReminderConditional>(
        WorthQueryClockCoordinate::from_nanoseconds(ns),
        execution,
        WorthQueryTemporalCatchUpPolicy::bounded(max_steps).unwrap(),
    )
    .unwrap()
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

        let legit = catch_up(&home, 15, execution, 4);
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
        assert_eq!(
            catch_up(&home, 5, execution, 2).status(),
            &WorthQueryTemporalCatchUpStatus::CaughtUp
        );

        controls.set_next_gate_write(Some("updated-1".to_owned()));
        mutate_reminder(&home, active_reminder("i2", 2, 100, 0, 1), 3);
        controls.set_next_gate_write(None);
        let mut report = catch_up(&home, 15, execution, 4);
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
        assert_eq!(
            catch_up(&home, 5, execution, 2).status(),
            &WorthQueryTemporalCatchUpStatus::CaughtUp
        );

        controls.set_next_gate_write(Some("updated-before-stop".to_owned()));
        mutate_reminder(&home, active_reminder("i2", 2, 100, 0, 1), 3);
        controls.set_next_gate_write(None);
        controls.set_expire_auth_after_apply(Some(auth.clone()));

        let mut stopped = catch_up(&home, 25, execution, 4);
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

fn direct_count(home: &super::program::ReminderHome) -> usize {
    home.runtime()
        .runtime()
        .inspect_conditional_runtime()
        .retained_direct_delivery_count()
}

fn select_branch<'a>(
    home: &'a super::program::ReminderHome,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> worth_query_host::facade::primary_graph::WorthQuerySelectedProductOperation<
    'a,
    super::schema::DesktopReminderSchema,
> {
    home.runtime()
        .runtime()
        .with_execution_request_advancement(*execution, |phase| {
            home.runtime()
                .runtime()
                .on_branch(home.runtime().current_world())
                .select(&phase)
        })
        .unwrap()
        .unwrap()
}

fn deliver_direct_gate_change(
    home: &super::program::ReminderHome,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    controls: &InvokerControls,
    gate_tag: &str,
    input: MutateReminderInput,
    seq: u64,
) -> worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt {
    controls.set_next_gate_write(Some(gate_tag.to_owned()));
    let change = mutate_reminder_returning_change(home, input, seq)
        .expect("committed gate mutation retains performed relational change");
    controls.set_next_gate_write(None);
    let binding = installed(home);
    let selected = select_branch(home, execution);
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
    let worth_query_host::facade::runtime::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(
        receipt,
    ) = delivered
    else {
        panic!("direct delivery must succeed");
    };
    receipt
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
        assert_eq!(
            catch_up(&home, 5, execution, 2).status(),
            &WorthQueryTemporalCatchUpStatus::CaughtUp
        );

        deliver_direct_gate_change(
            &home,
            execution,
            &controls,
            "direct-carry-gate",
            active_reminder("i1", 1, 10, 0, 1),
            1,
        );
        assert_eq!(direct_count(&home), 1);

        // Advance branch head via a non-gate mutation BEFORE any clock
        // observation so a same-branch successor carries `direct`.
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
        assert_eq!(direct_count(&home), 0);
    });
}

#[test]
fn desktop_temporal_partial_scan_retains_direct_skip_evidence_until_commit_is_passed() {
    use worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationOutcome;
    let (fire_log, auth, controls) = (
        ReminderFireLog::default(),
        AuthControl::valid(),
        InvokerControls::default(),
    );
    with_leased_request(128 * 1024 * 1024, None, |execution| {
        let home = open_memory_home_with_observation_limit(
            fire_log.clone(),
            auth.clone(),
            controls.clone(),
            1,
            execution,
        );
        assert_eq!(
            catch_up(&home, 5, execution, 2).status(),
            &WorthQueryTemporalCatchUpStatus::CaughtUp
        );

        // Commit 1 (earlier relevant gate write, NOT directly delivered).
        controls.set_next_gate_write(Some("gate-earlier-commit".to_owned()));
        mutate_reminder(&home, active_reminder("i1", 1, 50, 0, 1), 1);
        // Commit 2 (later gate write, directly delivered into `direct`).
        let direct_receipt = deliver_direct_gate_change(
            &home,
            execution,
            &controls,
            "gate-later-direct",
            active_reminder("i2", 1, 60, 0, 1),
            2,
        );

        // Advance branch head via a non-gate commit so successor carries `direct`.
        mutate_reminder(&home, active_reminder("i2", 2, 70, 0, 1), 3);
        let binding = installed(&home);
        binding
            .desktop_clock()
            .advance_to(WorthQueryClockCoordinate::from_nanoseconds(10))
            .unwrap();
        let mut port = select_branch(&home, execution)
            .conditional_clock(execution, binding.clock_handle())
            .unwrap();

        // Step 1 (1-commit scan): consumes ONLY Commit 1 and retains Commit 2's direct skip key.
        let WorthQueryConditionalClockObservationOutcome::Accepted(mut step1) =
            port.observe(execution)
        else {
            panic!("step 1 must accept");
        };
        assert!(step1.authoritative_work_remaining());
        let batch1 = step1.take_granular_invalidation_batch();
        assert_eq!(batch1.bridge_deliveries().len(), 1);
        assert_ne!(
            batch1.bridge_deliveries()[0]
                .correspondence_receipt()
                .change_set()
                .commit_identity(),
            direct_receipt.change_set().commit_identity()
        );
        assert_eq!(direct_count(&home), 1);

        // Step 2: passes Commit 2, skips duplicate Bridge delivery, and emits exact direct receipt.
        let WorthQueryConditionalClockObservationOutcome::Accepted(mut step2) =
            port.observe(execution)
        else {
            panic!("step 2 must accept");
        };
        let batch2 = step2.take_granular_invalidation_batch();
        assert_eq!(batch2.bridge_deliveries().len(), 1);
        let emitted2 = batch2.bridge_deliveries()[0].correspondence_receipt();
        assert_eq!(emitted2.counters(), direct_receipt.counters());
        assert_eq!(
            emitted2.change_set().commit_identity(),
            direct_receipt.change_set().commit_identity()
        );
        assert_eq!(direct_count(&home), 0);
        drop((step1, step2, port));
        drop(binding);
    });
}
