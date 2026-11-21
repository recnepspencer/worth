//! Perform a recovered original's outstanding Signal fork exactly once, at its
//! recorded source point and under its original association, then admit the
//! fork from Signal's own inventory as reopen would. World's recovered creation
//! custody issues the grant; Signal refuses any source that has moved.
use super::*;
use crate::lifecycle::signal_services::RuntimeWorldSignalServices;
use crate::publication::durable_signal::{
    WorldDurableSignalBacking, WorldDurableSignalProgress as Progress,
};
use crate::recovery::{
    RecoveredCreationCustody, RecoveredCreationFate, RecoveredCreationForks,
    RecoveredSignalForkPhase as Phase, RecoveredWorldCreationRecord,
    RuntimeWorldRecoveryDenial as Denial,
};
use std::ops::ControlFlow;
use worth_signal::facade::durability::SignalReadmittedSourcePoint;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// On the fork's durable commit the original holds every required fork
    /// and continues exactly like one recovered with both forks durable.
    pub(super) fn complete_recovered_signal_fork(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        record: &RecoveredWorldCreationRecord,
        custody: &mut RecoveredCreationCustody,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
    ) -> Result<(), Denial> {
        let RecoveredCreationFate::CompleteSignalFork(completion) = &mut custody.fate else {
            return Ok(());
        };
        let RuntimeWorldSignalServices::Authored(attachment) = &self.state.signal else {
            unreachable!("a Home World owns an authored Signal attachment")
        };
        if let Phase::Unbegun = completion.phase {
            let (prepared, backing) = prepare_signal_fork(attachment, record, execution)?;
            // From here a lost fate must never begin a second fork.
            completion.phase = Phase::ReopenRequired;
            let task = prepared
                .begin_recovered(
                    *execution,
                    record,
                    &attachment.issuer,
                    cancellation.signal_token(),
                )
                .map_err(|_| Denial::ReopenRequired)?;
            completion.phase = Phase::Driving(backing.retain(task));
        }
        let Phase::Driving(task) = &completion.phase else {
            return Err(Denial::ReopenRequired);
        };
        match task.drive(execution) {
            Progress::Committed(_) => {}
            Progress::Pending => return Err(Denial::Busy),
            Progress::ProvenNoEffect | Progress::Indeterminate => {
                completion.phase = Phase::ReopenRequired;
                return Err(Denial::ReopenRequired);
            }
        }
        let signal = readmit_signal_fork(attachment, record, execution)?;
        let RecoveredCreationFate::CompleteSignalFork(completion) =
            std::mem::replace(&mut custody.fate, RecoveredCreationFate::Release)
        else {
            unreachable!("the same outstanding fork")
        };
        custody.fate = RecoveredCreationFate::Continue(RecoveredCreationForks {
            native: Some(completion.native),
            signal: Some(signal),
        });
        Ok(())
    }
}

/// Signal readmits exactly the recorded source point; a moved source refuses
/// before any effect, so the fork never takes a different basis.
fn prepare_signal_fork<D, I, E, Ctx, T>(
    attachment: &crate::lifecycle::RuntimeWorldAuthoredSignalAttachment<D, I, E, Ctx, T>,
    record: &RecoveredWorldCreationRecord,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<
    (
        crate::lifecycle::PreparedWorldSignalFork,
        WorldDurableSignalBacking,
    ),
    Denial,
>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let crate::branch::SignalBranchCreationPlan::ForkExact { target } = &record.intent.signal
    else {
        unreachable!("only a planned Signal fork is outstanding")
    };
    let source = attachment
        .source_port()
        .readmit_descriptor(record.source_signal.descriptor())
        .map_err(|_| Denial::OriginalSourceMismatch)?;
    let backing = WorldDurableSignalBacking::reserve(execution)
        .map_err(|_| Denial::OutputCapacityExhausted)?;
    let prepared = attachment
        .prepare_branch_fork(
            execution,
            &source,
            target.clone(),
            record.intent.association.clone(),
        )
        .map_err(|_| Denial::OriginalSourceUnavailable)?;
    Ok((prepared, backing))
}

/// The committed fork, found and readmitted through Signal's own inventory.
fn readmit_signal_fork<D, I, E, Ctx, T>(
    attachment: &crate::lifecycle::RuntimeWorldAuthoredSignalAttachment<D, I, E, Ctx, T>,
    record: &RecoveredWorldCreationRecord,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<Arc<SignalReadmittedSourcePoint>, Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let port = attachment.source_port();
    let found = port
        .inspect_associated_advances(execution, |effect, point| {
            if effect.association() == Some(&record.intent.association) {
                ControlFlow::Break(port.readmit_source_point(point, execution))
            } else {
                ControlFlow::Continue(())
            }
        })
        .map_err(|_| Denial::OriginalSourceUnavailable)?;
    match found {
        ControlFlow::Break(Ok(point)) => Ok(Arc::new(point)),
        ControlFlow::Break(Err(_)) | ControlFlow::Continue(()) => {
            Err(Denial::OriginalSourceUnavailable)
        }
    }
}
