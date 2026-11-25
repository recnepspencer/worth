//! Directory Home integration courts for live continuation custody
//! preservation across fresh branch selection and explicit `retry_original_key`
//! settlement (Sol Round 1 blocker 5 & Round 2 finding 4).

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
        read_reminders, unique_temp_dir, DesktopReminderConditional, ReminderHome,
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
    home.catch_up_temporal::<DesktopReminderConditional>(
        WorthQueryClockCoordinate::from_nanoseconds(ns),
        execution,
        WorthQueryTemporalCatchUpPolicy::bounded(max_steps).unwrap(),
    )
    .unwrap()
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
        let obligations_before = home.runtime().conditional_decision_obligations();
        assert_eq!(obligations_before.len(), 1);

        mutate_reminder(&home, active_reminder("i2", 1, 50), 2);
        auth.set_valid(false);
        let after_branch_move = catch_up(&home, 16, execution, 2);
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
        let resumed = catch_up(&home, 20, execution, 4);
        assert_eq!(resumed.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
        assert_eq!(resumed.fired(), 1);
        assert!(home.runtime().conditional_decision_obligations().is_empty());
        assert_eq!(read_reminders(&home)["i1"].lifecycle, "completed");
        drop((report, after_branch_move, resumed));
        home.close(execution).unwrap().reclaim();

        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        assert!(home.runtime().conditional_decision_obligations().is_empty());
        let after_reopen = catch_up(&home, 30, execution, 4);
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
        let obligations = home.runtime().conditional_decision_obligations();
        assert_eq!(obligations.len(), 1);
        let original = obligations[0].clone();

        // Advance branch head so the indeterminate binding moves into
        // inactive predecessor custody.
        mutate_reminder(&home, active_reminder("i2", 1, 50), 2);
        let binding = installed(&home);
        let selected = select_current_branch(&home, execution);
        let mut port = selected
            .conditional_clock(execution, binding.clock_handle())
            .unwrap();
        assert_eq!(managed_clock_count(&home), 2);

        // Refused durable Finish keeps both obligation and superseded clock live.
        home.runtime()
            .runtime()
            .refuse_next_conditional_decision_publication_for_test();
        assert!(matches!(
            port.retry_original_key(execution, original.key, original.intent)
                .unwrap(),
            WorthQueryConditionalOriginalResolution::Published(_)
        ));
        assert_eq!(home.runtime().conditional_decision_obligations().len(), 1);
        assert_eq!(managed_clock_count(&home), 2);

        // Confirmed settlement discharges obligation and reclaims predecessor (2 -> 1).
        assert!(matches!(
            port.retry_original_key(execution, original.key, original.intent)
                .unwrap(),
            WorthQueryConditionalOriginalResolution::Published(_)
        ));
        drop(port);
        drop(binding);
        assert!(home.runtime().conditional_decision_obligations().is_empty());
        assert_eq!(managed_clock_count(&home), 1);

        let next_pump = catch_up(&home, 20, execution, 4);
        assert_eq!(
            next_pump.status(),
            &WorthQueryTemporalCatchUpStatus::CaughtUp
        );
        assert_eq!(next_pump.fired(), 0);
        assert!(home.runtime().conditional_decision_obligations().is_empty());
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
        assert_eq!(initial.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);

        controls.set_next_gate_write(Some("gate-restored-carry".to_owned()));
        mutate_reminder(&home, active_reminder("i2", 1, 50), 2);
        controls.set_next_gate_write(None);

        // Trigger durable Retain refusal AFTER commit cursor advances past the
        // gate commit, so `temporal_operation` restores the granular batch onto
        // B1 while World publishes i1's non-gate fire commit.
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
        let recovered = home
            .catch_up_temporal_observing::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(20),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
                |batch| streamed_batches.push(batch),
            )
            .unwrap();
        assert_eq!(
            recovered.status(),
            &WorthQueryTemporalCatchUpStatus::CaughtUp
        );
        let installation = home.primary_graph().granular_invalidation_installation();
        let delivered_count: usize = streamed_batches
            .iter()
            .map(|batch| batch.bridge_deliveries().len())
            .sum();
        let saw_refresh_all = streamed_batches
            .iter()
            .any(|batch| batch.coverage() == WorthQueryGranularInvalidationCoverage::RefreshAll);
        assert!(delivered_count > 0 || saw_refresh_all);
        assert!(streamed_batches
            .iter()
            .all(|batch| installation.admits_batch(batch)));

        let original = home.runtime().conditional_decision_obligations()[0].clone();
        let binding = installed(&home);
        let selected = select_current_branch(&home, execution);
        let mut port = selected
            .conditional_clock(execution, binding.clock_handle())
            .unwrap();
        assert!(matches!(
            port.retry_original_key(execution, original.key, original.intent)
                .unwrap(),
            WorthQueryConditionalOriginalResolution::Published(_)
        ));
        drop(port);
        drop((
            binding,
            installation,
            initial,
            failed,
            recovered,
            streamed_batches,
        ));
        home.close(execution).unwrap().reclaim();
    });
    let _ = std::fs::remove_dir_all(&dir);
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
        binding
            .desktop_clock()
            .advance_to(WorthQueryClockCoordinate::from_nanoseconds(15))
            .unwrap();
        let selected = select_current_branch(&home, execution);
        let mut port = selected
            .conditional_clock(execution, binding.clock_handle())
            .unwrap();

        // Case A: Active wake + refused Finish -> re-observing on the SAME
        // retained port retries Finish (never Retain) and clears obligations.
        home.runtime().runtime().lose_next_commit_answer_for_test();
        let WorthQueryConditionalClockObservationOutcome::Accepted(first) = port.observe(execution)
        else {
            panic!("initial active observation must accept");
        };
        assert_eq!(first.indeterminate_operation_count(), 1);
        drop(first);
        let ob1 = home.runtime().conditional_decision_obligations();
        assert_eq!(ob1.len(), 1);
        home.runtime()
            .runtime()
            .refuse_next_conditional_decision_publication_for_test();
        assert!(matches!(
            port.retry_original_key(execution, ob1[0].key, ob1[0].intent)
                .unwrap(),
            WorthQueryConditionalOriginalResolution::Published(_)
        ));
        assert_eq!(home.runtime().conditional_decision_obligations().len(), 1);
        home.runtime()
            .runtime()
            .refuse_next_conditional_decision_publication_for_test();
        assert!(matches!(
            port.observe(execution),
            WorthQueryConditionalClockObservationOutcome::Failed(_)
        ));
        assert_eq!(home.runtime().conditional_decision_obligations().len(), 1);
        let WorthQueryConditionalClockObservationOutcome::Accepted(settled) =
            port.observe(execution)
        else {
            panic!("active re-observation after refused Finish must retry Finish and accept");
        };
        assert_eq!(settled.already_committed_operation_count(), 1);
        assert!(home.runtime().conditional_decision_obligations().is_empty());
        drop((settled, port));

        // Case B: Active wake + confirmed Finish -> re-observing on the SAME
        // retained port must NEVER republish Retain.
        binding
            .desktop_clock()
            .advance_to(WorthQueryClockCoordinate::from_nanoseconds(25))
            .unwrap();
        let selected = select_current_branch(&home, execution);
        let mut port = selected
            .conditional_clock(execution, binding.clock_handle())
            .unwrap();
        home.runtime().runtime().lose_next_commit_answer_for_test();
        let WorthQueryConditionalClockObservationOutcome::Accepted(second) =
            port.observe(execution)
        else {
            panic!("second active observation must accept");
        };
        assert_eq!(second.indeterminate_operation_count(), 2);
        drop(second);
        let ob2 = home.runtime().conditional_decision_obligations();
        assert_eq!(ob2.len(), 1);
        assert!(matches!(
            port.retry_original_key(execution, ob2[0].key, ob2[0].intent)
                .unwrap(),
            WorthQueryConditionalOriginalResolution::Published(_)
        ));
        assert!(home.runtime().conditional_decision_obligations().is_empty());
        let WorthQueryConditionalClockObservationOutcome::Accepted(reobserved) =
            port.observe(execution)
        else {
            panic!("active re-observation after confirmed Finish must accept without Retain");
        };
        assert_eq!(reobserved.already_committed_operation_count(), 2);
        assert!(home.runtime().conditional_decision_obligations().is_empty());
        drop((reobserved, port));
        drop(binding);
        home.close(execution).unwrap().reclaim();
    });
    let _ = std::fs::remove_dir_all(&dir);
}
