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
    primary_graph::{
        WorthQueryConditionalClockObservationOutcome, WorthQueryConditionalExecutionCause,
        WorthQueryConditionalExecutionTerminal,
    },
    temporal_contribution::{
        WorthQueryDesktopTemporalHomeBinding, WorthQueryTemporalCatchUpPolicy,
        WorthQueryTemporalCatchUpStatus,
    },
};

use super::{
    adapters::{AuthControl, InvokerControls, ReminderFireLog},
    program::{
        installed, mutate_reminder, open_directory_home, open_directory_home_with_limits,
        open_memory_home_with_controls, read_reminders, unique_temp_dir,
        DesktopReminderConditional,
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

fn serial_policy(memory_bytes: Option<u64>, work: Option<u64>) -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, memory_bytes, work),
    )
}

#[test]
fn desktop_temporal_catch_up_enforces_serial_request_budget_on_admitted_snapshot() {
    let (fire_log, auth, controls) = (
        ReminderFireLog::default(),
        AuthControl::valid(),
        InvokerControls::default(),
    );
    let healthy_policy = serial_policy(Some(128 * 1024 * 1024), None);
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
    mutate_reminder(&home, active_reminder("i1", 1, 10), 1);
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

    let tiny_policy = serial_policy(Some(128 * 1024 * 1024), Some(1));
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
        mutate_reminder(&home, active_reminder("i1", 1, 10), 1);

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

#[test]
fn desktop_temporal_catch_up_enforces_tighter_installed_placement_and_in_flight_serial_cancel() {
    let clock = worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system();
    let tight_policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            Some(128 * 1024 * 1024),
            Some(1),
        ),
    );
    let tight_limits = WorthQueryApplicationLimits::bounded_serial(clock)
        .with_execution_policy(tight_policy)
        .install()
        .expect("tight installed policy limits");

    // Part A: Seed a due reminder in a Directory Home under healthy limits,
    // then reopen the SAME home under tighter installed World work ceiling (1).
    // That tighter ceiling must govern BOTH a generous leased caller AND a
    // generous lease-free ExecutionRequest::serial caller.
    let dir = unique_temp_dir("tighter-installed-placement");
    with_leased_request(128 * 1024 * 1024, None, |generous_leased| {
        let fire_log = ReminderFireLog::default();
        let auth = AuthControl::valid();
        {
            let seeded =
                open_directory_home(dir.clone(), fire_log.clone(), auth.clone(), generous_leased);
            mutate_reminder(&seeded, active_reminder("i1", 1, 10), 1);
            seeded.close(generous_leased).unwrap().reclaim();
        }
        let home = open_directory_home_with_limits(
            dir.clone(),
            fire_log.clone(),
            auth.clone(),
            Some(tight_limits),
            generous_leased,
        );
        let leased_stop = home
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(15),
                generous_leased,
                WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
            )
            .unwrap();
        assert_eq!(
            leased_stop.status(),
            &WorthQueryTemporalCatchUpStatus::BudgetExhausted,
            "tighter installed placement ceiling must govern generous leased caller: {leased_stop:?}"
        );
        assert_eq!(leased_stop.fired(), 0);
        assert!(leased_stop.remaining());

        let generous_serial_policy = ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(128 * 1024 * 1024), None),
        );
        let generous_serial = worth_execution::SerialRequest::from_policy(
            &generous_serial_policy,
            worth_execution::CancellationToken::new(),
            None,
        );
        let serial_stop = home
            .pump_temporal::<DesktopReminderConditional>(
                &ExecutionRequest::serial(&generous_serial),
                WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
            )
            .unwrap();
        assert_eq!(
            serial_stop.status(),
            &WorthQueryTemporalCatchUpStatus::BudgetExhausted,
            "tighter installed placement ceiling must govern generous serial caller: {serial_stop:?}"
        );
        assert_eq!(serial_stop.fired(), 0);
        assert!(serial_stop.remaining());
        drop((leased_stop, serial_stop, home));

        // Reopening under healthy limits proves neither work-1 call fired i1
        // and the durable reminder fires cleanly once under healthy ceilings.
        let reopened = open_directory_home(dir.clone(), fire_log, auth, generous_leased);
        let caught_up = reopened
            .catch_up_temporal::<DesktopReminderConditional>(
                WorthQueryClockCoordinate::from_nanoseconds(15),
                generous_leased,
                WorthQueryTemporalCatchUpPolicy::bounded(4).unwrap(),
            )
            .unwrap();
        assert_eq!(
            caught_up.status(),
            &WorthQueryTemporalCatchUpStatus::CaughtUp
        );
        assert_eq!(caught_up.fired(), 1);
        assert_eq!(read_reminders(&reopened)["i1"].lifecycle, "completed");
        drop(caught_up);
        reopened.close(generous_leased).unwrap().reclaim();
    });
    let _ = std::fs::remove_dir_all(&dir);

    // Part B: Under a finite (tighter-than-caller) installed World placement
    // policy, an in-flight cancellation of ExecutionRequest::serial still stops
    // wake commit with Cancelled and resumes cleanly on a fresh request.
    let bounded_clock = worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system();
    let finite_installed_policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            Some(128 * 1024 * 1024),
            Some(10_000_000),
        ),
    );
    let finite_limits = WorthQueryApplicationLimits::bounded_serial(bounded_clock)
        .with_execution_policy(finite_installed_policy)
        .install()
        .expect("finite installed policy limits");
    let healthy_policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(256 * 1024 * 1024), None),
    );
    let healthy_serial = worth_execution::SerialRequest::from_policy(
        &healthy_policy,
        worth_execution::CancellationToken::new(),
        None,
    );
    let healthy_exec = ExecutionRequest::serial(&healthy_serial);
    let controls = InvokerControls::default();
    let home = open_memory_home_with_controls(
        ReminderFireLog::default(),
        AuthControl::valid(),
        controls.clone(),
        Some(finite_limits),
        &healthy_exec,
    );
    mutate_reminder(&home, active_reminder("i1", 1, 10), 1);

    // Pre-existing caller serial memory hold must narrow live execution room
    // and refuse catch-up cleanly with BudgetExhausted (never NestedLeaseMisuse).
    let tight_serial_policy = serial_policy(Some(64 * 1024), None);
    let held_serial = worth_execution::SerialRequest::from_policy(
        &tight_serial_policy,
        worth_execution::CancellationToken::new(),
        None,
    );
    let held_exec = ExecutionRequest::serial(&held_serial);
    let caller_hold = held_exec
        .reserve_retained_memory(62 * 1024)
        .expect("pre-existing serial caller hold")
        .expect("bounded serial reservation");
    let hold_stop = home
        .catch_up_temporal::<DesktopReminderConditional>(
            WorthQueryClockCoordinate::from_nanoseconds(15),
            &held_exec,
            WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
        )
        .unwrap();
    assert_eq!(
        hold_stop.status(),
        &WorthQueryTemporalCatchUpStatus::BudgetExhausted,
        "pre-existing serial caller memory hold must enforce live remaining room: {hold_stop:?}"
    );
    assert_eq!(hold_stop.fired(), 0);
    assert!(hold_stop.remaining());
    drop(caller_hold);

    // In-flight cancellation under a finite tighter-than-caller installed World
    // placement must surface BOTH a typed Operation(ControlStopped) receipt on
    // direct clock observation AND Cancelled on catch_up_temporal, then resume
    // cleanly on a fresh serial request.
    let cancel_source = worth_execution::CancellationSource::new();
    controls.set_cancel_on_project(Some(cancel_source.clone()));
    let cancellable_serial =
        worth_execution::SerialRequest::from_policy(&healthy_policy, cancel_source.token(), None);
    let cancellable_exec = ExecutionRequest::serial(&cancellable_serial);
    let binding = installed(&home);
    let selected = home
        .runtime()
        .runtime()
        .with_execution_request_advancement(healthy_exec, |phase| {
            home.runtime()
                .runtime()
                .on_branch(home.runtime().current_world())
                .select(&phase)
        })
        .unwrap()
        .unwrap();
    let mut port = selected
        .conditional_clock(&healthy_exec, binding.clock_handle())
        .unwrap();
    let WorthQueryConditionalClockObservationOutcome::Accepted(observed) =
        port.observe(&cancellable_exec)
    else {
        panic!("in-flight serial cancellation must return Accepted with ControlStopped terminal");
    };
    assert!(observed.execution_provenance().iter().any(|provenance| {
        provenance.terminal() == WorthQueryConditionalExecutionTerminal::ControlStopped
            && matches!(
                provenance.cause(),
                Some(WorthQueryConditionalExecutionCause::Cancelled) | None
            )
    }));
    drop((observed, port));
    drop(binding);

    let cancelled = home
        .catch_up_temporal::<DesktopReminderConditional>(
            WorthQueryClockCoordinate::from_nanoseconds(15),
            &cancellable_exec,
            WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
        )
        .unwrap();
    assert_eq!(
        cancelled.status(),
        &WorthQueryTemporalCatchUpStatus::Cancelled,
        "in-flight serial cancellation must survive tighter installed placement narrowing: {cancelled:?}"
    );
    assert_eq!(cancelled.fired(), 0);
    assert!(cancelled.remaining());
    assert_eq!(read_reminders(&home)["i1"].fired_count, 0);

    controls.set_cancel_on_project(None);
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
