use worth_proof::TransitionOutcome;
use worth_store_io_scheduler::{SecureIoOperation, SecureIoPreservationRequest};
use worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement;

use crate::physical_runtime::{
    PhysicalMutationWorkRequest, PhysicalSchedulerDemand, PhysicalWorkAdmission,
    PhysicalWorkConsumerHandle, PhysicalWorkReadiness, PhysicalWorkScheduler, PhysicalWorkScope,
    ResourceAdmittedPhysicalWork,
};

use super::super::{
    PhysicalRecoveryStagingCommandDenial, PhysicalRecoveryStagingCommandDenialKind,
    PhysicalRecoveryStagingCommandOutcome, PhysicalRecoveryStagingCommandStage,
};
use crate::physical_runtime::recovery_coordination::PhysicalRecoveryCoordination;

pub(super) fn admit(
    coordination: &PhysicalRecoveryCoordination,
    stage: PhysicalRecoveryStagingCommandStage,
    scope: PhysicalWorkScope,
    durability: ArtifactRangeWriteDurabilityRequirement,
    bytes: u64,
    synchronization: bool,
) -> Result<ResourceAdmittedPhysicalWork, PhysicalRecoveryStagingCommandOutcome> {
    let request = if synchronization {
        PhysicalMutationWorkRequest::publication(
            scope,
            coordination.bases[1].clone(),
            coordination.work_security,
            durability,
        )
    } else {
        PhysicalMutationWorkRequest::recovery_exact_write(
            scope,
            coordination.bases[1].clone(),
            coordination.work_security,
            durability,
        )
    }
    .map_err(|_| submission_denial(stage))?;
    let receipt = match coordination
        .submission
        .mutation_submission()
        .submit(request)
        .into_raw()
    {
        TransitionOutcome::Success(receipt) => receipt,
        _ => return Err(submission_denial(stage)),
    };
    let admitted = PhysicalWorkAdmission::admit_recovery(
        &coordination.submission,
        receipt,
        &coordination.admission,
    )
    .map_err(|denial| pre_effect(stage, denial))?;
    let ready = match coordination.signal.request(admitted) {
        Ok(PhysicalWorkReadiness::Ready(ready)) => ready,
        Ok(PhysicalWorkReadiness::Blocked(blocked)) => {
            cancel_blocked(coordination, blocked);
            return Err(pre_effect(
                stage,
                crate::physical_runtime::PhysicalWorkPreEffectDenial::DependencyBlocked,
            ));
        }
        Err(denial) => return Err(pre_effect(stage, denial)),
    };
    let consumer = ready.consumer_handle();
    let (reservation, backend) = match coordination.scheduler.record_write(
        &coordination.scheduler_security,
        bytes.max(1),
        synchronization,
        true,
    ) {
        Ok(parts) => parts,
        Err(_) => {
            cancel_consumer(coordination, consumer);
            return Err(submission_denial(stage));
        }
    };
    let secure_operation = if synchronization {
        SecureIoOperation::BatchedWrite
    } else {
        SecureIoOperation::WriteBack
    };
    let secure_io = match worth_store_io_scheduler::admit_secure_io_scope_for_scheduler(
        SecureIoPreservationRequest::new(
            secure_operation,
            &coordination.scheduler_security,
            &backend,
        ),
    ) {
        Ok(secure_io) => secure_io,
        Err(_) => {
            cancel_consumer(coordination, consumer);
            return Err(submission_denial(stage));
        }
    };
    let demand = match PhysicalSchedulerDemand::foreground(ready, reservation, Some(secure_io)) {
        Ok(demand) => demand,
        Err(denial) => {
            cancel_consumer(coordination, consumer);
            return Err(scheduler(stage, denial));
        }
    };
    if let Err(denial) =
        PhysicalWorkAdmission::require_current_recovery(&coordination.submission, demand.intent())
    {
        cancel_consumer(coordination, consumer);
        return Err(pre_effect(stage, denial));
    }
    let policy =
        crate::physical_runtime::record_serving::admit_record_queue_policy(demand.queue_work());
    match PhysicalWorkScheduler::admit(coordination.scheduler.effects(), demand, &backend, policy) {
        Ok(work) => Ok(work),
        Err(denial) => {
            cancel_consumer(coordination, consumer);
            Err(scheduler(stage, denial))
        }
    }
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

fn submission_denial(
    stage: PhysicalRecoveryStagingCommandStage,
) -> PhysicalRecoveryStagingCommandOutcome {
    denied(stage, PhysicalRecoveryStagingCommandDenialKind::Submission)
}

fn pre_effect(
    stage: PhysicalRecoveryStagingCommandStage,
    denial: crate::physical_runtime::PhysicalWorkPreEffectDenial,
) -> PhysicalRecoveryStagingCommandOutcome {
    denied(
        stage,
        PhysicalRecoveryStagingCommandDenialKind::PreEffect(denial),
    )
}

fn scheduler(
    stage: PhysicalRecoveryStagingCommandStage,
    denial: crate::physical_runtime::PhysicalSchedulerDenial,
) -> PhysicalRecoveryStagingCommandOutcome {
    denied(
        stage,
        PhysicalRecoveryStagingCommandDenialKind::Scheduler(denial),
    )
}

fn denied(
    stage: PhysicalRecoveryStagingCommandStage,
    denial: PhysicalRecoveryStagingCommandDenialKind,
) -> PhysicalRecoveryStagingCommandOutcome {
    PhysicalRecoveryStagingCommandOutcome::DeniedBeforeEffect(
        PhysicalRecoveryStagingCommandDenial::new(stage, denial, None, None),
    )
}
