//! Integration courts for lease-free `ExecutionRequest::serial` budgets and
//! retention-capacity backpressure classification during catch-up (Sol Round 2
//! findings 1 & 2).

use worth_execution::ExecutionRequest;
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
fn desktop_temporal_catch_up_enforces_serial_request_budget_on_admitted_snapshot() {
    let fire_log = ReminderFireLog::default();
    let auth = AuthControl::valid();
    let controls = InvokerControls::default();

    let healthy_policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(128 * 1024 * 1024), None),
    );
    let healthy_serial = worth_execution::SerialRequest::from_policy(
        &healthy_policy,
        worth_execution::CancellationToken::new(),
        None,
    );
    let healthy_exec = ExecutionRequest::serial(&healthy_serial);

    let home = open_memory_home_with_controls(
        fire_log.clone(),
        auth.clone(),
        controls.clone(),
        None,
        &healthy_exec,
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
            &healthy_exec,
            WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
        )
        .unwrap();
    assert_eq!(pre.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
    assert_eq!(pre.fired(), 0);

    installed(&home)
        .desktop_clock()
        .advance_to(WorthQueryClockCoordinate::from_nanoseconds(15))
        .unwrap();

    let tiny_policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            Some(128 * 1024 * 1024),
            Some(1),
        ),
    );
    let tiny_serial = worth_execution::SerialRequest::from_policy(
        &tiny_policy,
        worth_execution::CancellationToken::new(),
        None,
    );
    let tiny_exec = ExecutionRequest::serial(&tiny_serial);
    let exhausted = home
        .pump_temporal::<DesktopReminderConditional>(
            &tiny_exec,
            WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
        )
        .unwrap();
    assert_eq!(
        exhausted.status(),
        &WorthQueryTemporalCatchUpStatus::BudgetExhausted,
        "lease-free ExecutionRequest::serial work ceiling 1 must stop wake commit on already-admitted snapshot: {exhausted:?}"
    );
    assert_eq!(exhausted.fired(), 0);
    assert!(exhausted.remaining());
    assert_eq!(read_reminders(&home)["i1"].fired_count, 0);

    let resumed = home
        .pump_temporal::<DesktopReminderConditional>(
            &healthy_exec,
            WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
        )
        .unwrap();
    assert_eq!(resumed.status(), &WorthQueryTemporalCatchUpStatus::CaughtUp);
    assert_eq!(resumed.fired(), 1);
    assert_eq!(read_reminders(&home)["i1"].lifecycle, "completed");
}

#[test]
fn desktop_temporal_catch_up_maps_retention_capacity_to_backpressured() {
    let fire_log = ReminderFireLog::default();
    let auth = AuthControl::valid();
    let clock = worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system();
    let mut history = WorthQueryBoundedApplicationLimits::DEFAULT_BUDGETS.history;
    history.retained_composite_commits = 2;
    let limits = WorthQueryApplicationLimits::bounded_serial(clock)
        .with_history(history)
        .install()
        .expect("bounded serial history limits install");

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

        let stopped = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(15),
                execution,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .expect("capacity exhaustion must map to Backpressured report, not raw error");
        assert_eq!(
            stopped.status(),
            &WorthQueryTemporalCatchUpStatus::Backpressured,
            "retention/history capacity exhaustion must classify as Backpressured, never BlockedByRetainedDecision or CaughtUp: {stopped:?}"
        );
        assert!(stopped.remaining());
        assert_eq!(stopped.fired(), 0);
        assert_eq!(read_reminders(&home)["i1"].fired_count, 0);
    });
}
