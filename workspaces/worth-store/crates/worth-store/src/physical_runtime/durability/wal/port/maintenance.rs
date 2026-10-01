use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_store_io_scheduler::QueueExecutionOutcome;
use worth_store_physical_backend::ArtifactTreeFile;

use super::PhysicalWalAppendPort;
use crate::physical_runtime::{
    instance::RecordSchedulerReservationDenial, PhysicalExecutorCommand,
    PhysicalMutationWorkRequest, PhysicalSchedulerDemand, PhysicalSchedulerDenial,
    PhysicalWalAppendFailureCause, PhysicalWalAppendScope, PhysicalWalBarrierScope,
    PhysicalWalFrameWriteDisposition, PhysicalWorkAdmission, PhysicalWorkReadiness,
    PhysicalWorkScheduler, PhysicalWorkSettlementEvidence,
};

#[path = "maintenance/barrier_command.rs"]
mod barrier_command;
use barrier_command::prepare_wal_barrier_command;

pub(in crate::physical_runtime) enum ScheduledMaintenanceDenial {
    NotStarted(PhysicalWalAppendFailureCause),
    WrittenAwaitingBarrier {
        interval: (u64, u64, u64, u64, u64, u64),
    },
    Write,
    Sync,
    Finish,
}

impl PhysicalWalAppendPort {
    /// Plans one maintenance frame and writes it through the shared WAL executor.
    ///
    /// A scheduler denial of the append happens before any byte is written.
    /// A scheduler denial of the following barrier keeps that written frame so
    /// the next call finishes the same barrier instead of appending again.
    pub(in crate::physical_runtime) fn append_scheduled_maintenance(
        &self,
        payload: &[u8],
    ) -> Result<(), ScheduledMaintenanceDenial> {
        self.append_scheduled_maintenance_receipt(payload)
            .map(|_| ())
    }

    pub(in crate::physical_runtime) fn append_scheduled_maintenance_receipt(
        &self,
        payload: &[u8],
    ) -> Result<super::DurableMaintenanceReceipt, ScheduledMaintenanceDenial> {
        let payload_digest = Sha256::digest(payload).into();
        if self.owner.maintenance_awaiting_barrier() {
            if !self.owner.maintenance_payload_matches(payload_digest) {
                return Err(ScheduledMaintenanceDenial::NotStarted(
                    PhysicalWalAppendFailureCause::RuntimeReleased,
                ));
            }
            let Some(artifact) = self.owner.planned_maintenance_artifact() else {
                self.abort_maintenance_frame();
                return Err(ScheduledMaintenanceDenial::Sync);
            };
            return self.finish_written_maintenance(&artifact, payload_digest);
        }
        let (artifact, segment, generation, offset, bytes) = self
            .plan_maintenance_frame(payload)
            .map_err(|()| {
            ScheduledMaintenanceDenial::NotStarted(PhysicalWalAppendFailureCause::RuntimeReleased)
        })?;
        let disposition = if offset == 0 {
            PhysicalWalFrameWriteDisposition::CreateSegment
        } else {
            PhysicalWalFrameWriteDisposition::AppendExistingSegment
        };
        let charged_bytes = bytes.len() as u64;
        let Some(charge) = self
            .publication
            .get()
            .and_then(|owner| owner.reserve_retained_bytes(bytes.len() as u64).ok())
        else {
            self.abort_maintenance_frame();
            return Err(ScheduledMaintenanceDenial::NotStarted(
                PhysicalWalAppendFailureCause::RuntimeReleased,
            ));
        };
        let Some(scope) = PhysicalWalAppendScope::new(
            segment,
            generation,
            offset,
            bytes.len() as u64,
            disposition,
        ) else {
            self.abort_maintenance_frame();
            return Err(ScheduledMaintenanceDenial::NotStarted(
                PhysicalWalAppendFailureCause::RuntimeReleased,
            ));
        };
        let command = match prepare_wal_frame_command(self, artifact.clone(), bytes, scope) {
            Ok(command) => command,
            Err(cause) => {
                self.abort_maintenance_frame();
                return Err(ScheduledMaintenanceDenial::NotStarted(cause));
            }
        };
        let settled = match self.execution.execute_physical_work(command) {
            Ok(outcome) => outcome.into_settled(),
            Err(_) => {
                // A missing settlement cannot prove that no byte escaped.
                // Keep this exact WAL charge until recovery resolves the tail.
                charge.seal();
                self.owner
                    .note_sealed_publication(segment, generation, charged_bytes);
                self.abort_maintenance_frame();
                return Err(ScheduledMaintenanceDenial::Write);
            }
        };
        let evidence = settled.into_evidence();
        if evidence.fate() != crate::physical_runtime::PhysicalWorkEffectFate::ProvenNoEffect {
            charge.seal();
            self.owner
                .note_sealed_publication(segment, generation, charged_bytes);
        }
        let wrote = matches!(
            evidence,
            PhysicalWorkSettlementEvidence::WalAppend {
                scheduler: QueueExecutionOutcome::Executed(_),
                ..
            } | PhysicalWorkSettlementEvidence::WalSegmentCreate {
                scheduler: QueueExecutionOutcome::Executed(_),
                ..
            }
        );
        if !wrote {
            self.abort_maintenance_frame();
            return Err(ScheduledMaintenanceDenial::Write);
        }
        self.owner.note_maintenance_written();
        self.finish_written_maintenance(&artifact, payload_digest)
    }

    fn finish_written_maintenance(
        &self,
        artifact: &ArtifactTreeFile,
        payload_digest: [u8; 32],
    ) -> Result<super::DurableMaintenanceReceipt, ScheduledMaintenanceDenial> {
        let interval = self
            .owner
            .planned_maintenance_interval()
            .ok_or(ScheduledMaintenanceDenial::Finish)?;
        match synchronize_scheduled_maintenance(self, artifact) {
            Ok(synchronization) => {
                let frame_digests = self
                    .owner
                    .planned_maintenance_frame_digests(payload_digest)
                    .ok_or(ScheduledMaintenanceDenial::Finish)?;
                self.finish_maintenance_frame()
                    .map_err(|()| ScheduledMaintenanceDenial::Finish)?;
                Ok(super::DurableMaintenanceReceipt::completed(
                    payload_digest,
                    interval,
                    Some(frame_digests),
                    synchronization,
                ))
            }
            Err(MaintenanceBarrierDenial::Waiting(_)) => {
                Err(ScheduledMaintenanceDenial::WrittenAwaitingBarrier { interval })
            }
            Err(MaintenanceBarrierDenial::Failed) => {
                self.abort_maintenance_frame();
                Err(ScheduledMaintenanceDenial::Sync)
            }
        }
    }
}

pub(super) fn prepare_wal_frame_command(
    port: &PhysicalWalAppendPort,
    artifact: ArtifactTreeFile,
    payload: Vec<u8>,
    scope: PhysicalWalAppendScope,
) -> Result<PhysicalExecutorCommand, PhysicalWalAppendFailureCause> {
    let runtime = port
        .runtime
        .upgrade()
        .ok_or(PhysicalWalAppendFailureCause::RuntimeReleased)?;
    let request = PhysicalMutationWorkRequest::wal_append(
        scope,
        port.record.wal_append_basis(),
        port.record.security(),
    )
    .map_err(PhysicalWalAppendFailureCause::SubmissionDenied)?;
    let receipt = match runtime
        .submission
        .mutation_submission()
        .submit(request)
        .into_raw()
    {
        TransitionOutcome::Success(receipt) => receipt,
        TransitionOutcome::Denied(denial) => {
            return Err(PhysicalWalAppendFailureCause::SubmissionDenied(denial))
        }
        TransitionOutcome::Deferred(deferred) => {
            return Err(PhysicalWalAppendFailureCause::SubmissionDeferred(deferred))
        }
        TransitionOutcome::Stale(stale) => {
            return Err(PhysicalWalAppendFailureCause::SubmissionStale(stale))
        }
        TransitionOutcome::RebindRequired(rebind) => match rebind {},
        TransitionOutcome::Failed(failure) => {
            return Err(PhysicalWalAppendFailureCause::SubmissionFailed(failure))
        }
    };
    let admitted = PhysicalWorkAdmission::admit(
        &runtime.submission,
        receipt,
        &port.physical,
        &runtime.health,
    )
    .map_err(PhysicalWalAppendFailureCause::PreEffect)?;
    let ready = match runtime
        .signal
        .request(admitted)
        .map_err(PhysicalWalAppendFailureCause::PreEffect)?
    {
        PhysicalWorkReadiness::Ready(ready) => ready,
        PhysicalWorkReadiness::Blocked(blocked) => {
            return Err(PhysicalWalAppendFailureCause::DependencyBlocked {
                class: blocked.class(),
                condition: blocked.condition(),
            })
        }
    };
    let (reservation, backend) = port
        .scheduler
        .wal_append(port.record.scheduler_security(), scope.byte_count())
        .map_err(|denial: RecordSchedulerReservationDenial| match denial {
            RecordSchedulerReservationDenial::Admission(denial) => {
                PhysicalWalAppendFailureCause::SchedulerReservationDenied(denial)
            }
            RecordSchedulerReservationDenial::OwedBackgroundTurn => {
                PhysicalWalAppendFailureCause::Scheduler(
                    PhysicalSchedulerDenial::OwedBackgroundTurn,
                )
            }
        })?;
    let demand = PhysicalSchedulerDemand::foreground(ready, reservation, None)
        .map_err(PhysicalWalAppendFailureCause::Scheduler)?;
    PhysicalWorkAdmission::require_current(&runtime.submission, demand.intent(), &runtime.health)
        .map_err(PhysicalWalAppendFailureCause::PreEffect)?;
    let policy =
        crate::physical_runtime::record_serving::admit_record_queue_policy(demand.queue_work());
    let work = PhysicalWorkScheduler::admit(port.scheduler.effects(), demand, &backend, policy)
        .map_err(PhysicalWalAppendFailureCause::Scheduler)?;
    PhysicalExecutorCommand::wal_frame_write(work, artifact, payload)
        .map_err(PhysicalWalAppendFailureCause::Command)
}

pub(super) enum MaintenanceBarrierDenial {
    Waiting(PhysicalSchedulerDenial),
    Failed,
}

fn synchronize_scheduled_maintenance(
    port: &PhysicalWalAppendPort,
    artifact: &ArtifactTreeFile,
) -> Result<crate::physical_runtime::CompletedPhysicalWalBarrier, MaintenanceBarrierDenial> {
    let interval = port
        .owner
        .planned_maintenance_interval()
        .ok_or(MaintenanceBarrierDenial::Failed)?;
    synchronize_maintenance_interval(port, artifact, interval)
}

pub(super) fn synchronize_maintenance_interval(
    port: &PhysicalWalAppendPort,
    artifact: &ArtifactTreeFile,
    interval: (u64, u64, u64, u64, u64, u64),
) -> Result<crate::physical_runtime::CompletedPhysicalWalBarrier, MaintenanceBarrierDenial> {
    let scope = PhysicalWalBarrierScope::new(
        maintenance_barrier_identity(b"group", interval),
        maintenance_barrier_identity(b"member", interval),
        1,
        interval.0,
        interval.1,
        interval.2,
        interval.3,
        interval.4,
        interval.5,
    )
    .ok_or(MaintenanceBarrierDenial::Failed)?;
    let command = prepare_wal_barrier_command(port, artifact.clone(), scope)?;
    let settled = port
        .execution
        .execute_physical_work(command)
        .map_err(|_| MaintenanceBarrierDenial::Failed)?
        .into_settled();
    match settled.into_evidence() {
        PhysicalWorkSettlementEvidence::WalBarrier {
            physical,
            scheduler: QueueExecutionOutcome::Executed(_),
        } => Ok(physical),
        _ => Err(MaintenanceBarrierDenial::Failed),
    }
}

fn maintenance_barrier_identity(
    label: &[u8],
    interval: (u64, u64, u64, u64, u64, u64),
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"store.physical.maintenance-barrier.v1");
    digest.update(label);
    for value in [
        interval.0, interval.1, interval.2, interval.3, interval.4, interval.5,
    ] {
        digest.update(value.to_le_bytes());
    }
    digest.finalize().into()
}
