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

pub struct ApplicationHomeRecoveryCustody {
    location: std::path::PathBuf,
    descriptor: crate::descriptor::HomeDescriptor,
    checkpoint: Result<crate::ApplicationHomeCheckpoint, crate::ApplicationHomeCheckpointDenial>,
    physical: PhysicalCloseCustody,
    native: Option<worth_store_relational::ClosingRelationalSection>,
    checkpoint_owner: Arc<crate::checkpoint::owner::HomeCheckpointOwner>,
}

impl ApplicationHomeRecoveryCustody {
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
            native: Some(native),
            checkpoint_owner: context.checkpoint,
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
        if self.native.is_some() {
            let PhysicalCloseCustody::Retained(serving) = &self.physical else {
                unreachable!("Native teardown precedes physical close start")
            };
            use crate::checkpoint::close_settlement::{self, CloseCheckpointWaitDenial};
            match close_settlement::settle(serving, &self.checkpoint_owner, execution) {
                Ok(Some(checkpoint)) => self.checkpoint = checkpoint,
                Ok(None) => {}
                Err(CloseCheckpointWaitDenial::Interrupted(cause)) => {
                    return self.refused(ApplicationHomeCloseDenial::Execution(cause))
                }
                Err(CloseCheckpointWaitDenial::RecoveryRequired(observation)) => {
                    return self.refused(ApplicationHomeCloseDenial::CheckpointRecoveryRequired(
                        observation,
                    ))
                }
            }
            self.native
                .take()
                .expect("stopped Native owner is retained until settlement")
                .finish();
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
