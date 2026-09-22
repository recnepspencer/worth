use worth_proof::TransitionOutcome;
use worth_store_io_scheduler::BackgroundPacingOutcome;

use crate::physical_runtime::work::PhysicalWalReclamationScope;
use crate::physical_runtime::{
    PhysicalMutationWorkRequest, PhysicalReadWorkRequest, PhysicalSchedulerDemand,
    PhysicalWorkAdmission, PhysicalWorkConsumerHandle, PhysicalWorkReadiness,
    PhysicalWorkScheduler, PhysicalWorkScope, ResourceAdmittedPhysicalWork,
};

use super::super::PhysicalRecoveryCoordination;

mod denial;
pub use denial::{
    PhysicalRecoveryCleanupAdmissionDenial, PhysicalRecoveryCleanupAdmissionDenialKind,
};

pub(super) fn read(
    coordination: &PhysicalRecoveryCoordination,
    bytes: u64,
) -> Result<ResourceAdmittedPhysicalWork, PhysicalRecoveryCleanupAdmissionDenial> {
    let ready = ready(coordination, submit_read(coordination)?)?;
    admit_foreground_read(coordination, ready, bytes)
}

pub(super) fn removal(
    coordination: &PhysicalRecoveryCoordination,
    scope: PhysicalWalReclamationScope,
) -> Result<ResourceAdmittedPhysicalWork, PhysicalRecoveryCleanupAdmissionDenial> {
    let ready = ready(coordination, submit_removal(coordination, scope)?)?;
    admit_background_removal(coordination, ready, scope)
}

fn submit_read(
    coordination: &PhysicalRecoveryCoordination,
) -> Result<
    crate::physical_runtime::PhysicalWorkSubmissionReceipt,
    PhysicalRecoveryCleanupAdmissionDenial,
> {
    let request = PhysicalReadWorkRequest::new(
        PhysicalWorkScope::artifact(
            worth_store_physical_format::RecordArtifactFile::CurrentRootSelector,
        ),
        coordination.bases[0].clone(),
        coordination.work_security,
    )
    .map_err(|_| request_denial())?;
    match coordination
        .submission
        .read_submission()
        .submit(request)
        .into_raw()
    {
        TransitionOutcome::Success(receipt) => Ok(receipt),
        TransitionOutcome::Denied(denial) => Err(before_submission(
            PhysicalRecoveryCleanupAdmissionDenialKind::SubmissionDenied(denial),
        )),
        TransitionOutcome::Deferred(deferred) => Err(before_submission(
            PhysicalRecoveryCleanupAdmissionDenialKind::SubmissionDeferred(deferred),
        )),
        TransitionOutcome::Stale(stale) => Err(before_submission(
            PhysicalRecoveryCleanupAdmissionDenialKind::SubmissionStale(stale),
        )),
        TransitionOutcome::RebindRequired(rebind) => match rebind {},
        TransitionOutcome::Failed(failure) => Err(before_submission(
            PhysicalRecoveryCleanupAdmissionDenialKind::SubmissionFailed(failure),
        )),
    }
}

fn admit_foreground_read(
    coordination: &PhysicalRecoveryCoordination,
    ready: crate::physical_runtime::ReadyPhysicalWork,
    bytes: u64,
) -> Result<ResourceAdmittedPhysicalWork, PhysicalRecoveryCleanupAdmissionDenial> {
    let consumer = ready.consumer_handle();
    let (reservation, backend) = coordination
        .scheduler
        .record_read(&coordination.scheduler_security, bytes.max(1))
        .map_err(|denial| {
            let kind = match denial {
                crate::physical_runtime::instance::RecordSchedulerReservationDenial::Admission(
                    denial,
                ) => PhysicalRecoveryCleanupAdmissionDenialKind::SchedulerForegroundCapacity(denial),
                crate::physical_runtime::instance::RecordSchedulerReservationDenial::OwedBackgroundTurn => {
                    PhysicalRecoveryCleanupAdmissionDenialKind::Scheduler(
                        crate::physical_runtime::PhysicalSchedulerDenial::OwedBackgroundTurn,
                    )
                }
            };
            after_cancel(coordination, consumer, kind)
        })?;
    let demand =
        PhysicalSchedulerDemand::foreground(ready, reservation, None).map_err(|denial| {
            after_cancel(
                coordination,
                consumer,
                PhysicalRecoveryCleanupAdmissionDenialKind::Scheduler(denial),
            )
        })?;
    PhysicalWorkAdmission::require_current_recovery(&coordination.submission, demand.intent())
        .map_err(|denial| {
            after_cancel(
                coordination,
                consumer,
                PhysicalRecoveryCleanupAdmissionDenialKind::PreEffect(denial),
            )
        })?;
    let policy =
        crate::physical_runtime::record_serving::admit_record_queue_policy(demand.queue_work());
    PhysicalWorkScheduler::admit(coordination.scheduler.effects(), demand, &backend, policy).map_err(|denial| {
        after_cancel(
            coordination,
            consumer,
            PhysicalRecoveryCleanupAdmissionDenialKind::Scheduler(denial),
        )
    })
}

fn submit_removal(
    coordination: &PhysicalRecoveryCoordination,
    scope: PhysicalWalReclamationScope,
) -> Result<
    crate::physical_runtime::PhysicalWorkSubmissionReceipt,
    PhysicalRecoveryCleanupAdmissionDenial,
> {
    let request = PhysicalMutationWorkRequest::wal_reclamation(
        scope,
        coordination.bases[3].clone(),
        coordination.work_security,
    )
    .map_err(|_| request_denial())?;
    match coordination
        .submission
        .mutation_submission()
        .submit(request)
        .into_raw()
    {
        TransitionOutcome::Success(receipt) => Ok(receipt),
        TransitionOutcome::Denied(denial) => Err(before_submission(
            PhysicalRecoveryCleanupAdmissionDenialKind::SubmissionDenied(denial),
        )),
        TransitionOutcome::Deferred(deferred) => Err(before_submission(
            PhysicalRecoveryCleanupAdmissionDenialKind::SubmissionDeferred(deferred),
        )),
        TransitionOutcome::Stale(stale) => Err(before_submission(
            PhysicalRecoveryCleanupAdmissionDenialKind::SubmissionStale(stale),
        )),
        TransitionOutcome::RebindRequired(rebind) => match rebind {},
        TransitionOutcome::Failed(failure) => Err(before_submission(
            PhysicalRecoveryCleanupAdmissionDenialKind::SubmissionFailed(failure),
        )),
    }
}

fn admit_background_removal(
    coordination: &PhysicalRecoveryCoordination,
    ready: crate::physical_runtime::ReadyPhysicalWork,
    scope: PhysicalWalReclamationScope,
) -> Result<ResourceAdmittedPhysicalWork, PhysicalRecoveryCleanupAdmissionDenial> {
    let consumer = ready.consumer_handle();
    #[cfg(feature = "certification-test-authority")]
    let foreground_pressure_events =
        u64::from(coordination.take_certification_cleanup_background_deferral());
    #[cfg(not(feature = "certification-test-authority"))]
    let foreground_pressure_events = 0;
    let (pacing, backend, policy, capacity) = coordination
        .scheduler
        .wal_reclamation_background(
            &coordination.scheduler_security,
            scope.byte_count(),
            foreground_pressure_events,
        )
        .map_err(|denial| {
            let kind = match denial {
                crate::physical_runtime::instance::PhysicalWalReclamationSchedulerAdmissionDenial::Foreground(denial) => PhysicalRecoveryCleanupAdmissionDenialKind::SchedulerForegroundCapacity(denial),
                crate::physical_runtime::instance::PhysicalWalReclamationSchedulerAdmissionDenial::OwedBackgroundTurn => PhysicalRecoveryCleanupAdmissionDenialKind::Scheduler(crate::physical_runtime::PhysicalSchedulerDenial::OwedBackgroundTurn),
                crate::physical_runtime::instance::PhysicalWalReclamationSchedulerAdmissionDenial::Background(denial) => PhysicalRecoveryCleanupAdmissionDenialKind::SchedulerBackgroundCapacity(denial),
            };
            after_cancel(coordination, consumer, kind)
        })?;
    let lease = match pacing {
        BackgroundPacingOutcome::AdmittedWithDebt(admitted) => admitted.into_lease(),
        other => {
            drop(capacity);
            if matches!(
                other,
                BackgroundPacingOutcome::Denied(_) | BackgroundPacingOutcome::Violation(_)
            ) {
                coordination
                    .scheduler
                    .cancel_wal_reclamation_background_head();
            }
            return Err(after_cancel(
                coordination,
                consumer,
                PhysicalRecoveryCleanupAdmissionDenialKind::BackgroundPacing(other),
            ))
        }
    };
    let demand =
        PhysicalSchedulerDemand::wal_reclamation_background(ready, lease, capacity).map_err(|denial| {
            after_cancel(
                coordination,
                consumer,
                PhysicalRecoveryCleanupAdmissionDenialKind::Scheduler(denial),
            )
        })?;
    PhysicalWorkAdmission::require_current_recovery(&coordination.submission, demand.intent())
        .map_err(|denial| {
            after_cancel(
                coordination,
                consumer,
                PhysicalRecoveryCleanupAdmissionDenialKind::PreEffect(denial),
            )
        })?;
    PhysicalWorkScheduler::admit(coordination.scheduler.effects(), demand, &backend, policy).map_err(|denial| {
        after_cancel(
            coordination,
            consumer,
            PhysicalRecoveryCleanupAdmissionDenialKind::Scheduler(denial),
        )
    })
}

fn ready(
    coordination: &PhysicalRecoveryCoordination,
    receipt: crate::physical_runtime::PhysicalWorkSubmissionReceipt,
) -> Result<crate::physical_runtime::ReadyPhysicalWork, PhysicalRecoveryCleanupAdmissionDenial> {
    let identity = receipt.identity();
    let admitted = PhysicalWorkAdmission::admit_recovery(
        &coordination.submission,
        receipt,
        &coordination.admission,
    )
    .map_err(|denial| {
        PhysicalRecoveryCleanupAdmissionDenial::after_submission(
            PhysicalRecoveryCleanupAdmissionDenialKind::PreEffect(denial),
            true,
        )
    })?;
    match coordination.signal.request(admitted) {
        Ok(PhysicalWorkReadiness::Ready(ready)) => Ok(ready),
        Ok(PhysicalWorkReadiness::Blocked(blocked)) => {
            let identity = blocked.intent().identity();
            let route = blocked.authority().binding();
            let cancelled = if let Some((_, request)) = blocked.into_revalidation_parts() {
                cancel(
                    coordination,
                    PhysicalWorkConsumerHandle::new(identity, request, route),
                )
            } else {
                coordination.submission.cancel_before_dispatch(identity)
            };
            Err(PhysicalRecoveryCleanupAdmissionDenial::after_submission(
                PhysicalRecoveryCleanupAdmissionDenialKind::PreEffect(
                    crate::physical_runtime::PhysicalWorkPreEffectDenial::DependencyBlocked,
                ),
                cancelled,
            ))
        }
        Err(denial) => {
            let cancelled = coordination.submission.cancel_before_dispatch(identity);
            Err(PhysicalRecoveryCleanupAdmissionDenial::after_submission(
                PhysicalRecoveryCleanupAdmissionDenialKind::PreEffect(denial),
                cancelled,
            ))
        }
    }
}

fn cancel(
    coordination: &PhysicalRecoveryCoordination,
    consumer: PhysicalWorkConsumerHandle,
) -> bool {
    let cancelled = coordination
        .submission
        .cancel_before_dispatch(consumer.identity());
    let _ = coordination.signal.cancel(consumer);
    cancelled
}

fn after_cancel(
    coordination: &PhysicalRecoveryCoordination,
    consumer: PhysicalWorkConsumerHandle,
    kind: PhysicalRecoveryCleanupAdmissionDenialKind,
) -> PhysicalRecoveryCleanupAdmissionDenial {
    PhysicalRecoveryCleanupAdmissionDenial::after_submission(kind, cancel(coordination, consumer))
}

const fn request_kind() -> PhysicalRecoveryCleanupAdmissionDenialKind {
    PhysicalRecoveryCleanupAdmissionDenialKind::Request
}

const fn request_denial() -> PhysicalRecoveryCleanupAdmissionDenial {
    before_submission(request_kind())
}

const fn before_submission(
    kind: PhysicalRecoveryCleanupAdmissionDenialKind,
) -> PhysicalRecoveryCleanupAdmissionDenial {
    PhysicalRecoveryCleanupAdmissionDenial::before_submission(kind)
}
