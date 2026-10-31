//! Owned physical custody after the native owner has stopped serving.
use super::start::PhysicalCloseStart;
use super::{ApplicationHomeCloseDenial, ApplicationHomeCloseOutcome, ApplicationHomeClosePosture};
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelStop, WorkCeilingDenial};
use worth_store::physical_runtime::{
    PhysicalStoreCloseFailure, PhysicalStoreCloseObservation, PhysicalStoreCloseOutcome,
    PhysicalStoreCloseTask, PhysicalStoreCloseTaskPoll, ServingPhysicalRuntime,
};

enum PhysicalCloseCustody {
    Retained(Arc<ServingPhysicalRuntime>),
    Running(PhysicalStoreCloseTask),
    InspectionRequired(PhysicalStoreCloseOutcome),
    Failed(PhysicalStoreCloseFailure),
}
enum NativeCloseCustody {
    Serving(worth_store_relational::RelationalStoreSection),
    Prepared(worth_store_relational::ClosingRelationalSection),
    Released,
}

pub struct ApplicationHomeRecoveryCustody {
    location: std::path::PathBuf,
    descriptor: crate::descriptor::HomeDescriptor,
    checkpoint: Result<crate::ApplicationHomeCheckpoint, crate::ApplicationHomeCheckpointDenial>,
    physical: PhysicalCloseCustody,
    native: NativeCloseCustody,
    world_close_report: Option<worth_runtime_world::facade::RuntimeWorldCloseReport>,
    world_composition: Option<crate::world_section::ApplicationHomeBridgeComposition>,
    checkpoint_owner: Arc<crate::checkpoint::owner::HomeCheckpointOwner>,
    world_publications: Arc<crate::world_section::WorldPublications>,
}

impl ApplicationHomeRecoveryCustody {
    pub(crate) fn from_pre_service_home(
        context: crate::home_context::HomeContext,
        native: worth_store_relational::ClosingRelationalSection,
    ) -> Self {
        Self::retained(
            context,
            native,
            Err(crate::ApplicationHomeCheckpointDenial::HomePreparationIncomplete),
        )
    }
    pub(crate) fn from_never_attached_home(context: crate::home_context::HomeContext) -> Self {
        Self {
            location: context.location,
            descriptor: context.descriptor,
            checkpoint: Err(crate::ApplicationHomeCheckpointDenial::NativeSectionNeverAttached),
            physical: PhysicalCloseCustody::Retained(context.serving),
            native: NativeCloseCustody::Released,
            world_close_report: None,
            world_composition: None,
            checkpoint_owner: context.checkpoint,
            world_publications: context.world_publications,
        }
    }

    pub(super) fn retained(
        context: crate::home_context::HomeContext,
        native: worth_store_relational::ClosingRelationalSection,
        checkpoint: Result<
            crate::ApplicationHomeCheckpoint,
            crate::ApplicationHomeCheckpointDenial,
        >,
    ) -> Self {
        Self {
            location: context.location,
            descriptor: context.descriptor,
            checkpoint,
            physical: PhysicalCloseCustody::Retained(context.serving),
            native: NativeCloseCustody::Prepared(native),
            world_close_report: context.world_close_report,
            world_composition: None,
            checkpoint_owner: context.checkpoint,
            world_publications: context.world_publications,
        }
    }

    pub(crate) fn after_world_close(
        home: crate::ReadmittedApplicationHome,
        world_composition: Option<crate::world_section::ApplicationHomeBridgeComposition>,
    ) -> Self {
        let crate::ReadmittedApplicationHome {
            context,
            relational,
        } = home;
        Self {
            location: context.location,
            descriptor: context.descriptor,
            checkpoint: Err(crate::ApplicationHomeCheckpointDenial::WorldSectionRequired),
            physical: PhysicalCloseCustody::Retained(context.serving),
            native: NativeCloseCustody::Serving(relational),
            world_close_report: context.world_close_report,
            world_composition,
            checkpoint_owner: context.checkpoint,
            world_publications: context.world_publications,
        }
    }

    pub fn world_close_report(
        &self,
    ) -> Option<&worth_runtime_world::facade::RuntimeWorldCloseReport> {
        self.world_close_report.as_ref()
    }
    pub fn world_publication_posture(
        &self,
    ) -> worth_runtime_world::facade::WorldPublicationTaskPosture {
        self.world_publications.posture()
    }

    /// Actual Native registry observation while the retained section is
    /// Serving. None means this phase does not expose that observation.
    pub fn native_active_snapshot_count(&self) -> Option<usize> {
        match &self.native {
            NativeCloseCustody::Serving(native) => {
                Some(native.runtime().retention().active_snapshot_count())
            }
            NativeCloseCustody::Prepared(_) | NativeCloseCustody::Released => None,
        }
    }

    pub fn location(&self) -> &std::path::Path {
        &self.location
    }
    pub fn home_identity(&self) -> crate::HomeIdentity {
        self.descriptor.identity
    }
    pub fn checkpoint_result(
        &self,
    ) -> &Result<crate::ApplicationHomeCheckpoint, crate::ApplicationHomeCheckpointDenial> {
        &self.checkpoint
    }
    pub fn physical_close_observation(&self) -> Option<PhysicalStoreCloseObservation> {
        match &self.physical {
            PhysicalCloseCustody::Retained(_) | PhysicalCloseCustody::InspectionRequired(_) => None,
            PhysicalCloseCustody::Running(task) => task.observation().cloned(),
            PhysicalCloseCustody::Failed(failure) => failure.observation().cloned(),
        }
    }
    pub fn inspection(&self) -> Option<&PhysicalStoreCloseOutcome> {
        match &self.physical {
            PhysicalCloseCustody::InspectionRequired(outcome) => Some(outcome),
            PhysicalCloseCustody::Failed(failure) => failure.actual_outcome(),
            PhysicalCloseCustody::Retained(_) | PhysicalCloseCustody::Running(_) => None,
        }
    }
    /// Consume the explicit inspection posture into Store's actual outcome and
    /// any recovery handoff it owns. Other postures are returned unchanged.
    pub fn into_physical_close_outcome(self) -> Result<PhysicalStoreCloseOutcome, Self> {
        match self.physical {
            PhysicalCloseCustody::InspectionRequired(outcome) => Ok(outcome),
            physical @ (PhysicalCloseCustody::Retained(_)
            | PhysicalCloseCustody::Running(_)
            | PhysicalCloseCustody::Failed(_)) => Err(Self { physical, ..self }),
        }
    }

    pub fn physical_close_failure(&self) -> Option<&PhysicalStoreCloseFailure> {
        match &self.physical {
            PhysicalCloseCustody::Failed(failure) => Some(failure),
            PhysicalCloseCustody::Retained(_)
            | PhysicalCloseCustody::Running(_)
            | PhysicalCloseCustody::InspectionRequired(_) => None,
        }
    }

    /// Move the actual failed task evidence to its caller for inspection.
    pub fn into_physical_close_failure(self) -> Result<PhysicalStoreCloseFailure, Self> {
        match self.physical {
            PhysicalCloseCustody::Failed(failure) => Ok(failure),
            physical @ (PhysicalCloseCustody::Retained(_)
            | PhysicalCloseCustody::Running(_)
            | PhysicalCloseCustody::InspectionRequired(_)) => Err(Self { physical, ..self }),
        }
    }

    /// Retry the same physical release, retaining a running task rather than
    /// starting another close. The caller supplies its actual retry request.
    pub fn try_release(self, execution: &ExecutionRequest<'_, '_>) -> ApplicationHomeCloseOutcome {
        self.release_started(execution, PhysicalCloseStart::Ordinary)
    }

    pub(super) fn release_started(
        mut self,
        execution: &ExecutionRequest<'_, '_>,
        start: PhysicalCloseStart,
    ) -> ApplicationHomeCloseOutcome {
        if let Err(cause) = interrupted(execution) {
            return self.refused(ApplicationHomeCloseDenial::Execution(cause));
        }
        if let Some(composition) = self.world_composition.take() {
            if let Err(composition) = composition.try_close() {
                self.world_composition = Some(composition);
                return self.refused(ApplicationHomeCloseDenial::WorldCompositionRetained);
            }
        }
        if matches!(self.native, NativeCloseCustody::Serving(_)) {
            if let Err(cause) = self.settle_retained_checkpoint(execution) {
                return self.refused(cause);
            }
        }
        self.native = match std::mem::replace(&mut self.native, NativeCloseCustody::Released) {
            NativeCloseCustody::Serving(native) => match native.prepare_close() {
                Ok(native) => NativeCloseCustody::Prepared(native),
                Err((native, cause)) => {
                    self.native = NativeCloseCustody::Serving(native);
                    return self.refused(ApplicationHomeCloseDenial::Relational(cause));
                }
            },
            phase @ (NativeCloseCustody::Prepared(_) | NativeCloseCustody::Released) => phase,
        };
        if matches!(self.native, NativeCloseCustody::Prepared(_)) {
            if let Err(cause) = self.settle_retained_checkpoint(execution) {
                return self.refused(cause);
            }
            let NativeCloseCustody::Prepared(native) =
                std::mem::replace(&mut self.native, NativeCloseCustody::Released)
            else {
                unreachable!("prepared Native owner is retained until settlement");
            };
            native.finish();
        }
        if let PhysicalCloseCustody::Retained(serving) = self.physical {
            if let Err(cause) = interrupted(execution) {
                return Self {
                    physical: PhysicalCloseCustody::Retained(serving),
                    ..self
                }
                .refused(ApplicationHomeCloseDenial::Execution(cause));
            }
            let serving = match Arc::try_unwrap(serving) {
                Ok(serving) => serving,
                Err(serving) => {
                    return Self {
                        physical: PhysicalCloseCustody::Retained(serving),
                        ..self
                    }
                    .refused(ApplicationHomeCloseDenial::PhysicalCustodyRetained)
                }
            };
            self.physical = match start.start(serving) {
                Ok(task) => PhysicalCloseCustody::Running(task),
                Err(refused) => {
                    let (serving, error) = refused.into_parts();
                    return Self {
                        physical: PhysicalCloseCustody::Retained(Arc::new(serving)),
                        ..self
                    }
                    .refused(
                        ApplicationHomeCloseDenial::PhysicalCloseStartFailed {
                            kind: error.kind(),
                            detail: error.to_string(),
                        },
                    );
                }
            };
        }
        loop {
            self.physical = match self.physical {
                PhysicalCloseCustody::Running(task) => match task.poll() {
                    PhysicalStoreCloseTaskPoll::Pending(task) => {
                        PhysicalCloseCustody::Running(task)
                    }
                    PhysicalStoreCloseTaskPoll::Failed(failure) => {
                        let cause = *failure.cause();
                        return Self {
                            physical: PhysicalCloseCustody::Failed(failure),
                            ..self
                        }
                        .refused(ApplicationHomeCloseDenial::PhysicalCloseFailed(cause));
                    }
                    PhysicalStoreCloseTaskPoll::Completed(outcome) => match outcome {
                        PhysicalStoreCloseOutcome::Closed { shutdown, .. } => {
                            return ApplicationHomeCloseOutcome::Closed {
                                store: shutdown,
                                checkpoint: self.checkpoint,
                            }
                        }
                        outcome @ PhysicalStoreCloseOutcome::InspectionRequired { .. } => {
                            return Self {
                                physical: PhysicalCloseCustody::InspectionRequired(outcome),
                                ..self
                            }
                            .refused(ApplicationHomeCloseDenial::PhysicalInspectionRequired);
                        }
                    },
                },
                PhysicalCloseCustody::InspectionRequired(outcome) => {
                    return Self {
                        physical: PhysicalCloseCustody::InspectionRequired(outcome),
                        ..self
                    }
                    .refused(ApplicationHomeCloseDenial::PhysicalInspectionRequired);
                }
                PhysicalCloseCustody::Failed(failure) => {
                    let cause = *failure.cause();
                    return Self {
                        physical: PhysicalCloseCustody::Failed(failure),
                        ..self
                    }
                    .refused(ApplicationHomeCloseDenial::PhysicalCloseFailed(cause));
                }
                PhysicalCloseCustody::Retained(_) => {
                    unreachable!("retained physical owner was classified before waiting")
                }
            };
            if let Err(cause) = interrupted(execution) {
                return self.refused(ApplicationHomeCloseDenial::Execution(cause));
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    fn settle_retained_checkpoint(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), ApplicationHomeCloseDenial> {
        let PhysicalCloseCustody::Retained(serving) = &self.physical else {
            return Err(ApplicationHomeCloseDenial::PhysicalCustodyRetained);
        };
        if let Some(checkpoint) =
            crate::checkpoint::close_settlement::settle(serving, &self.checkpoint_owner, execution)
                .map_err(super::checkpoint_settlement_denial)?
        {
            self.checkpoint = if self.world_close_report.is_some() {
                Err(crate::ApplicationHomeCheckpointDenial::WorldSectionRequired)
            } else {
                checkpoint
            };
        }
        Ok(())
    }

    fn refused(self, reason: ApplicationHomeCloseDenial) -> ApplicationHomeCloseOutcome {
        ApplicationHomeCloseOutcome::CloseRefused {
            reason,
            posture: ApplicationHomeClosePosture::RecoveryRequired(self),
        }
    }
}

// Releasing an owner remains lawful after checkpoint work/memory exhaustion;
// checkpoint failure cannot refuse close. Cancellation and deadline still stop
// waiting and return actual physical custody. No new allowance is created.
pub(crate) fn interrupted(execution: &ExecutionRequest<'_, '_>) -> Result<(), WorkCeilingDenial> {
    match execution.consult() {
        Ok(()) => Ok(()),
        Err(WorkCeilingDenial::Stopped(
            stop @ (MapKernelStop::Cancelled | MapKernelStop::DeadlineElapsed),
        )) => Err(WorkCeilingDenial::Stopped(stop)),
        Err(WorkCeilingDenial::Stopped(
            MapKernelStop::WorkCounterOverflow
            | MapKernelStop::WorkCeiling
            | MapKernelStop::NestedStopped
            | MapKernelStop::AccountingUnavailable,
        )) => Ok(()),
        Err(cause @ (WorkCeilingDenial::Admission(_) | WorkCeilingDenial::Panicked)) => Err(cause),
    }
}
