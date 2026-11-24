//! Directory Home integration courts for live continuation custody
//! preservation across fresh branch selection and explicit `retry_original_key`
//! settlement (Sol Round 1 blocker 5 & Round 2 finding 4).

use worth_query_host::facade::{
    domain::WorthQueryClockCoordinate,
    primary_graph::WorthQueryConditionalOriginalResolution,
    temporal_contribution::{
        WorthQueryDesktopTemporalHomeBinding, WorthQueryTemporalCatchUpPolicy,
        WorthQueryTemporalCatchUpStatus,
    },
};

use super::{
    adapters::{AuthControl, ReminderFireLog},
    program::{
        installed, mutate_reminder, open_directory_home, read_reminders, unique_temp_dir,
        DesktopReminderConditional,
    },
    schema::MutateReminderInput,
    with_leased_request,
};

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
fn desktop_temporal_retry_original_key_settles_predecessor_custody_before_next_pump() {
    let dir = unique_temp_dir("retry-original-key-predecessor-settle");
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

        // Pumping again must NOT flush the settled predecessor wake back into
        // a fresh Retain obligation or duplicate-fire i1.
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
        assert_eq!(read_reminders(&home)["i1"].lifecycle, "completed");
        assert_eq!(fire_log.snapshot().len(), 1);
        drop((report, next_pump));
        home.close(execution).unwrap().reclaim();
    });
    let _ = std::fs::remove_dir_all(&dir);
}
