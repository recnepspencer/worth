use worth_proof::TransitionOutcome;

use crate::physical_runtime::{
    PhysicalReadWorkRequest, PhysicalSchedulerDemand, PhysicalWorkAdmission,
    PhysicalWorkConsumerHandle, PhysicalWorkReadiness, PhysicalWorkScheduler, PhysicalWorkScope,
    ResourceAdmittedPhysicalWork,
};

use super::super::PhysicalRecoveryFreshReopenDenialKind;
use crate::physical_runtime::recovery_coordination::PhysicalRecoveryCoordination;

pub(super) fn admit(
    coordination: &PhysicalRecoveryCoordination,
    scope: PhysicalWorkScope,
    bytes: u64,
) -> Result<ResourceAdmittedPhysicalWork, PhysicalRecoveryFreshReopenDenialKind> {
    let request = PhysicalReadWorkRequest::new(
        scope,
        coordination.bases[0].clone(),
        coordination.work_security,
    )
    .map_err(|_| PhysicalRecoveryFreshReopenDenialKind::Submission)?;
    let receipt = match coordination
        .submission
        .read_submission()
        .submit(request)
        .into_raw()
    {
        TransitionOutcome::Success(receipt) => receipt,
        _ => return Err(PhysicalRecoveryFreshReopenDenialKind::Submission),
    };
    let admitted = PhysicalWorkAdmission::admit_recovery(
        &coordination.submission,
        receipt,
        &coordination.admission,
    )
    .map_err(PhysicalRecoveryFreshReopenDenialKind::PreEffect)?;
    let ready = match coordination.signal.request(admitted) {
        Ok(PhysicalWorkReadiness::Ready(ready)) => ready,
        Ok(PhysicalWorkReadiness::Blocked(blocked)) => {
            cancel_blocked(coordination, blocked);
            return Err(PhysicalRecoveryFreshReopenDenialKind::PreEffect(
                crate::physical_runtime::PhysicalWorkPreEffectDenial::DependencyBlocked,
            ));
        }
        Err(denial) => return Err(PhysicalRecoveryFreshReopenDenialKind::PreEffect(denial)),
    };
    let consumer = ready.consumer_handle();
    let (reservation, backend) = coordination
        .scheduler
        .record_read(&coordination.scheduler_security, bytes.max(1))
        .map_err(|_| {
            cancel_consumer(coordination, consumer);
            PhysicalRecoveryFreshReopenDenialKind::Submission
        })?;
    let demand =
        PhysicalSchedulerDemand::foreground(ready, reservation, None).map_err(|denial| {
            cancel_consumer(coordination, consumer);
            PhysicalRecoveryFreshReopenDenialKind::Scheduler(denial)
        })?;
    PhysicalWorkAdmission::require_current_recovery(&coordination.submission, demand.intent())
        .map_err(|denial| {
            cancel_consumer(coordination, consumer);
            PhysicalRecoveryFreshReopenDenialKind::PreEffect(denial)
        })?;
    let policy =
        crate::physical_runtime::record_serving::admit_record_queue_policy(demand.queue_work());
    PhysicalWorkScheduler::admit(coordination.scheduler.effects(), demand, &backend, policy)
        .map_err(|denial| {
            cancel_consumer(coordination, consumer);
            PhysicalRecoveryFreshReopenDenialKind::Scheduler(denial)
        })
}

fn cancel_blocked(
    coordination: &PhysicalRecoveryCoordination,
    blocked: crate::physical_runtime::BlockedPhysicalWork,
) {
    let identity = blocked.intent().identity();
    let route = blocked.authority().binding();
    if let Some((_, request)) = blocked.into_revalidation_parts() {
        cancel_consumer(
            coordination,
            PhysicalWorkConsumerHandle::new(identity, request, route),
        );
    }
}

fn cancel_consumer(
    coordination: &PhysicalRecoveryCoordination,
    consumer: PhysicalWorkConsumerHandle,
) {
    let _ = coordination
        .submission
        .cancel_before_dispatch(consumer.identity());
    let _ = coordination.signal.cancel(consumer);
}
