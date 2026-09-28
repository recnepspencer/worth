use super::*;

pub(super) fn prepare_wal_barrier_command(
    port: &PhysicalWalAppendPort,
    artifact: ArtifactTreeFile,
    scope: PhysicalWalBarrierScope,
) -> Result<PhysicalExecutorCommand, MaintenanceBarrierDenial> {
    let runtime = port
        .runtime
        .upgrade()
        .ok_or(MaintenanceBarrierDenial::Failed)?;
    let request = PhysicalMutationWorkRequest::wal_durability_barrier(
        scope,
        port.record.wal_barrier_basis(),
        port.record.security(),
    )
    .map_err(|_| MaintenanceBarrierDenial::Failed)?;
    let receipt = match runtime
        .submission
        .mutation_submission()
        .submit(request)
        .into_raw()
    {
        TransitionOutcome::Success(receipt) => receipt,
        TransitionOutcome::Denied(_)
        | TransitionOutcome::Deferred(_)
        | TransitionOutcome::Stale(_)
        | TransitionOutcome::Failed(_) => return Err(MaintenanceBarrierDenial::Failed),
        TransitionOutcome::RebindRequired(rebind) => match rebind {},
    };
    let admitted = PhysicalWorkAdmission::admit(
        &runtime.submission,
        receipt,
        &port.physical,
        &runtime.health,
    )
    .map_err(|_| MaintenanceBarrierDenial::Failed)?;
    let ready = match runtime
        .signal
        .request(admitted)
        .map_err(|_| MaintenanceBarrierDenial::Failed)?
    {
        PhysicalWorkReadiness::Ready(ready) => ready,
        PhysicalWorkReadiness::Blocked(_) => return Err(MaintenanceBarrierDenial::Failed),
    };
    #[cfg(feature = "certification-test-authority")]
    if port
        .owe_before_maintenance_barrier
        .swap(false, std::sync::atomic::Ordering::Relaxed)
    {
        port.scheduler.certification_owe_background_turn();
    }
    let (reservation, backend) = port
        .scheduler
        .wal_durability_barrier(port.record.scheduler_security())
        .map_err(|denial| match denial {
            RecordSchedulerReservationDenial::OwedBackgroundTurn => {
                MaintenanceBarrierDenial::Waiting(PhysicalSchedulerDenial::OwedBackgroundTurn)
            }
            RecordSchedulerReservationDenial::Admission(_) => MaintenanceBarrierDenial::Failed,
        })?;
    let demand = PhysicalSchedulerDemand::foreground(ready, reservation, None).map_err(
        |denial| match denial {
            PhysicalSchedulerDenial::OwedBackgroundTurn
            | PhysicalSchedulerDenial::EffectConflict
            | PhysicalSchedulerDenial::EffectSlotsExhausted => {
                MaintenanceBarrierDenial::Waiting(denial)
            }
            _ => MaintenanceBarrierDenial::Failed,
        },
    )?;
    PhysicalWorkAdmission::require_current(&runtime.submission, demand.intent(), &runtime.health)
        .map_err(|_| MaintenanceBarrierDenial::Failed)?;
    let policy =
        crate::physical_runtime::record_serving::admit_record_queue_policy(demand.queue_work());
    let work = PhysicalWorkScheduler::admit(port.scheduler.effects(), demand, &backend, policy)
        .map_err(|denial| match denial {
            PhysicalSchedulerDenial::OwedBackgroundTurn
            | PhysicalSchedulerDenial::EffectConflict
            | PhysicalSchedulerDenial::EffectSlotsExhausted => {
                MaintenanceBarrierDenial::Waiting(denial)
            }
            _ => MaintenanceBarrierDenial::Failed,
        })?;
    let binding = maintenance_barrier_identity(
        b"binding",
        (
            scope.segment(),
            scope.generation(),
            scope.lsn_start(),
            scope.lsn_end_exclusive(),
            scope.append_offset(),
            scope.append_byte_count(),
        ),
    );
    PhysicalExecutorCommand::wal_barrier(work, artifact, binding)
        .map_err(|_| MaintenanceBarrierDenial::Failed)
}
