use worth_proof::TransitionOutcome;

use crate::physical_runtime::{
    PhysicalMutationWorkRequest, PhysicalSchedulerDemand, PhysicalWorkAdmission,
    PhysicalWorkConsumerHandle, PhysicalWorkReadiness, PhysicalWorkScheduler, PhysicalWorkScope,
    ResourceAdmittedPhysicalWork,
};

use super::super::{
    PhysicalRecoveryPublicationCommandDenial, PhysicalRecoveryPublicationCommandDenialKind,
    PhysicalRecoveryPublicationCommandOutcome, PhysicalRecoveryPublicationCommandStage,
};
use crate::physical_runtime::recovery_coordination::PhysicalRecoveryCoordination;

pub(super) fn admit(
    coordination: &PhysicalRecoveryCoordination,
    stage: PhysicalRecoveryPublicationCommandStage,
    scope: PhysicalWorkScope,
) -> Result<ResourceAdmittedPhysicalWork, PhysicalRecoveryPublicationCommandOutcome> {
    let request = PhysicalMutationWorkRequest::recovery_root_publication(
        scope,
        coordination.bases[2].clone(),
        coordination.work_security,
    )
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
    let admitted = match stage {
        PhysicalRecoveryPublicationCommandStage::CandidateMaterialization => coordination
            .scheduler
            .root_candidate_materialization(&coordination.scheduler_security)
            .map(|admission| admission.into_parts()),
        PhysicalRecoveryPublicationCommandStage::CandidateSynchronization => coordination
            .scheduler
            .root_candidate_sync(&coordination.scheduler_security)
            .map(|admission| admission.into_parts()),
        PhysicalRecoveryPublicationCommandStage::RootProtocolReplacement => coordination
            .scheduler
            .root_catalog_replacement(&coordination.scheduler_security)
            .map(|admission| admission.into_parts()),
        PhysicalRecoveryPublicationCommandStage::RecordNamespaceSynchronization => coordination
            .scheduler
            .root_namespace_sync(&coordination.scheduler_security)
            .map(|admission| admission.into_parts()),
    };
    let (reservation, backend) = match admitted {
        Ok(parts) => parts,
        Err(_) => {
            cancel_consumer(coordination, consumer);
            return Err(submission_denial(stage));
        }
    };
    let demand = match PhysicalSchedulerDemand::foreground(ready, reservation, None) {
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
    stage: PhysicalRecoveryPublicationCommandStage,
) -> PhysicalRecoveryPublicationCommandOutcome {
    denied(
        stage,
        PhysicalRecoveryPublicationCommandDenialKind::Submission,
    )
}

fn pre_effect(
    stage: PhysicalRecoveryPublicationCommandStage,
    denial: crate::physical_runtime::PhysicalWorkPreEffectDenial,
) -> PhysicalRecoveryPublicationCommandOutcome {
    denied(
        stage,
        PhysicalRecoveryPublicationCommandDenialKind::PreEffect(denial),
    )
}

fn scheduler(
    stage: PhysicalRecoveryPublicationCommandStage,
    denial: crate::physical_runtime::PhysicalSchedulerDenial,
) -> PhysicalRecoveryPublicationCommandOutcome {
    denied(
        stage,
        PhysicalRecoveryPublicationCommandDenialKind::Scheduler(denial),
    )
}

fn denied(
    stage: PhysicalRecoveryPublicationCommandStage,
    denial: PhysicalRecoveryPublicationCommandDenialKind,
) -> PhysicalRecoveryPublicationCommandOutcome {
    PhysicalRecoveryPublicationCommandOutcome::DeniedBeforeEffect(
        PhysicalRecoveryPublicationCommandDenial::new(
            stage,
            denial,
            Box::new([]),
            None,
            None,
            None,
        ),
    )
}
