//! Refused close preserves serving custody or the actual physical close task.
pub(crate) mod checkpoint_result;
pub(crate) mod posture;
mod start;

pub use posture::ApplicationHomeRecoveryCustody;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_store::physical_runtime::{
    ClosedRuntime, PhysicalStoreCloseFailureCause, ServingShutdownOutcome,
};
use worth_store_relational::RelationalSectionCloseDenial;

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
    WorldCompositionRetained,
    CheckpointSettlement(crate::ApplicationHomeCheckpointDenial),
    WorldResultSnapshot(worth_runtime_world::facade::RuntimeWorldResultSnapshotCleanupDenial),
    World(
        worth_runtime_world::facade::RuntimeWorldServiceDenial<
            worth_runtime_world::facade::RuntimeWorldCloseDenial,
        >,
    ),
    Execution(WorkCeilingDenial),
    Relational(RelationalSectionCloseDenial),
    PhysicalCustodyRetained,
    PhysicalCloseStartFailed {
        kind: std::io::ErrorKind,
        detail: String,
    },
    PhysicalCloseFailed(PhysicalStoreCloseFailureCause),
    PhysicalInspectionRequired,
    CheckpointRecoveryRequired(crate::ApplicationHomeCheckpointRecovery),
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
        if self.context.world_close_report.is_some() {
            return ApplicationHomeRecoveryCustody::after_world_close(self, None)
                .release_started(execution, start);
        }
        if let Err(reason) = posture::interrupted(execution) {
            return ApplicationHomeCloseOutcome::CloseRefused {
                reason: ApplicationHomeCloseDenial::Execution(reason),
                posture: ApplicationHomeClosePosture::Serving(self),
            };
        }
        let Self {
            relational,
            context,
        } = self;
        let prior_checkpoint = match crate::checkpoint::close_settlement::settle(
            &context.serving,
            &context.checkpoint,
            execution,
        ) {
            Ok(result) => result,
            Err(cause) => {
                let reason = checkpoint_settlement_denial(cause);
                return ApplicationHomeCloseOutcome::CloseRefused {
                    reason,
                    posture: ApplicationHomeClosePosture::Serving(Self {
                        relational,
                        context,
                    }),
                };
            }
        };
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
        let checkpoint = if let Some(result) = prior_checkpoint {
            result
        } else {
            crate::checkpoint::publication::worldless_checkpoint(&context.checkpoint, execution)
        };
        // Checkpoint failure never refuses close. Accepted native work was
        // classified by its real owner before teardown; physical completion
        // now remains owned even if the original request stops waiting.
        ApplicationHomeRecoveryCustody::retained(context, closing, checkpoint)
            .release_started(execution, start)
    }
}

pub(crate) fn checkpoint_settlement_denial(
    cause: crate::checkpoint::close_settlement::CloseCheckpointWaitDenial,
) -> ApplicationHomeCloseDenial {
    use crate::checkpoint::close_settlement::CloseCheckpointWaitDenial;
    match cause {
        CloseCheckpointWaitDenial::Interrupted(cause) => {
            ApplicationHomeCloseDenial::Execution(cause)
        }
        CloseCheckpointWaitDenial::RecoveryRequired(observation) => {
            ApplicationHomeCloseDenial::CheckpointRecoveryRequired(observation)
        }
        CloseCheckpointWaitDenial::Acknowledgement(cause) => {
            ApplicationHomeCloseDenial::CheckpointSettlement(cause)
        }
    }
}
