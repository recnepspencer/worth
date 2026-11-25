//! Directory Home integration courts for live continuation custody
//! preservation across fresh branch selection and explicit `retry_original_key`
//! settlement (Sol Round 1 blocker 5 & Round 2 finding 4).

use worth_query_host::facade::{
    domain::WorthQueryClockCoordinate,
    primary_graph::{
        WorthQueryConditionalOriginalResolution, WorthQueryGranularInvalidationCoverage,
    },
    temporal_contribution::{
        WorthQueryDesktopTemporalHomeBinding, WorthQueryTemporalCatchUpPolicy,
        WorthQueryTemporalCatchUpStatus,
    },
};

use super::{
    adapters::{AuthControl, InvokerControls, ReminderFireLog},
    program::{
        installed, mutate_reminder, open_directory_home, open_directory_home_configured,
        read_reminders, unique_temp_dir, DesktopReminderConditional,
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

#[test]
fn desktop_temporal_live_continuation_survives_fresh_branch_selection() {
    let dir = unique_temp_dir("live-continuation-branch-advance");
    let fire_log = ReminderFireLog::default();
    let auth = AuthControl::valid();
    with_leased_request(512 * 1024 * 1024, None, |execution| {
        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        mutate_reminder(&home, active_reminder("i1", 1, 10), 1);
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

        mutate_reminder(&home, active_reminder("i2", 1, 50), 2);
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
fn desktop_temporal_retry_original_key_settles_predecessor_custody_before_next_pump() {
    let dir = unique_temp_dir("retry-original-key-predecessor-settle");
    let fire_log = ReminderFireLog::default();
    let auth = AuthControl::valid();
    with_leased_request(512 * 1024 * 1024, None, |execution| {
        let home = open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), execution);
        mutate_reminder(&home, active_reminder("i1", 1, 10), 1);
        // Lose only the answer so World publishes the original commit while
        // the binding retains an OperationIndeterminate continuation.
        home.runtime().runtime().lose_next_commit_answer_for_test();
        let report = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(15),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
            )
            .unwrap();
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
        let mut port = selected
            .conditional_clock(execution, binding.clock_handle())
            .unwrap();
        assert_eq!(
            home.runtime()
                .runtime()
                .inspect_conditional_runtime()
                .managed_clock_count(),
            2,
            "indeterminate predecessor clock must stay retained while obligation is unresolved"
        );

        // First prove a refused durable Finish keeps the durable obligation
        // live AND does not prematurely reclaim the superseded predecessor clock.
        home.runtime()
            .runtime()
            .refuse_next_conditional_decision_publication_for_test();
        let resolved_under_refused_finish = port
            .retry_original_key(execution, original.key, original.intent)
            .expect("key fate resolves even when durable Finish publication is refused");
        assert!(matches!(
            resolved_under_refused_finish,
            WorthQueryConditionalOriginalResolution::Published(_)
        ));
        assert_eq!(
            home.runtime().conditional_decision_obligations().len(),
            1,
            "refused durable Finish must keep the durable obligation live"
        );
        assert_eq!(
            home.runtime()
                .runtime()
                .inspect_conditional_runtime()
                .managed_clock_count(),
            2,
            "refused durable Finish must not prematurely reclaim predecessor clock"
        );

        // Now confirm settlement with healthy execution: durable obligation
        // discharges AND superseded predecessor clock is reclaimed (2 -> 1).
        let resolved = port
            .retry_original_key(execution, original.key, original.intent)
            .expect("retry_original_key resolves published original");
        assert!(
            matches!(
                resolved,
                WorthQueryConditionalOriginalResolution::Published(_)
            ),
            "published original must resolve via retry_original_key: {resolved:?}"
        );
        drop(port);
        drop(binding);
        assert!(
            home.runtime().conditional_decision_obligations().is_empty(),
            "retry_original_key must discharge the durable conditional obligation"
        );
        assert_eq!(
            home.runtime()
                .runtime()
                .inspect_conditional_runtime()
                .managed_clock_count(),
            1,
            "confirmed settlement must reclaim the superseded predecessor managed clock"
        );

        // Pumping again must NOT flush the settled predecessor wake back into
        // a fresh Retain obligation or duplicate-fire i1, and retains 1 clock.
        let next_pump = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(20),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .unwrap();
        assert_eq!(
            next_pump.status(),
            &WorthQueryTemporalCatchUpStatus::CaughtUp
        );
        assert_eq!(next_pump.fired(), 0);
        assert!(
            home.runtime().conditional_decision_obligations().is_empty(),
            "subsequent pump after retry_original_key must not re-record Retain for the settled predecessor wake"
        );
        assert_eq!(
            home.runtime()
                .runtime()
                .inspect_conditional_runtime()
                .managed_clock_count(),
            1,
            "managed clock count must remain 1 after subsequent pump"
        );
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
    let fire_log = ReminderFireLog::default();
    let auth = AuthControl::valid();
    let controls = InvokerControls::default();
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
        let initial = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(5),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
            )
            .unwrap();
        assert_eq!(initial.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);

        // Commit a Signal-watched gate update so B1 consumes its granular
        // invalidation delivery when advancing the commit cursor on the next
        // observation.
        controls.set_next_gate_write(Some("gate-restored-carry".to_owned()));
        mutate_reminder(&home, active_reminder("i2", 1, 50), 2);
        controls.set_next_gate_write(None);

        // Trigger a durable Retain refusal AFTER B1 advanced the commit cursor
        // past the gate commit and released that granular invalidation batch,
        // so `temporal_operation` restores it onto B1 while World publishes
        // i1's non-gate fire commit and advances the branch head.
        home.runtime().runtime().lose_next_commit_answer_for_test();
        home.runtime()
            .runtime()
            .refuse_next_conditional_decision_publication_for_test();
        let failed = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(15),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
            )
            .unwrap();
        assert!(
            matches!(
                failed.status(),
                WorthQueryTemporalCatchUpStatus::ObservationFailed(_)
            ),
            "durable Retain refusal after commit cursor advance must surface ObservationFailed: {failed:?}"
        );
        assert!(
            failed.granular_invalidation_batches().is_empty(),
            "failed observation must restore owed granular invalidations instead of emitting them"
        );

        // Branch head advanced via i1's non-gate commit; selecting same-branch
        // successor B2 inherits the already-advanced commit cursor (past the
        // gate commit) and reads zero new gate deliveries. Only carrying B1's
        // restored invalidation batch into B2 can deliver it now.
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
            &WorthQueryTemporalCatchUpStatus::CaughtUp,
            "successor binding should flush inactive Retain and complete observation cleanly: {recovered:?}"
        );
        let installation = home.primary_graph().granular_invalidation_installation();
        let delivered_count: usize = streamed_batches
            .iter()
            .map(|batch| batch.bridge_deliveries().len())
            .sum();
        let saw_refresh_all = streamed_batches
            .iter()
            .any(|batch| batch.coverage() == WorthQueryGranularInvalidationCoverage::RefreshAll);
        assert!(
            delivered_count > 0 || saw_refresh_all,
            "restored gate granular invalidation must carry across same-branch successor (batches={}, delivered_count={delivered_count}, saw_refresh_all={saw_refresh_all}, report={recovered:?})",
            streamed_batches.len()
        );
        assert!(streamed_batches
            .iter()
            .all(|batch| installation.admits_batch(batch)));
        // Settle the flushed predecessor obligation before closing Directory Home.
        let obligations = home.runtime().conditional_decision_obligations();
        assert_eq!(obligations.len(), 1);
        let original = obligations[0].clone();
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
