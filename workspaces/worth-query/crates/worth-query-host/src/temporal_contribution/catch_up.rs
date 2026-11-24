mod classification;
mod types;

use worth_execution::ExecutionRequest;
use worth_query_execution::facade::{
    primary_graph::{
        WorthQueryConditionalClockHandle, WorthQueryConditionalClockObservationOutcome,
        WorthQueryGranularInvalidationDeliveryBatch, WorthQueryPrimaryGraphApplicationRuntime,
    },
    product::WorthQueryProductBranch,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryClockCoordinate, WorthQueryNamedClock,
};

use super::{
    clock::WorthQueryDesktopClockHandle, home_binding::WorthQueryDesktopTemporalHomeBinding,
};
use classification::{
    classify_advancement_stop, classify_branch_stop, classify_clock_port_denial,
    classify_observation_failure, classify_work_ceiling_denial, empty_report, CatchUpAccumulator,
};

pub use types::{
    WorthQueryTemporalCatchUpClockStop, WorthQueryTemporalCatchUpDenial,
    WorthQueryTemporalCatchUpPolicy, WorthQueryTemporalCatchUpReport,
    WorthQueryTemporalCatchUpStatus, MAX_TEMPORAL_CATCH_UP_STEPS,
};

/// Validates that `desktop_clock` shares underlying source state with `binding`,
/// advances it monotonically to `target_now`, and pumps due timed intents under
/// `execution` and `policy`.
#[allow(clippy::too_many_arguments)]
pub fn pump_temporal_catch_up_until<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: WorthQueryProductBranch,
    binding: &Binding,
    desktop_clock: &WorthQueryDesktopClockHandle<Binding::Clock>,
    target_now: WorthQueryClockCoordinate<Binding::Clock>,
    execution: &ExecutionRequest<'_, '_>,
    policy: WorthQueryTemporalCatchUpPolicy,
) -> Result<WorthQueryTemporalCatchUpReport<Binding::Clock>, WorthQueryTemporalCatchUpDenial>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryDesktopTemporalHomeBinding<Schema>,
{
    if !binding.desktop_clock().shares_state_with(desktop_clock) {
        return Err(WorthQueryTemporalCatchUpDenial::ForeignClockHandle);
    }
    if let Err(denial) = execution.consult() {
        return Ok(empty_report(classify_work_ceiling_denial(&denial), true));
    }
    desktop_clock
        .advance_to(target_now)
        .map_err(WorthQueryTemporalCatchUpDenial::ClockAdvance)?;
    pump_temporal_catch_up(runtime, branch, binding.clock_handle(), execution, policy)
}

/// Streaming variant of [`pump_temporal_catch_up_until`] forwarding every
/// non-empty [`WorthQueryGranularInvalidationDeliveryBatch`] to `on_invalidations`
/// immediately with O(1) accumulator memory.
#[allow(clippy::too_many_arguments)]
pub fn pump_temporal_catch_up_until_observing<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: WorthQueryProductBranch,
    binding: &Binding,
    desktop_clock: &WorthQueryDesktopClockHandle<Binding::Clock>,
    target_now: WorthQueryClockCoordinate<Binding::Clock>,
    execution: &ExecutionRequest<'_, '_>,
    policy: WorthQueryTemporalCatchUpPolicy,
    on_invalidations: impl FnMut(WorthQueryGranularInvalidationDeliveryBatch),
) -> Result<WorthQueryTemporalCatchUpReport<Binding::Clock>, WorthQueryTemporalCatchUpDenial>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryDesktopTemporalHomeBinding<Schema>,
{
    if !binding.desktop_clock().shares_state_with(desktop_clock) {
        return Err(WorthQueryTemporalCatchUpDenial::ForeignClockHandle);
    }
    if let Err(denial) = execution.consult() {
        return Ok(empty_report(classify_work_ceiling_denial(&denial), true));
    }
    desktop_clock
        .advance_to(target_now)
        .map_err(WorthQueryTemporalCatchUpDenial::ClockAdvance)?;
    pump_temporal_catch_up_observing(
        runtime,
        branch,
        binding.clock_handle(),
        execution,
        policy,
        on_invalidations,
    )
}

/// Pumps due timed intents from the currently installed named clock source
/// under bounded `execution` (`Work`/`Memory`/`Cancellation`) and `policy`
/// (`max_steps`), retaining any non-empty granular invalidation batches on the
/// returned report.
pub fn pump_temporal_catch_up<Schema, Node, Clock>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: WorthQueryProductBranch,
    clock_handle: &WorthQueryConditionalClockHandle<Schema, Node, Clock>,
    execution: &ExecutionRequest<'_, '_>,
    policy: WorthQueryTemporalCatchUpPolicy,
) -> Result<WorthQueryTemporalCatchUpReport<Clock>, WorthQueryTemporalCatchUpDenial>
where
    Schema: ApplicationSchema + 'static,
    Clock: WorthQueryNamedClock,
{
    pump_loop(
        runtime,
        branch,
        clock_handle,
        execution,
        policy,
        None::<&mut fn(_)>,
    )
}

/// Streaming variant of [`pump_temporal_catch_up`] delivering each step's
/// non-empty granular invalidation batch directly to `on_invalidations`.
pub fn pump_temporal_catch_up_observing<Schema, Node, Clock>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: WorthQueryProductBranch,
    clock_handle: &WorthQueryConditionalClockHandle<Schema, Node, Clock>,
    execution: &ExecutionRequest<'_, '_>,
    policy: WorthQueryTemporalCatchUpPolicy,
    mut on_invalidations: impl FnMut(WorthQueryGranularInvalidationDeliveryBatch),
) -> Result<WorthQueryTemporalCatchUpReport<Clock>, WorthQueryTemporalCatchUpDenial>
where
    Schema: ApplicationSchema + 'static,
    Clock: WorthQueryNamedClock,
{
    pump_loop(
        runtime,
        branch,
        clock_handle,
        execution,
        policy,
        Some(&mut on_invalidations),
    )
}

fn pump_loop<Schema, Node, Clock, Sink>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: WorthQueryProductBranch,
    clock_handle: &WorthQueryConditionalClockHandle<Schema, Node, Clock>,
    execution: &ExecutionRequest<'_, '_>,
    policy: WorthQueryTemporalCatchUpPolicy,
    mut sink: Option<&mut Sink>,
) -> Result<WorthQueryTemporalCatchUpReport<Clock>, WorthQueryTemporalCatchUpDenial>
where
    Schema: ApplicationSchema + 'static,
    Clock: WorthQueryNamedClock,
    Sink: FnMut(WorthQueryGranularInvalidationDeliveryBatch),
{
    let mut state = CatchUpAccumulator::<Clock>::new();
    if let Err(denial) = execution.consult() {
        return Ok(state.finish(classify_work_ceiling_denial(&denial), true));
    }

    for step in 0..policy.max_steps().get() {
        if let Err(denial) = execution.consult() {
            return Ok(state.finish(classify_work_ceiling_denial(&denial), true));
        }
        let selected = match runtime
            .with_funded_host_advancement(execution.allocation_policy(), |phase| {
                runtime.on_branch(branch).select(&phase)
            }) {
            Ok(Ok(selected)) => selected,
            Ok(Err(denial)) => {
                if let Some(status) = classify_branch_stop(&denial) {
                    return Ok(state.finish(status, true));
                }
                return Err(WorthQueryTemporalCatchUpDenial::BranchSelection(denial));
            }
            Err(denial) => {
                if let Some(status) = classify_advancement_stop(&denial) {
                    return Ok(state.finish(status, true));
                }
                return Err(WorthQueryTemporalCatchUpDenial::Advancement(denial));
            }
        };
        let mut port = match selected.conditional_clock(execution, clock_handle) {
            Ok(port) => port,
            Err(denial) => {
                if let Some(status) = classify_clock_port_denial(&denial) {
                    return Ok(state.finish(status, true));
                }
                return Err(WorthQueryTemporalCatchUpDenial::ClockPort(denial));
            }
        };
        match port.observe(execution) {
            WorthQueryConditionalClockObservationOutcome::Accepted(mut receipt)
            | WorthQueryConditionalClockObservationOutcome::Duplicate(mut receipt) => {
                let batch = receipt.take_granular_invalidation_batch();
                if !batch.is_empty() {
                    if let Some(callback) = sink.as_deref_mut() {
                        callback(batch);
                    } else {
                        state.retain_invalidation_batch(batch);
                    }
                }
                let step_outcome = state.record_receipt(&receipt);
                let has_more_work = receipt.due_work_remaining()
                    || receipt.authoritative_work_remaining()
                    || receipt.retained_eligible_wake_count() > 0
                    || receipt.retained_deferred_wake_count() > 0
                    || receipt.retained_failed_wake_count() > 0
                    || step_outcome.retry_product_stale;
                if let Some(stop_status) = step_outcome.stop_status {
                    return Ok(state.finish(stop_status, true));
                }
                if !has_more_work {
                    return Ok(state.finish(WorthQueryTemporalCatchUpStatus::CaughtUp, false));
                }
                if step + 1 >= policy.max_steps().get() {
                    return Ok(
                        state.finish(WorthQueryTemporalCatchUpStatus::StepLimitReached, true)
                    );
                }
            }
            WorthQueryConditionalClockObservationOutcome::Stale => {
                return Ok(state.finish(
                    WorthQueryTemporalCatchUpStatus::ClockStopped(
                        WorthQueryTemporalCatchUpClockStop::Stale,
                    ),
                    true,
                ));
            }
            WorthQueryConditionalClockObservationOutcome::Reordered => {
                return Ok(state.finish(
                    WorthQueryTemporalCatchUpStatus::ClockStopped(
                        WorthQueryTemporalCatchUpClockStop::Reordered,
                    ),
                    true,
                ));
            }
            WorthQueryConditionalClockObservationOutcome::Closed => {
                return Ok(state.finish(
                    WorthQueryTemporalCatchUpStatus::ClockStopped(
                        WorthQueryTemporalCatchUpClockStop::Closed,
                    ),
                    true,
                ));
            }
            WorthQueryConditionalClockObservationOutcome::Failed(failure) => {
                return Ok(state.finish(classify_observation_failure(failure), true));
            }
        }
    }

    Ok(state.finish(WorthQueryTemporalCatchUpStatus::StepLimitReached, true))
}
