//! Perform a recovered mixed original's planned Signal leg exactly once, from
//! its recorded section, at its recorded basis and under its original
//! association; then admit the effect from Signal's own inventory as reopen
//! would. World's recovered record issues the grant.
use super::super::RuntimeWorldOwnerRoot;
use crate::lifecycle::signal_services::RuntimeWorldSignalServices;
use crate::publication::durable_signal::WorldDurableSignalProgress as Progress;
use crate::recovery::recovered_publication::RecoveredSignalLegPhase as Phase;
use crate::recovery::{
    RecoveredWorldPublicationRecord, RuntimeWorldRecoveryDenial as Recovery,
    WorldRecoveredContinuationDenial as Denial,
};
use std::ops::ControlFlow;
use std::sync::Arc;
use worth_signal::facade::durability::SignalReadmittedSourcePoint;

/// Whether the original now holds Signal's durable effect.
pub(super) enum SignalLegProgress {
    Durable,
    Pending,
}

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn complete_recovered_signal_leg(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        record: &RecoveredWorldPublicationRecord,
    ) -> Result<SignalLegProgress, Denial> {
        let Some(leg) = &record.planned_signal else {
            return Ok(SignalLegProgress::Durable);
        };
        let RuntimeWorldSignalServices::Authored(attachment) = &self.state.signal else {
            unreachable!("a Home World owns an authored Signal attachment")
        };
        let task = {
            let mut phase = leg.phase.lock().unwrap_or_else(|e| e.into_inner());
            if let Phase::Unbegun = &*phase {
                let prepared = attachment
                    .prepare_planned_signal_leg(
                        execution,
                        leg.planned.leg(),
                        record.expected.basis().signal_basis(),
                        record.open.association.clone(),
                    )
                    .map_err(Denial::SignalLeg)?;
                // From here a lost fate must never begin a second leg.
                *phase = Phase::ReopenRequired;
                let cancellation = crate::publication::RuntimeWorldCancellationSource::new();
                let task = prepared
                    .begin_recovered(
                        *execution,
                        record,
                        &attachment.issuer,
                        cancellation.token().signal_token(),
                    )
                    .map_err(|_| Denial::Admission(Recovery::ReopenRequired))?;
                *phase = Phase::Driving(task);
            }
            match &*phase {
                Phase::Driving(task) => Arc::clone(task),
                Phase::Durable(_) => return Ok(SignalLegProgress::Durable),
                Phase::ReopenRequired => return Err(Denial::Admission(Recovery::ReopenRequired)),
                Phase::Unbegun => unreachable!("begun above"),
            }
        };
        match task.drive(execution) {
            Progress::Committed(_) => {}
            Progress::Pending => return Ok(SignalLegProgress::Pending),
            Progress::ProvenNoEffect | Progress::Indeterminate => {
                *leg.phase.lock().unwrap_or_else(|e| e.into_inner()) = Phase::ReopenRequired;
                return Err(Denial::Admission(Recovery::ReopenRequired));
            }
        }
        let point = readmit_signal_leg(attachment, record, execution)?;
        *leg.phase.lock().unwrap_or_else(|e| e.into_inner()) = Phase::Durable(point);
        Ok(SignalLegProgress::Durable)
    }
}

/// The committed leg, found and readmitted through Signal's own inventory.
fn readmit_signal_leg<D, I, E, Ctx, T>(
    attachment: &crate::lifecycle::RuntimeWorldAuthoredSignalAttachment<D, I, E, Ctx, T>,
    record: &RecoveredWorldPublicationRecord,
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
            if effect.association() == Some(&record.open.association) {
                ControlFlow::Break(port.readmit_source_point(point, execution))
            } else {
                ControlFlow::Continue(())
            }
        })
        .map_err(|_| Denial::Admission(Recovery::ReopenRequired))?;
    match found {
        ControlFlow::Break(Ok(point)) => Ok(Arc::new(point)),
        ControlFlow::Break(Err(_)) | ControlFlow::Continue(()) => {
            Err(Denial::Admission(Recovery::ReopenRequired))
        }
    }
}
