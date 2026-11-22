//! Owned physical custody after the native owner has stopped serving.
use super::start::PhysicalCloseStart;
use super::{ApplicationHomeCloseDenial, ApplicationHomeCloseOutcome, ApplicationHomeClosePosture};
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelStop, WorkCeilingDenial};
use worth_store::physical_runtime::{
    PhysicalStoreCloseFailure, PhysicalStoreCloseObservation, PhysicalStoreCloseOutcome,
    PhysicalStoreCloseTask, PhysicalStoreCloseTaskPoll, ServingPhysicalRuntime,
};
mod native_retirement;
mod world_retirement;
use world_retirement::WorldOwnerCloseCustody;
mod physical_retirement;

enum PhysicalCloseCustody {
    Retained(Arc<ServingPhysicalRuntime>),
    Running(Box<PhysicalStoreCloseTask>),
    InspectionRequired(Box<PhysicalStoreCloseOutcome>),
    Failed(Box<PhysicalStoreCloseFailure>),
}
enum NativeCloseCustody {
    Serving(worth_store_relational::RelationalStoreSection),
    Prepared(worth_store_relational::ClosingRelationalSection),
    NeverAttached,
    Retired(worth_store_relational::RetiredRelationalSectionAllocation),
}

pub struct ApplicationHomeRecoveryCustody {
    inner: Box<ApplicationHomeRecoveryCustodyBody>,
}

struct ApplicationHomeRecoveryCustodyBody {
    location: std::path::PathBuf,
    descriptor: crate::descriptor::HomeDescriptor,
    checkpoint: super::checkpoint_result::HomeCloseCheckpointResult,
    physical: Option<PhysicalCloseCustody>,
    signal_medium: Option<std::sync::Weak<crate::signal_medium::StoreSignalMedium>>,
    signal_owner_retirement:
        Option<worth_signal::facade::durability::SignalRetainedAuthoredHomeAllocation>,
    native: NativeCloseCustody,
    world_owner: WorldOwnerCloseCustody,
    world_origin_retirement: Option<worth_runtime_world::facade::WorldCheckpointOriginRetirement>,
    world_close_report: Option<worth_runtime_world::facade::RuntimeWorldCloseReport>,
    world_composition: Option<crate::world_section::HomeBridgeCloseCustody>,
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
        let mut checkpoint = context
            .world_close_checkpoint
            .into_inner()
            .unwrap_or_else(|error| error.into_inner());
        checkpoint.store(Err(
            crate::ApplicationHomeCheckpointDenial::NativeSectionNeverAttached,
        ));
        Self {
            inner: Box::new(ApplicationHomeRecoveryCustodyBody {
                location: context.location,
                descriptor: context.descriptor,
                checkpoint,
                signal_medium: context
                    .signal_close_probe
                    .into_inner()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
                signal_owner_retirement: context.signal_owner_retirement,
                physical: Some(PhysicalCloseCustody::Retained(context.serving)),
                native: NativeCloseCustody::NeverAttached,
                world_owner: WorldOwnerCloseCustody::from_original(context.world_owner_retirement),
                world_origin_retirement: context.world_origin_retirement,
                world_close_report: None,
                world_composition: None,
                checkpoint_owner: context.checkpoint,
                world_publications: context.world_publications,
            }),
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
        let mut stored = context
            .world_close_checkpoint
            .into_inner()
            .unwrap_or_else(|error| error.into_inner());
        stored.store(checkpoint);
        Self {
            inner: Box::new(ApplicationHomeRecoveryCustodyBody {
                location: context.location,
                descriptor: context.descriptor,
                checkpoint: stored,
                signal_medium: context
                    .signal_close_probe
                    .into_inner()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
                signal_owner_retirement: context.signal_owner_retirement,
                physical: Some(PhysicalCloseCustody::Retained(context.serving)),
                native: NativeCloseCustody::Prepared(native),
                world_owner: WorldOwnerCloseCustody::from_original(context.world_owner_retirement),
                world_origin_retirement: context.world_origin_retirement,
                world_close_report: context.world_close_report,
                world_composition: None,
                checkpoint_owner: context.checkpoint,
                world_publications: context.world_publications,
            }),
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
        let checkpoint = context
            .world_close_checkpoint
            .into_inner()
            .unwrap_or_else(|error| error.into_inner());
        // Preserve the seal's existing settled-result invariant before Native
        // teardown; only this SAME prepaid backing moves into recovery custody.
        let _ = checkpoint.result();
        Self {
            inner: Box::new(ApplicationHomeRecoveryCustodyBody {
                location: context.location,
                descriptor: context.descriptor,
                checkpoint,
                signal_medium: context
                    .signal_close_probe
                    .into_inner()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
                signal_owner_retirement: context.signal_owner_retirement,
                physical: Some(PhysicalCloseCustody::Retained(context.serving)),
                native: NativeCloseCustody::Serving(relational),
                world_owner: WorldOwnerCloseCustody::from_original(context.world_owner_retirement),
                world_origin_retirement: context.world_origin_retirement,
                world_close_report: context.world_close_report,
                world_composition: world_composition
                    .map(crate::world_section::HomeBridgeCloseCustody::Serving),
                checkpoint_owner: context.checkpoint,
                world_publications: context.world_publications,
            }),
        }
    }

    pub fn world_close_report(
        &self,
    ) -> Option<&worth_runtime_world::facade::RuntimeWorldCloseReport> {
        self.inner.world_close_report.as_ref()
    }
    pub fn world_publication_posture(
        &self,
    ) -> worth_runtime_world::facade::WorldPublicationTaskPosture {
        self.inner.world_publications.posture()
    }

    /// Actual Native registry observation while the retained section is
    /// Serving. None means this phase does not expose that observation.
    pub fn native_active_snapshot_count(&self) -> Option<usize> {
        match &self.inner.native {
            NativeCloseCustody::Serving(native) => {
                Some(native.runtime().retention().active_snapshot_count())
            }
            NativeCloseCustody::Prepared(_)
            | NativeCloseCustody::Retired(_)
            | NativeCloseCustody::NeverAttached => None,
        }
    }

    pub fn location(&self) -> &std::path::Path {
        &self.inner.location
    }
    pub fn home_identity(&self) -> crate::HomeIdentity {
        self.inner.descriptor.identity
    }
    pub fn checkpoint_result(
        &self,
    ) -> &Result<crate::ApplicationHomeCheckpoint, crate::ApplicationHomeCheckpointDenial> {
        self.inner.checkpoint.result()
    }
    pub fn physical_close_observation(&self) -> Option<PhysicalStoreCloseObservation> {
        match self.inner.physical.as_ref().expect("live physical custody") {
            PhysicalCloseCustody::Retained(_) | PhysicalCloseCustody::InspectionRequired(_) => None,
            PhysicalCloseCustody::Running(task) => task.observation().cloned(),
            PhysicalCloseCustody::Failed(failure) => failure.observation().cloned(),
        }
    }
    pub fn inspection(&self) -> Option<&PhysicalStoreCloseOutcome> {
        match self.inner.physical.as_ref().expect("live physical custody") {
            PhysicalCloseCustody::InspectionRequired(outcome) => Some(outcome),
            PhysicalCloseCustody::Failed(failure) => failure.actual_outcome(),
            PhysicalCloseCustody::Retained(_) | PhysicalCloseCustody::Running(_) => None,
        }
    }
    /// Consume the explicit inspection posture into Store's actual outcome and
    /// any recovery handoff it owns. Other postures are returned unchanged.
    pub fn into_physical_close_outcome(mut self) -> Result<PhysicalStoreCloseOutcome, Self> {
        match self.inner.physical.take().expect("live physical custody") {
            PhysicalCloseCustody::InspectionRequired(outcome) => Ok(*outcome),
            physical => {
                self.inner.physical = Some(physical);
                Err(self)
            }
        }
    }

    pub fn physical_close_failure(&self) -> Option<&PhysicalStoreCloseFailure> {
        match self.inner.physical.as_ref().expect("live physical custody") {
            PhysicalCloseCustody::Failed(failure) => Some(failure),
            PhysicalCloseCustody::Retained(_)
            | PhysicalCloseCustody::Running(_)
            | PhysicalCloseCustody::InspectionRequired(_) => None,
        }
    }

    /// Move the actual failed task evidence to its caller for inspection.
    pub fn into_physical_close_failure(mut self) -> Result<PhysicalStoreCloseFailure, Self> {
        match self.inner.physical.take().expect("live physical custody") {
            PhysicalCloseCustody::Failed(failure) => Ok(*failure),
            physical => {
                self.inner.physical = Some(physical);
                Err(self)
            }
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
        loop {
            let Some(composition) = self.inner.world_composition.take() else {
                break;
            };
            if matches!(
                composition,
                crate::world_section::HomeBridgeCloseCustody::Retired(_)
            ) {
                self.inner.world_composition = Some(composition);
                break;
            }
            let NativeCloseCustody::Serving(native) = &self.inner.native else {
                self.inner.world_composition = Some(composition);
                return self.refused(ApplicationHomeCloseDenial::WorldCompositionRetained);
            };
            match composition.advance(execution, native.runtime()) {
                Ok((composition, progress)) => {
                    self.inner.world_composition = Some(composition);
                    if progress
                        == worth_runtime_bridge::facade::BridgeHomeRetirementProgress::Complete
                    {
                        break;
                    }
                }
                Err((composition, cause)) => {
                    self.inner.world_composition = Some(composition);
                    return self.refused(cause);
                }
            }
        }
        if let Err(cause) = self.advance_world_owner_close(execution) {
            return self.refused(cause);
        }
        self = match self.advance_native_close(execution) {
            Ok(home) => home,
            Err((home, cause)) => return home.refused(cause),
        };
        self.finish_physical_release(execution, start)
    }

    fn settle_retained_checkpoint(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), ApplicationHomeCloseDenial> {
        let Some(PhysicalCloseCustody::Retained(serving)) = &self.inner.physical else {
            return Err(ApplicationHomeCloseDenial::PhysicalCustodyPhaseMismatch);
        };
        if let Some(checkpoint) = crate::checkpoint::close_settlement::settle(
            serving,
            &self.inner.checkpoint_owner,
            execution,
        )
        .map_err(super::checkpoint_settlement_denial)?
        {
            self.inner.checkpoint.store(checkpoint);
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
