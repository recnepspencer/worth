//! Directory Home integration courts for live continuation custody
//! preservation across fresh branch selection and explicit `retry_original_key`
//! settlement (Sol Round 1 blocker 5 & Round 2 finding 4).

use super::pump::{catch_up as pump_reminders, catch_up_observing};

use worth_query_host::facade::{
    domain::WorthQueryClockCoordinate,
    primary_graph::{
        WorthQueryConditionalClockObservationOutcome, WorthQueryConditionalOriginalResolution,
        WorthQueryGranularInvalidationCoverage,
    },
    temporal_contribution::{
        WorthQueryDesktopTemporalHomeBinding, WorthQueryTemporalCatchUpPolicy,
        WorthQueryTemporalCatchUpReport, WorthQueryTemporalCatchUpStatus,
    },
};

use super::{
    adapters::{AuthControl, InvokerControls, ReminderFireLog},
    program::{
        installed, mutate_reminder, open_directory_home, open_directory_home_configured,
        read_reminders, unique_temp_dir, ReminderHome,
    },
    schema::MutateReminderInput,
    with_leased_request,
};

fn active_reminder(identity: &str, revision: u64, due: u64) -> MutateReminderInput {
    MutateReminderInput {
        identity: identity.to_owned(),
        revision,
        due,
        interval: 0,
        max_fires: 1,
        fired_count: 0,
        lifecycle: "active".to_owned(),
    }
}

fn managed_clock_count(home: &ReminderHome) -> usize {
    home.runtime()
        .runtime()
        .inspect_conditional_runtime()
        .managed_clock_count()
}

fn obligations(home: &ReminderHome) -> Vec<Obligation> {
    home.runtime().conditional_decision_obligations()
}

fn assert_caught_up(
    report: &WorthQueryTemporalCatchUpReport<super::adapters::DesktopReminderClock>,
) {
    assert_eq!(report.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
}

fn select_current_branch<'a>(
    home: &'a ReminderHome,
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

fn catch_up(
    home: &ReminderHome,
    ns: u64,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    max_steps: usize,
) -> WorthQueryTemporalCatchUpReport<super::adapters::DesktopReminderClock> {
    pump_reminders(
        home,
        WorthQueryClockCoordinate::from_nanoseconds(ns),
        execution,
        WorthQueryTemporalCatchUpPolicy::bounded(max_steps).unwrap(),
    )
    .unwrap()
}

type ReminderPort<'a> =
    worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationPort<
        'a,
        super::schema::DesktopReminderSchema,
        super::adapters::DesktopReminderNode,
        super::adapters::DesktopReminderClock,
    >;
type Obligation = worth_query_host::facade::primary_graph::QueryConditionalDecision;

fn assert_retry_published(
    port: &mut ReminderPort<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    ob: &Obligation,
) {
    assert!(matches!(
        port.retry_original_key(execution, ob.key, ob.intent)
            .unwrap(),
        WorthQueryConditionalOriginalResolution::Published(_)
    ));
}

#[test]
fn desktop_temporal_live_continuation_survives_fresh_branch_selection() {
    let dir = unique_temp_dir("live-continuation-branch-advance");
    let (fire_log, auth) = (ReminderFireLog::default(), AuthControl::valid());
    with_leased_request(512 * 1024 * 1024, None, |execution| {
        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        mutate_reminder(&home, active_reminder("i1", 1, 10), 1);
        let rejection = home
            .runtime()
            .runtime()
            .reject_next_commit_before_transaction_for_test()
            .unwrap();
        home.runtime().runtime().lose_next_commit_answer_for_test();
        let report = catch_up(&home, 15, execution, 2);
        drop(rejection);
        assert_eq!(
            report.status(),
            &WorthQueryTemporalCatchUpStatus::BlockedByRetainedDecision
        );
        assert_eq!(report.indeterminate(), 1);
        let obligations_before = obligations(&home);
        assert_eq!(obligations_before.len(), 1);

        mutate_reminder(&home, active_reminder("i2", 1, 50), 2);
        auth.set_valid(false);
        let after_branch_move = catch_up(&home, 16, execution, 2);
        assert_eq!(
            after_branch_move.status(),
            &WorthQueryTemporalCatchUpStatus::AuthorizationRefused
        );
        assert!(after_branch_move.remaining());
        assert_eq!(obligations(&home), obligations_before);

        auth.set_valid(true);
        let resumed = catch_up(&home, 20, execution, 4);
        assert_caught_up(&resumed);
        assert_eq!(resumed.fired(), 1);
        assert!(obligations(&home).is_empty());
        assert_eq!(read_reminders(&home)["i1"].lifecycle, "completed");
        drop((report, after_branch_move, resumed));
        home.close(execution).unwrap().reclaim();

        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        assert!(obligations(&home).is_empty());
        let after_reopen = catch_up(&home, 30, execution, 4);
        assert_caught_up(&after_reopen);
        assert_eq!(after_reopen.fired(), 0);
        drop(after_reopen);
        home.close(execution).unwrap().reclaim();
    });
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn desktop_temporal_retry_original_key_settles_predecessor_custody_before_next_pump() {
    let dir = unique_temp_dir("retry-original-key-predecessor-settle");
    let (fire_log, auth) = (ReminderFireLog::default(), AuthControl::valid());
    with_leased_request(512 * 1024 * 1024, None, |execution| {
        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        mutate_reminder(&home, active_reminder("i1", 1, 10), 1);
        home.runtime().runtime().lose_next_commit_answer_for_test();
        let report = catch_up(&home, 15, execution, 2);
        assert_eq!(
            report.status(),
            &WorthQueryTemporalCatchUpStatus::BlockedByRetainedDecision
        );
        assert_eq!(report.indeterminate(), 1);
        let original = obligations(&home)[0].clone();

        mutate_reminder(&home, active_reminder("i2", 1, 50), 2);
        let binding = installed(&home);
        let mut port = select_current_branch(&home, execution)
            .conditional_clock(execution, binding.clock_handle())
            .unwrap();
        assert_eq!(managed_clock_count(&home), 2);

        home.runtime()
            .runtime()
            .refuse_next_conditional_decision_publication_for_test();
        assert_retry_published(&mut port, execution, &original);
        assert_eq!(obligations(&home).len(), 1);
        assert_eq!(managed_clock_count(&home), 2);

        assert_retry_published(&mut port, execution, &original);
        drop(port);
        drop(binding);
        assert!(obligations(&home).is_empty());
        assert_eq!(managed_clock_count(&home), 1);

        let next_pump = catch_up(&home, 20, execution, 4);
        assert_caught_up(&next_pump);
        assert_eq!(next_pump.fired(), 0);
        assert!(obligations(&home).is_empty());
        assert_eq!(managed_clock_count(&home), 1);
        assert_eq!(read_reminders(&home)["i1"].lifecycle, "completed");
        assert_eq!(fire_log.snapshot().len(), 1);
        drop((report, next_pump));
        home.close(execution).unwrap().reclaim();
    });
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn desktop_temporal_restored_invalidations_carry_across_same_branch_successor() {
    let dir = unique_temp_dir("restored-invalidations-successor");
    let (fire_log, auth, controls) = (
        ReminderFireLog::default(),
        AuthControl::valid(),
        InvokerControls::default(),
    );
    with_leased_request(512 * 1024 * 1024, None, |execution| {
        let home = open_directory_home_configured(
            dir.clone(),
            fire_log.clone(),
            auth.clone(),
            controls.clone(),
            None,
            execution,
        );
        mutate_reminder(&home, active_reminder("i1", 1, 10), 1);
        let initial = catch_up(&home, 5, execution, 2);
        assert_caught_up(&initial);

        controls.set_next_gate_write(Some("gate-restored-carry".to_owned()));
        mutate_reminder(&home, active_reminder("i2", 1, 50), 2);
        controls.set_next_gate_write(None);

        home.runtime().runtime().lose_next_commit_answer_for_test();
        home.runtime()
            .runtime()
            .refuse_next_conditional_decision_publication_for_test();
        let failed = catch_up(&home, 15, execution, 2);
        assert!(matches!(
            failed.status(),
            WorthQueryTemporalCatchUpStatus::ObservationFailed(_)
        ));
        assert!(failed.granular_invalidation_batches().is_empty());

        let mut streamed_batches = Vec::new();
        let recovered = catch_up_observing(
            &home,
            WorthQueryClockCoordinate::from_nanoseconds(20),
            execution,
            WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            |batch| streamed_batches.push(batch),
        )
        .unwrap();
        assert_caught_up(&recovered);
        let installation = home.primary_graph().granular_invalidation_installation();
        let delivered_count: usize = streamed_batches
            .iter()
            .map(|b| b.bridge_deliveries().len())
            .sum();
        let saw_refresh_all = streamed_batches
            .iter()
            .any(|b| b.coverage() == WorthQueryGranularInvalidationCoverage::RefreshAll);
        assert!(delivered_count > 0 || saw_refresh_all);
        assert!(streamed_batches
            .iter()
            .all(|b| installation.admits_batch(b)));

        let original = obligations(&home)[0].clone();
        let binding = installed(&home);
        let mut port = select_current_branch(&home, execution)
            .conditional_clock(execution, binding.clock_handle())
            .unwrap();
        assert_retry_published(&mut port, execution, &original);
        drop(port);
        drop((binding, installation, initial, failed, recovered));
        drop(streamed_batches);
        home.close(execution).unwrap().reclaim();
    });
    let _ = std::fs::remove_dir_all(&dir);
}

fn trigger_active_indeterminate<'a>(
    home: &'a ReminderHome,
    binding: &'a super::program::InstalledDesktopReminder,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    advance_ns: u64,
    expected_total_indeterminate: usize,
) -> (ReminderPort<'a>, Obligation) {
    binding
        .desktop_clock()
        .advance_to(WorthQueryClockCoordinate::from_nanoseconds(advance_ns))
        .unwrap();
    let mut port = select_current_branch(home, execution)
        .conditional_clock(execution, binding.clock_handle())
        .unwrap();
    home.runtime().runtime().lose_next_commit_answer_for_test();
    let WorthQueryConditionalClockObservationOutcome::Accepted(obs) = port.observe(execution)
    else {
        panic!("active observation must accept");
    };
    assert_eq!(
        obs.indeterminate_operation_count(),
        expected_total_indeterminate
    );
    drop(obs);
    let obs_list = obligations(home);
    assert_eq!(obs_list.len(), 1);
    (port, obs_list[0].clone())
}

#[test]
fn desktop_temporal_active_wake_reobserve_honors_refused_and_confirmed_finish() {
    let dir = unique_temp_dir("active-wake-refused-and-confirmed-finish");
    let (fire_log, auth) = (ReminderFireLog::default(), AuthControl::valid());
    with_leased_request(512 * 1024 * 1024, None, |execution| {
        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        mutate_reminder(&home, active_reminder("i1", 1, 10), 1);
        mutate_reminder(&home, active_reminder("i2", 1, 20), 2);
        let binding = installed(&home);

        let (mut port, ob1) = trigger_active_indeterminate(&home, &binding, execution, 15, 1);
        home.runtime()
            .runtime()
            .refuse_next_conditional_decision_publication_for_test();
        assert_retry_published(&mut port, execution, &ob1);
        assert_eq!(obligations(&home).len(), 1);
        home.runtime()
            .runtime()
            .refuse_next_conditional_decision_publication_for_test();
        assert!(matches!(
            port.observe(execution),
            WorthQueryConditionalClockObservationOutcome::Failed(_)
        ));
        assert_eq!(obligations(&home).len(), 1);
        let WorthQueryConditionalClockObservationOutcome::Accepted(settled) =
            port.observe(execution)
        else {
            panic!("active re-observation after refused Finish must accept");
        };
        assert_eq!(settled.already_committed_operation_count(), 1);
        assert!(obligations(&home).is_empty());
        drop((settled, port));

        let (mut port, ob2) = trigger_active_indeterminate(&home, &binding, execution, 25, 2);
        assert_retry_published(&mut port, execution, &ob2);
        assert!(obligations(&home).is_empty());
        let WorthQueryConditionalClockObservationOutcome::Accepted(reobserved) =
            port.observe(execution)
        else {
            panic!("active re-observation after confirmed Finish must accept");
        };
        assert_eq!(reobserved.already_committed_operation_count(), 2);
        assert!(obligations(&home).is_empty());
        drop((reobserved, port));
        drop(binding);
        home.close(execution).unwrap().reclaim();
    });
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn desktop_temporal_graceful_close_after_active_wake_finish_without_reobserve() {
    for refuse_first_finish in [false, true] {
        let dir = unique_temp_dir(if refuse_first_finish {
            "graceful-close-refused-finish"
        } else {
            "graceful-close-confirmed-finish"
        });
        let (fire_log, auth) = (ReminderFireLog::default(), AuthControl::valid());
        with_leased_request(512 * 1024 * 1024, None, |execution| {
            let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
            mutate_reminder(&home, active_reminder("i1", 1, 10), 1);
            let binding = installed(&home);
            let (mut port, ob) = trigger_active_indeterminate(&home, &binding, execution, 15, 1);
            if refuse_first_finish {
                home.runtime()
                    .runtime()
                    .refuse_next_conditional_decision_publication_for_test();
            }
            assert_retry_published(&mut port, execution, &ob);
            drop(port);
            drop(binding);
            home.close(execution)
                .expect("graceful close after settled active wake must succeed without reobserve")
                .reclaim();

            let reopened = open_directory_home(dir.clone(), fire_log, auth, execution);
            assert!(obligations(&reopened).is_empty());
            reopened.close(execution).unwrap().reclaim();
        });
        let _ = std::fs::remove_dir_all(&dir);
    }
}
