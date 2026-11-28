//! Focused unit & Directory Home integration courts + 64-seed DST for desktop
//! temporal contribution and bounded caller-pumped reopen catch-up.

use self::pump::pump as pump_reminders;

mod adapters;
mod adversarial_courts;
mod catchup_dst;
mod courts;
mod custody_courts;
mod invalidation_courts;
mod mismatched_due_court;
mod program;
mod pump;
mod schema;
mod serial_and_capacity_courts;

use worth_execution::{
    CancellationSource, CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest,
};
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    domain::{
        WorthQueryClockCoordinate, WorthQueryClockSourceIdentity, WorthQueryClockTimelineIdentity,
    },
    temporal_contribution::{
        WorthQueryAlwaysReadyTemporalPredicate, WorthQueryDesktopClockAdvanceDenial,
        WorthQueryDesktopClockHandle, WorthQueryDesktopClockSource,
        WorthQueryDesktopTemporalNodeSpec, WorthQueryTemporalCatchUpClockStop,
        WorthQueryTemporalCatchUpDenial, WorthQueryTemporalCatchUpPolicy,
        WorthQueryTemporalCatchUpReport, WorthQueryTemporalCatchUpStatus,
        MAX_TEMPORAL_CATCH_UP_STEPS,
    },
};

use adapters::{AuthControl, DesktopReminderClock, ReminderFireLog};
use program::{installed, open_memory_home};

fn with_leased_request<R>(
    max_memory_bytes: u64,
    work_ceiling: Option<u64>,
    run: impl FnOnce(&ExecutionRequest<'_, '_>) -> R,
) -> R {
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(max_memory_bytes),
                work_ceiling,
            ),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .expect("serial execution lease admits");
    let request = ExecutionRequest::leased(&lease);
    run(&request)
}

fn with_cancelled_request<R>(run: impl FnOnce(&ExecutionRequest<'_, '_>) -> R) -> R {
    let source = CancellationSource::new();
    source.cancel();
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(128 * 1024 * 1024), None),
        ),
        cancellation: source.token(),
        deadline: None,
    })
    .expect("cancelled execution lease admits");
    let request = ExecutionRequest::leased(&lease);
    run(&request)
}

#[test]
fn desktop_temporal_bounds_and_clock_handle_refuse_invalid_ceilings_and_regression() {
    assert!(matches!(
        WorthQueryTemporalCatchUpPolicy::bounded(0),
        Err(WorthQueryTemporalCatchUpDenial::InvalidMaxSteps { requested: 0, .. })
    ));
    assert!(matches!(
        WorthQueryTemporalCatchUpPolicy::bounded(MAX_TEMPORAL_CATCH_UP_STEPS + 1),
        Err(WorthQueryTemporalCatchUpDenial::InvalidMaxSteps { .. })
    ));
    assert!(matches!(
        WorthQueryTemporalCatchUpPolicy::bounded(usize::MAX),
        Err(WorthQueryTemporalCatchUpDenial::InvalidMaxSteps { .. })
    ));
    let (_source, handle) = WorthQueryDesktopClockSource::<DesktopReminderClock>::new(
        WorthQueryClockSourceIdentity::declare("src").unwrap(),
        WorthQueryClockTimelineIdentity::declare("tl").unwrap(),
        WorthQueryClockCoordinate::from_nanoseconds(0),
    );
    handle
        .advance_to(WorthQueryClockCoordinate::from_nanoseconds(10))
        .unwrap();
    assert_eq!(
        handle.advance_to(WorthQueryClockCoordinate::from_nanoseconds(9)),
        Err(WorthQueryDesktopClockAdvanceDenial::ClockRegression {
            current: 10,
            requested: 9,
        })
    );
    handle.close();
    assert_eq!(
        handle.advance_to(WorthQueryClockCoordinate::from_nanoseconds(11)),
        Err(WorthQueryDesktopClockAdvanceDenial::ClockClosed)
    );

    with_leased_request(128 * 1024 * 1024, None, |execution| {
        let home = open_memory_home(ReminderFireLog::default(), AuthControl::valid(), execution);
        installed(&home).desktop_clock().close();
        let stopped = pump_reminders(
            &home,
            execution,
            WorthQueryTemporalCatchUpPolicy::bounded(2).unwrap(),
        )
        .unwrap();
        assert_eq!(
            stopped.status(),
            &WorthQueryTemporalCatchUpStatus::ClockStopped(
                WorthQueryTemporalCatchUpClockStop::Closed
            )
        );
    });
}

#[test]
fn desktop_temporal_public_types_preserve_auto_traits_and_const_bounds() {
    fn assert_unwind_safe<T: Send + Sync + std::panic::UnwindSafe + std::panic::RefUnwindSafe>() {}
    fn assert_send_sync<T: Send + Sync>() {}

    assert_unwind_safe::<WorthQueryDesktopClockSource<DesktopReminderClock>>();
    assert_unwind_safe::<WorthQueryDesktopClockHandle<DesktopReminderClock>>();
    assert_unwind_safe::<WorthQueryDesktopClockAdvanceDenial>();
    assert_unwind_safe::<WorthQueryDesktopTemporalNodeSpec>();
    assert_unwind_safe::<WorthQueryAlwaysReadyTemporalPredicate>();
    assert_unwind_safe::<WorthQueryTemporalCatchUpPolicy>();
    assert_unwind_safe::<WorthQueryTemporalCatchUpClockStop>();

    assert_send_sync::<WorthQueryTemporalCatchUpStatus>();
    assert_send_sync::<WorthQueryTemporalCatchUpDenial>();
    assert_send_sync::<WorthQueryTemporalCatchUpReport<DesktopReminderClock>>();

    const MAX: usize = MAX_TEMPORAL_CATCH_UP_STEPS;
    assert_eq!(
        WorthQueryTemporalCatchUpPolicy::bounded(MAX)
            .unwrap()
            .max_steps()
            .get(),
        MAX
    );
}
