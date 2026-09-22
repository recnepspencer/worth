use std::sync::{Arc, Weak};

use worth_proof::TransitionOutcome;

use crate::physical_runtime::instance::{
    PhysicalSchedulerAdmissionOwner, PhysicalStoreWorkRuntime, RecordSchedulerReservationDenial,
};
use crate::physical_runtime::record_serving::RecordWorkAdmission;
use crate::physical_runtime::work::PhysicalWorkAdmissionAuthority;
use worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement;
use worth_store_physical_format::RecordArtifactFile;

use crate::physical_runtime::{
    PhysicalExecutorCommand, PhysicalMutationWorkRequest, PhysicalPublicationEffect,
    PhysicalRootPublicationWorkAction, PhysicalRootPublicationWorkScope, PhysicalSchedulerDemand,
    PhysicalWorkAdmission, PhysicalWorkExecution, PhysicalWorkPreEffectDenial,
    PhysicalWorkReadiness, PhysicalWorkScheduler, PhysicalWorkScope,
    PhysicalWorkSubmissionDeferred, PhysicalWorkSubmissionDenial, PhysicalWorkSubmissionFailure,
    PhysicalWorkSubmissionStale, SettledPhysicalWork,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalRootPublicationWorkFailure {
    RuntimeReleased,
    SubmissionDenied(PhysicalWorkSubmissionDenial),
    SubmissionDeferred(PhysicalWorkSubmissionDeferred),
    SubmissionStale(PhysicalWorkSubmissionStale),
    SubmissionFailed(PhysicalWorkSubmissionFailure),
    PreEffect(PhysicalWorkPreEffectDenial),
    DependencyBlocked,
    SchedulerReservation(RecordSchedulerReservationDenial),
    Scheduler(crate::physical_runtime::PhysicalSchedulerDenial),
    Command(crate::physical_runtime::PhysicalExecutorCommandDenial),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRootPublicationWorkFailureCause {
    RuntimeReleased,
    SubmissionDenied,
    SubmissionDeferred,
    SubmissionStale,
    SubmissionFailed,
    PreEffect,
    DependencyBlocked,
    SchedulerReservation,
    Scheduler,
    EffectConflict,
    Command,
}

impl PhysicalRootPublicationWorkFailure {
    pub(in crate::physical_runtime) const fn cause(
        &self,
    ) -> PhysicalRootPublicationWorkFailureCause {
        match self {
            Self::RuntimeReleased => PhysicalRootPublicationWorkFailureCause::RuntimeReleased,
            Self::SubmissionDenied(_) => PhysicalRootPublicationWorkFailureCause::SubmissionDenied,
            Self::SubmissionDeferred(_) => {
                PhysicalRootPublicationWorkFailureCause::SubmissionDeferred
            }
            Self::SubmissionStale(_) => PhysicalRootPublicationWorkFailureCause::SubmissionStale,
            Self::SubmissionFailed(_) => PhysicalRootPublicationWorkFailureCause::SubmissionFailed,
            Self::PreEffect(_) => PhysicalRootPublicationWorkFailureCause::PreEffect,
            Self::DependencyBlocked => PhysicalRootPublicationWorkFailureCause::DependencyBlocked,
            Self::SchedulerReservation(_) => {
                PhysicalRootPublicationWorkFailureCause::SchedulerReservation
            }
            Self::Scheduler(crate::physical_runtime::PhysicalSchedulerDenial::EffectConflict) => {
                PhysicalRootPublicationWorkFailureCause::EffectConflict
            }
            Self::Scheduler(_) => PhysicalRootPublicationWorkFailureCause::Scheduler,
            Self::Command(_) => PhysicalRootPublicationWorkFailureCause::Command,
        }
    }
}

#[derive(Clone)]
pub(in crate::physical_runtime) struct PhysicalRootPublicationWorkPort {
    runtime: Weak<PhysicalStoreWorkRuntime>,
    execution: PhysicalWorkExecution,
    physical: PhysicalWorkAdmissionAuthority,
    scheduler: PhysicalSchedulerAdmissionOwner,
    record: Arc<RecordWorkAdmission>,
}

impl PhysicalRootPublicationWorkPort {
    pub(in crate::physical_runtime) fn new(
        runtime: &Arc<PhysicalStoreWorkRuntime>,
        generation: crate::physical_runtime::LifecycleGeneration,
        physical: PhysicalWorkAdmissionAuthority,
        scheduler: PhysicalSchedulerAdmissionOwner,
        record: Arc<RecordWorkAdmission>,
    ) -> Self {
        Self {
            runtime: Arc::downgrade(runtime),
            execution: PhysicalStoreWorkRuntime::execution(runtime, generation),
            physical,
            scheduler,
            record,
        }
    }

    pub(in crate::physical_runtime) fn execute(
        &self,
        scope: PhysicalRootPublicationWorkScope,
    ) -> Result<SettledPhysicalWork, PhysicalRootPublicationWorkFailure> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(PhysicalRootPublicationWorkFailure::RuntimeReleased)?;
        let request = PhysicalMutationWorkRequest::root_publication(
            scope,
            self.record.root_publication_basis(),
            self.record.security(),
        )
        .map_err(PhysicalRootPublicationWorkFailure::SubmissionDenied)?;
        let receipt = match runtime
            .submission
            .mutation_submission()
            .submit(request)
            .into_raw()
        {
            TransitionOutcome::Success(receipt) => receipt,
            TransitionOutcome::Denied(denial) => {
                return Err(PhysicalRootPublicationWorkFailure::SubmissionDenied(denial))
            }
            TransitionOutcome::Deferred(deferred) => {
                return Err(PhysicalRootPublicationWorkFailure::SubmissionDeferred(
                    deferred,
                ))
            }
            TransitionOutcome::Stale(stale) => {
                return Err(PhysicalRootPublicationWorkFailure::SubmissionStale(stale))
            }
            TransitionOutcome::RebindRequired(rebind) => match rebind {},
            TransitionOutcome::Failed(failure) => {
                return Err(PhysicalRootPublicationWorkFailure::SubmissionFailed(
                    failure,
                ))
            }
        };
        let admitted = PhysicalWorkAdmission::admit(
            &runtime.submission,
            receipt,
            &self.physical,
            &runtime.health,
        )
        .map_err(PhysicalRootPublicationWorkFailure::PreEffect)?;
        let ready = match runtime
            .signal
            .request(admitted)
            .map_err(PhysicalRootPublicationWorkFailure::PreEffect)?
        {
            PhysicalWorkReadiness::Ready(ready) => ready,
            PhysicalWorkReadiness::Blocked(_) => {
                return Err(PhysicalRootPublicationWorkFailure::DependencyBlocked)
            }
        };
        let (reservation, backend) = self.scheduler_admission(scope.action())?;
        let demand = PhysicalSchedulerDemand::foreground(ready, reservation, None)
            .map_err(PhysicalRootPublicationWorkFailure::Scheduler)?;
        PhysicalWorkAdmission::require_current(
            &runtime.submission,
            demand.intent(),
            &runtime.health,
        )
        .map_err(PhysicalRootPublicationWorkFailure::PreEffect)?;
        let policy =
            crate::physical_runtime::record_serving::admit_record_queue_policy(demand.queue_work());
        let work = PhysicalWorkScheduler::admit(self.scheduler.effects(), demand, &backend, policy)
            .map_err(PhysicalRootPublicationWorkFailure::Scheduler)?;
        let command = PhysicalExecutorCommand::root_publication_effect(work)
            .map_err(PhysicalRootPublicationWorkFailure::Command)?;
        let settled = self
            .execution
            .execute_physical_work(command)
            .map_err(PhysicalRootPublicationWorkFailure::PreEffect)?
            .into_settled();
        Ok(settled)
    }

    fn scheduler_admission(
        &self,
        action: PhysicalRootPublicationWorkAction,
    ) -> Result<
        (
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation,
            worth_store_io_scheduler::IoSchedulerBackendCapabilityAdmission,
        ),
        PhysicalRootPublicationWorkFailure,
    > {
        let security = self.record.scheduler_security();
        match action {
            PhysicalRootPublicationWorkAction::SynchronizeCandidateArtifact { .. } => self
                .scheduler
                .root_candidate_sync(security)
                .map(|admission| admission.into_parts()),
            PhysicalRootPublicationWorkAction::ReplaceBootstrapCatalog => self
                .scheduler
                .root_catalog_replacement(security)
                .map(|admission| admission.into_parts()),
            PhysicalRootPublicationWorkAction::SynchronizeParentNamespace => self
                .scheduler
                .root_namespace_sync(security)
                .map(|admission| admission.into_parts()),
        }
        .map_err(PhysicalRootPublicationWorkFailure::SchedulerReservation)
    }

    /// Admits one record-artifact publication effect on the ordinary write lane.
    ///
    /// Retirement uses this for segment removal and the following family sync.
    /// The media effect still synchronizes the parent directory before completion.
    pub(in crate::physical_runtime) fn execute_record_effect(
        &self,
        artifact: RecordArtifactFile,
        effect: PhysicalPublicationEffect,
        bytes: u64,
        permit: Option<crate::physical_runtime::durability::RetirementRemovalPermit>,
    ) -> Result<SettledPhysicalWork, PhysicalRootPublicationWorkFailure> {
        let work = self.admit_record_work(artifact, effect, bytes)?;
        let command = match effect {
            PhysicalPublicationEffect::RemoveArtifact => {
                let permit = permit.ok_or(PhysicalRootPublicationWorkFailure::Command(
                    crate::physical_runtime::PhysicalExecutorCommandDenial::RetirementRemovalRequiresClaim,
                ))?;
                PhysicalExecutorCommand::retirement_removal(work, permit)
            }
            _ => PhysicalExecutorCommand::publication_effect(work, effect),
        }
        .map_err(PhysicalRootPublicationWorkFailure::Command)?;
        self.execution
            .execute_physical_work(command)
            .map_err(PhysicalRootPublicationWorkFailure::PreEffect)
            .map(|outcome| outcome.into_settled())
    }

    fn admit_record_work(
        &self,
        artifact: RecordArtifactFile,
        effect: PhysicalPublicationEffect,
        bytes: u64,
    ) -> Result<
        crate::physical_runtime::ResourceAdmittedPhysicalWork,
        PhysicalRootPublicationWorkFailure,
    > {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(PhysicalRootPublicationWorkFailure::RuntimeReleased)?;
        let scope = match effect {
            PhysicalPublicationEffect::RemoveArtifact => {
                PhysicalWorkScope::artifact_removal(artifact)
            }
            PhysicalPublicationEffect::SynchronizeArtifact
            | PhysicalPublicationEffect::SynchronizeArtifactParent
            | PhysicalPublicationEffect::ReplaceCatalog
            | PhysicalPublicationEffect::SynchronizeRecordFamily => {
                PhysicalWorkScope::artifact(artifact)
            }
        };
        let request = PhysicalMutationWorkRequest::publication(
            scope,
            self.record
                .mutation_basis(crate::physical_runtime::record_serving::RecordPublicationStage::NamespaceSynchronization),
            self.record.security(),
            ArtifactRangeWriteDurabilityRequirement::FileDataSynchronization,
        )
        .map_err(PhysicalRootPublicationWorkFailure::SubmissionDenied)?;
        let receipt = match runtime
            .submission
            .mutation_submission()
            .submit(request)
            .into_raw()
        {
            TransitionOutcome::Success(receipt) => receipt,
            TransitionOutcome::Denied(denial) => {
                return Err(PhysicalRootPublicationWorkFailure::SubmissionDenied(denial))
            }
            TransitionOutcome::Deferred(deferred) => {
                return Err(PhysicalRootPublicationWorkFailure::SubmissionDeferred(deferred))
            }
            TransitionOutcome::Stale(stale) => {
                return Err(PhysicalRootPublicationWorkFailure::SubmissionStale(stale))
            }
            TransitionOutcome::RebindRequired(rebind) => match rebind {},
            TransitionOutcome::Failed(failure) => {
                return Err(PhysicalRootPublicationWorkFailure::SubmissionFailed(failure))
            }
        };
        let admitted = PhysicalWorkAdmission::admit(
            &runtime.submission,
            receipt,
            &self.physical,
            &runtime.health,
        )
        .map_err(PhysicalRootPublicationWorkFailure::PreEffect)?;
        let ready = match runtime
            .signal
            .request(admitted)
            .map_err(PhysicalRootPublicationWorkFailure::PreEffect)?
        {
            PhysicalWorkReadiness::Ready(ready) => ready,
            PhysicalWorkReadiness::Blocked(_) => {
                return Err(PhysicalRootPublicationWorkFailure::DependencyBlocked)
            }
        };
        let (reservation, backend) = self
            .scheduler
            .record_generation_removal(self.record.scheduler_security(), bytes)
            .map_err(PhysicalRootPublicationWorkFailure::SchedulerReservation)?;
        let demand = PhysicalSchedulerDemand::foreground(ready, reservation, None)
            .map_err(PhysicalRootPublicationWorkFailure::Scheduler)?;
        PhysicalWorkAdmission::require_current(
            &runtime.submission,
            demand.intent(),
            &runtime.health,
        )
        .map_err(PhysicalRootPublicationWorkFailure::PreEffect)?;
        let policy =
            crate::physical_runtime::record_serving::admit_record_queue_policy(demand.queue_work());
        PhysicalWorkScheduler::admit(self.scheduler.effects(), demand, &backend, policy)
            .map_err(PhysicalRootPublicationWorkFailure::Scheduler)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn certification_public_removal_rejected(
        &self,
        artifact: RecordArtifactFile,
    ) -> Result<bool, PhysicalRootPublicationWorkFailure> {
        let work = self.admit_record_work(artifact, PhysicalPublicationEffect::RemoveArtifact, 1)?;
        Ok(PhysicalExecutorCommand::publication_effect(
            work,
            PhysicalPublicationEffect::RemoveArtifact,
        )
        .is_err())
    }
}
