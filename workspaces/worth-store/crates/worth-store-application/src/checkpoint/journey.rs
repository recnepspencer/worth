//! One original invoking context drives the complete caller journey.
use super::owner::{HomeCheckpointAttempt, HomeCheckpointOwner};
use super::{ApplicationHomeCheckpoint as Summary, ApplicationHomeCheckpointDenial as Denial};
use std::sync::Arc;
use worth_execution::{
    ExecutionRequest, ExecutionResourceLease, MapKernelContext, MapKernelStop, WorkCeilingDenial,
};
use worth_store::physical_runtime::ServingPhysicalRuntime;

pub(crate) fn checkpoint(
    serving: &ServingPhysicalRuntime,
    owner: &HomeCheckpointOwner,
    execution: &ExecutionRequest<'_, '_>,
    capture: impl FnOnce() -> Result<
        worth_store_relational::PreparedRelationalSection,
        worth_store_relational::RelationalSectionCheckpointDenial,
    >,
    observer: Option<&super::step_custody::StepObserver<'_>>,
    after_acknowledgement: Option<&(dyn Fn() + Send + Sync)>,
) -> Result<Summary, Denial> {
    if let Some(attempt) = owner.pending() {
        return Err(Denial::RecoveryRequired(attempt.observation()));
    }
    let mut retained = None;
    let mut returned = None;
    let mut caller_driver = None;
    let run = |lease: Option<&ExecutionResourceLease<'_>>| {
        execution
            .run_native_with_checkpoint(|context| {
                returned = Some((|| {
                    let (native, input) =
                        super::preparation::capture(serving, execution, capture)?.into_parts();
                    let (attempt, driver) =
                        owner.install(native).map_err(Denial::RecoveryRequired)?;
                    retained = Some(attempt.clone());
                    caller_driver = Some(driver);
                    super::physical_start::start(serving, &attempt, input, context, lease)?;
                    drive_selection(serving, &attempt, context, observer)?;
                    let result = acknowledge(&attempt, context)?;
                    if let Some(observe) = after_acknowledgement {
                        observe();
                    }
                    super::physical_checkpoint::complete(
                        serving,
                        &attempt,
                        result.generation,
                        context,
                        lease,
                        true,
                    )?;
                    Ok(result)
                })());
            })
            .map(|_| ())
    };
    // Leased in_scope only lends the effective original lease. Serial entry
    // goes straight to the one run, without creating an extra serial scope.
    let scope = if execution.is_leased() {
        execution.in_scope(run).and_then(|result| result)
    } else {
        run(None)
    };
    let result = finish_handoff(owner, retained, scope, returned);
    drop(caller_driver);
    result
}

pub(super) fn recover(
    serving: &ServingPhysicalRuntime,
    owner: &HomeCheckpointOwner,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<Summary, Denial> {
    let attempt = owner.pending().ok_or(Denial::NoPendingCheckpoint)?;
    let driver = attempt
        .try_driver()
        .ok_or_else(|| Denial::RecoveryRequired(attempt.observation()))?;
    let mut returned = None;
    let scope = execution
        .run_native_with_checkpoint(|context| {
            returned = Some((|| {
                attempt.stop_advancement();
                if attempt.observation().selected_generation.is_none() {
                    drive_selection(serving, &attempt, context, None)?;
                }
                let result = acknowledge(&attempt, context)?;
                super::physical_checkpoint::complete(
                    serving,
                    &attempt,
                    result.generation,
                    context,
                    None,
                    false,
                )?;
                Ok(result)
            })());
        })
        .map(|_| ());
    let result = finish_handoff(owner, Some(attempt), scope, returned);
    drop(driver);
    result
}

fn finish_handoff(
    owner: &HomeCheckpointOwner,
    retained: Option<Arc<HomeCheckpointAttempt>>,
    scope: Result<(), WorkCeilingDenial>,
    mut returned: Option<Result<Summary, Denial>>,
) -> Result<Summary, Denial> {
    let result = match scope {
        Ok(()) => returned
            .take()
            .expect("completed journey recorded its description"),
        Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
            if matches!(returned, Some(Err(_))) =>
        {
            if let Some(attempt) = &retained {
                attempt.stop_advancement();
            }
            returned.take().expect("recorded exact refusal")
        }
        Err(cause) => {
            if let Some(attempt) = &retained {
                attempt.stop_advancement();
                return Err(Denial::RecoveryRequired(attempt.observation()));
            }
            return Err(Denial::Execution(cause));
        }
    };
    if let Some(attempt) = retained {
        let observation = attempt.observation();
        let unsettled = attempt
            .physical
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .unsettled()
            || attempt
                .wal
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .pending()
            || (observation.selected_generation.is_some() && !observation.acknowledgement_complete);
        if unsettled {
            attempt.stop_advancement();
            return Err(Denial::RecoveryRequired(attempt.observation()));
        }
        owner.clear(&attempt);
    }
    result
}

pub(super) fn drive_selection(
    serving: &ServingPhysicalRuntime,
    attempt: &Arc<HomeCheckpointAttempt>,
    context: &mut MapKernelContext<'_, '_>,
    observer: Option<&super::step_custody::StepObserver<'_>>,
) -> Result<(), Denial> {
    use super::physical_publication::PublicationProgress;
    if attempt.observation().selected_generation.is_some() {
        return Ok(());
    }
    let records = serving.anchored_records().map_err(Denial::Unavailable)?;
    loop {
        if attempt.observation().selected_generation.is_some() {
            return Ok(());
        }
        context
            .poll()
            .map_err(|stop| Denial::Execution(WorkCeilingDenial::Stopped(stop)))?;
        match attempt.advance_physical(&records, observer) {
            PublicationProgress::Pending => std::thread::sleep(std::time::Duration::from_millis(1)),
            PublicationProgress::Selected(receipt) => {
                attempt.selected(receipt.membership().clone());
                return Ok(());
            }
            PublicationProgress::Failed(failure) => return Err(Denial::Publication(
                worth_store::physical_runtime::AnchoredRecordPublicationFailure::OwnedPublication(
                    Box::new(failure),
                ),
            )),
            PublicationProgress::Stopped(prefix) => {
                return Err(Denial::StoppedBeforeSelection { prefix })
            }
            PublicationProgress::InspectionRequired => {
                return Err(Denial::RecoveryRequired(attempt.observation()))
            }
        }
    }
}

pub(super) fn acknowledge(
    attempt: &HomeCheckpointAttempt,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Summary, Denial> {
    if let Some(result) = *attempt
        .completed
        .lock()
        .unwrap_or_else(|error| error.into_inner())
    {
        return Ok(result);
    }
    let native = attempt
        .native()
        .ok_or_else(|| Denial::RecoveryRequired(attempt.observation()))?;
    native
        .acknowledge_selected(context)
        .map_err(Denial::Native)?;
    let result = Summary {
        covered: native.covered(),
        covered_transitions: native.covered_transitions(),
        generation: attempt
            .observation()
            .selected_generation
            .ok_or_else(|| Denial::RecoveryRequired(attempt.observation()))?,
        bytes: native.bytes().len(),
    };
    attempt.finish(result);
    Ok(result)
}
