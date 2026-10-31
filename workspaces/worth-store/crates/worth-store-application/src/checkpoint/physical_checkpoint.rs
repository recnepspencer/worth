//! Retain and poll the actual optional WAL checkpoint; never block on wait.
use super::{owner::HomeCheckpointAttempt, ApplicationHomeCheckpointDenial as Denial};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::*;

#[derive(Default)]
pub(super) enum HomeWalCheckpoint {
    #[default]
    NotStarted,
    Admitting(PhysicalCheckpointRequest),
    Running(PhysicalCheckpointHandle),
    Settled(PhysicalCheckpointOutcome),
}
impl HomeWalCheckpoint {
    pub(super) fn pending(&self) -> bool {
        matches!(self, Self::Admitting(_) | Self::Running(_))
    }
}

pub(super) fn complete(
    serving: &ServingPhysicalRuntime,
    attempt: &HomeCheckpointAttempt,
    generation: u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    start_if_missing: bool,
) -> Result<(), Denial> {
    let not_started = matches!(
        *attempt
            .wal
            .lock()
            .unwrap_or_else(|error| error.into_inner()),
        HomeWalCheckpoint::NotStarted
    );
    if not_started {
        if !start_if_missing {
            return Ok(());
        }
        let view = context
            .retain_interruption_view(lease)
            .map_err(Denial::InterruptionView)?;
        let mut key = [0_u8; 32];
        key[..8].copy_from_slice(b"WAPHCKP1");
        key[8..16].copy_from_slice(&generation.to_le_bytes());
        let request = PhysicalCheckpointRequest::fuzzy(
            PhysicalCheckpointIdempotencyKey::new(key),
            PhysicalCheckpointDeadline::after_milliseconds(120_000)
                .expect("fixed positive physical policy"),
        );
        *attempt
            .wal
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = HomeWalCheckpoint::Admitting(request);
        install_start(
            attempt,
            serving
                .checkpoints()
                .start_with_interruption_view(request, view)
                .into_raw(),
        )?;
    }
    let admitted = match *attempt
        .wal
        .lock()
        .unwrap_or_else(|error| error.into_inner())
    {
        HomeWalCheckpoint::Admitting(request) => Some(request),
        _ => None,
    };
    if let Some(request) = admitted {
        // Same idempotency rejoin, preserving the accepted attempt's original
        // controls. This path never supplies a refreshed interruption view.
        install_start(attempt, serving.checkpoints().start(request).into_raw())?;
    }
    loop {
        let terminal = {
            let mut wal = attempt
                .wal
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            match &*wal {
                HomeWalCheckpoint::Running(handle) => match handle.poll() {
                    PhysicalCheckpointPoll::Pending(_) => None,
                    PhysicalCheckpointPoll::Terminal(outcome) => {
                        *wal = HomeWalCheckpoint::Settled(outcome.clone());
                        Some(outcome)
                    }
                },
                HomeWalCheckpoint::Settled(outcome) => Some(outcome.clone()),
                HomeWalCheckpoint::NotStarted => return Ok(()),
                HomeWalCheckpoint::Admitting(_) => None,
            }
        };
        if let Some(outcome) = terminal {
            return match outcome {
                PhysicalCheckpointOutcome::Completed(_) => Ok(()),
                outcome @ (PhysicalCheckpointOutcome::ProvenNoEffect(_)
                | PhysicalCheckpointOutcome::Indeterminate(_)) => {
                    Err(Denial::PhysicalCheckpointOutcome(outcome))
                }
            };
        }
        context
            .poll()
            .map_err(|stop| Denial::Execution(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

fn install_start(
    attempt: &HomeCheckpointAttempt,
    outcome: TransitionOutcome<
        PhysicalCheckpointHandle,
        PhysicalCheckpointStartDenial,
        PhysicalCheckpointStartDeferred,
        PhysicalCheckpointStartStale,
        PhysicalCheckpointStartRebindRequired,
        PhysicalCheckpointStartFailure,
    >,
) -> Result<(), Denial> {
    match outcome {
        TransitionOutcome::Success(handle) => {
            *attempt
                .wal
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = HomeWalCheckpoint::Running(handle);
            Ok(())
        }
        other => {
            *attempt
                .wal
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = HomeWalCheckpoint::NotStarted;
            Err(Denial::PhysicalCheckpoint {
                detail: format!(
                    "physical checkpoint start {}",
                    crate::open::physical_admission::unfinished_outcome(&other)
                ),
            })
        }
    }
}

pub(super) fn settle_for_close(
    serving: &ServingPhysicalRuntime,
    attempt: &HomeCheckpointAttempt,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<Result<(), Denial>, worth_execution::WorkCeilingDenial> {
    let admitting = match *attempt
        .wal
        .lock()
        .unwrap_or_else(|error| error.into_inner())
    {
        HomeWalCheckpoint::Admitting(request) => Some(request),
        _ => None,
    };
    if let Some(request) = admitting {
        if let Err(cause) = install_start(attempt, serving.checkpoints().start(request).into_raw())
        {
            return Ok(Err(cause));
        }
    }
    loop {
        let terminal = {
            let mut wal = attempt
                .wal
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            match &*wal {
                HomeWalCheckpoint::NotStarted => return Ok(Ok(())),
                HomeWalCheckpoint::Running(handle) => match handle.poll() {
                    PhysicalCheckpointPoll::Pending(_) => None,
                    PhysicalCheckpointPoll::Terminal(outcome) => {
                        *wal = HomeWalCheckpoint::Settled(outcome.clone());
                        Some(outcome)
                    }
                },
                HomeWalCheckpoint::Settled(outcome) => Some(outcome.clone()),
                HomeWalCheckpoint::Admitting(_) => None,
            }
        };
        if let Some(outcome) = terminal {
            return Ok(match outcome {
                PhysicalCheckpointOutcome::Completed(_) => Ok(()),
                outcome @ (PhysicalCheckpointOutcome::ProvenNoEffect(_)
                | PhysicalCheckpointOutcome::Indeterminate(_)) => {
                    Err(Denial::PhysicalCheckpointOutcome(outcome))
                }
            });
        }
        crate::close::posture::interrupted(execution)?;
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
