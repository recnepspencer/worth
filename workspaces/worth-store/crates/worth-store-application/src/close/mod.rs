//! Refused close preserves serving custody or the actual physical close task.
mod posture;
mod start;

pub use posture::ApplicationHomeRecoveryCustody;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_relational::facade::runtime::RelationalRuntimeCloseDenial;
use worth_store::physical_runtime::{
    ClosedRuntime, PhysicalStoreCloseFailureCause, ServingShutdownOutcome,
};

pub enum ApplicationHomeCloseOutcome {
    Closed {
        store: ServingShutdownOutcome<ClosedRuntime>,
        checkpoint:
            Result<crate::ApplicationHomeCheckpoint, crate::ApplicationHomeCheckpointDenial>,
    },
    CloseRefused {
        reason: ApplicationHomeCloseDenial,
        posture: ApplicationHomeClosePosture,
    },
}

#[derive(Debug)]
pub enum ApplicationHomeCloseDenial {
    Execution(WorkCeilingDenial),
    Relational(RelationalRuntimeCloseDenial),
    PhysicalCustodyRetained,
    PhysicalCloseStartFailed {
        kind: std::io::ErrorKind,
        detail: String,
    },
    PhysicalCloseFailed(PhysicalStoreCloseFailureCause),
    PhysicalInspectionRequired,
}

pub enum ApplicationHomeClosePosture {
    Serving(crate::ReadmittedApplicationHome),
    RecoveryRequired(ApplicationHomeRecoveryCustody),
}

impl crate::ReadmittedApplicationHome {
    pub fn close(self, execution: &ExecutionRequest<'_, '_>) -> ApplicationHomeCloseOutcome {
        self.close_start(execution, start::PhysicalCloseStart::Ordinary)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(crate) fn close_observed(
        self,
        execution: &ExecutionRequest<'_, '_>,
        observe: impl FnOnce(&worth_store::physical_runtime::PhysicalStoreClosePlan) + Send + 'static,
    ) -> ApplicationHomeCloseOutcome {
        self.close_start(
            execution,
            start::PhysicalCloseStart::Observed(Box::new(observe)),
        )
    }

    fn close_start(
        self,
        execution: &ExecutionRequest<'_, '_>,
        start: start::PhysicalCloseStart,
    ) -> ApplicationHomeCloseOutcome {
        if let Err(reason) = execution.consult() {
            return ApplicationHomeCloseOutcome::CloseRefused {
                reason: ApplicationHomeCloseDenial::Execution(reason),
                posture: ApplicationHomeClosePosture::Serving(self),
            };
        }
        let Self {
            relational,
            context,
        } = self;
        let closing = match relational.prepare_close() {
            Ok(closing) => closing,
            Err((relational, reason)) => {
                return ApplicationHomeCloseOutcome::CloseRefused {
                    reason: ApplicationHomeCloseDenial::Relational(reason),
                    posture: ApplicationHomeClosePosture::Serving(Self {
                        relational,
                        context,
                    }),
                }
            }
        };
        let checkpoint =
            crate::checkpoint::publication::checkpoint(&context.serving, execution, || {
                closing.prepare_checkpoint(execution)
            });
        // Checkpoint failure never refuses close. Accepted native work was
        // classified by its real owner before teardown; physical completion
        // now remains owned even if the original request stops waiting.
        closing.finish();
        ApplicationHomeRecoveryCustody::retained(context, checkpoint)
            .release_started(execution, start)
    }
}
