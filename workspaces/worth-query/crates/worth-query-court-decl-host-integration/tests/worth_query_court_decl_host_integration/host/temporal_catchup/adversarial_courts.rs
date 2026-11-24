//! Adversarial integration courts for capacity backpressure, non-authorization
//! terminal refusal, and caller-funded budget & cancellation enforcement
//! (Sol Round 1 blockers 1 & 2).

use worth_execution::{CancellationSource, ExecutionRequest, ExecutionResourceLease, LeaseRequest};
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    application_installation::{WorthQueryApplicationLimits, WorthQueryBoundedApplicationLimits},
    domain::WorthQueryClockCoordinate,
    temporal_contribution::{WorthQueryTemporalCatchUpPolicy, WorthQueryTemporalCatchUpStatus},
};

use super::{
    adapters::{AuthControl, InvokerControls, ReminderFireLog},
    program::{
        installed, mutate_reminder, open_memory_home_with_controls, read_reminders,
        DesktopReminderConditional,
    },
    schema::MutateReminderInput,
    with_leased_request,
};

#[test]
fn desktop_temporal_catch_up_distinguishes_unobserved_write_from_authorization_refusal() {
    let fire_log = ReminderFireLog::default();
    let auth = AuthControl::valid();
    let controls = InvokerControls::default();
    controls.set_force_unobserved_write(true);

    with_leased_request(128 * 1024 * 1024, None, |execution| {
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
                interval: 5,
                max_fires: 2,
                fired_count: 0,
                lifecycle: "active".to_owned(),
            },
            1,
        );
        let report = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(15),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .unwrap();
        assert_eq!(
            report.status(),
            &WorthQueryTemporalCatchUpStatus::OperationRefused,
            "UnobservedWrite must classify as OperationRefused, never AuthorizationRefused: {report:?}"
        );
        assert_eq!(report.fired(), 0);
        assert!(report.remaining());
        assert_eq!(read_reminders(&home)["i1"].revision, 1);
        assert_eq!(read_reminders(&home)["i1"].fired_count, 0);
    });
}

#[test]
fn desktop_temporal_catch_up_reports_backpressured_never_caught_up_on_capacity_deferral() {
    let fire_log = ReminderFireLog::default();
    let auth = AuthControl::valid();
    let clock = worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system();
    let mut budgets = WorthQueryBoundedApplicationLimits::DEFAULT_BUDGETS;
    budgets.observations.active_observations = 4;
    let limits = WorthQueryApplicationLimits::bounded_serial(clock)
        .with_observations(budgets.observations)
        .install()
        .expect("bounded serial limits install");

    with_leased_request(128 * 1024 * 1024, None, |execution| {
        let home = open_memory_home_with_controls(
            fire_log.clone(),
            auth.clone(),
            InvokerControls::default(),
            Some(limits),
            execution,
        );
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
        let mut pinned = Vec::new();
        for _ in 0..4 {
            let branch = home.runtime().current_world();
            match home
                .runtime()
                .with_host_advancement(|phase| home.runtime().on_branch(branch).select(&phase))
            {
                Ok(Ok(sel)) => pinned.push(sel),
                _ => break,
            }
        }
        assert!(!pinned.is_empty());
        let stopped = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(15),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
            )
            .unwrap();
        assert_eq!(
            stopped.status(),
            &WorthQueryTemporalCatchUpStatus::Backpressured,
            "capacity exhaustion must report Backpressured, never CaughtUp: {stopped:?}"
        );
        assert!(stopped.remaining());
        assert_eq!(stopped.fired(), 0);
        drop(pinned);
        assert_eq!(read_reminders(&home)["i1"].fired_count, 0);

        let resumed = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(15),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .unwrap();
        assert_eq!(resumed.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
        assert_eq!(resumed.fired(), 1);
        assert!(!resumed.remaining());
    });
}

#[test]
fn desktop_temporal_catch_up_enforces_caller_budget_and_mid_step_cancel_on_admitted_snapshot() {
    let fire_log = ReminderFireLog::default();
    let auth = AuthControl::valid();
    let controls = InvokerControls::default();

    with_leased_request(128 * 1024 * 1024, None, |execution| {
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
                interval: 0,
                max_fires: 1,
                fired_count: 0,
                lifecycle: "active".to_owned(),
            },
            1,
        );
        let pre = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(5),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
            )
            .unwrap();
        assert_eq!(pre.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
        assert_eq!(pre.fired(), 0);

        installed(&home)
            .desktop_clock()
            .advance_to(WorthQueryClockCoordinate::from_nanoseconds(15))
            .unwrap();
        with_leased_request(128 * 1024 * 1024, Some(5), |tiny_exec| {
            let exhausted = home
                .pump_temporal::<DesktopReminderConditional>(
                    tiny_exec,
                    WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
                )
                .unwrap();
            assert_eq!(
                exhausted.status(),
                &WorthQueryTemporalCatchUpStatus::BudgetExhausted,
                "tiny positive caller budget must stop wake commit on already-admitted snapshot: {exhausted:?}"
            );
            assert_eq!(exhausted.fired(), 0);
            assert!(exhausted.remaining());
        });
        assert_eq!(read_reminders(&home)["i1"].fired_count, 0);

        let cancel_source = CancellationSource::new();
        controls.set_cancel_on_project(Some(cancel_source.clone()));
        let lease = ExecutionResourceLease::serial(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(128 * 1024 * 1024), None),
            ),
            cancellation: cancel_source.token(),
            deadline: None,
        })
        .unwrap();
        let cancel_exec = ExecutionRequest::leased(&lease);
        let cancelled = home
            .pump_temporal::<DesktopReminderConditional>(
                &cancel_exec,
                WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
            )
            .unwrap();
        assert_eq!(
            cancelled.status(),
            &WorthQueryTemporalCatchUpStatus::Cancelled,
            "post-consult cancellation must stop wake commit on already-admitted snapshot: {cancelled:?}"
        );
        assert_eq!(cancelled.fired(), 0);
        assert!(cancelled.remaining());
        assert_eq!(read_reminders(&home)["i1"].fired_count, 0);

        controls.set_cancel_on_project(None);
        let resumed = home
            .pump_temporal::<DesktopReminderConditional>(
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .unwrap();
        assert_eq!(resumed.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
        assert_eq!(resumed.fired(), 1);
        assert_eq!(read_reminders(&home)["i1"].lifecycle, "completed");
    });
}
