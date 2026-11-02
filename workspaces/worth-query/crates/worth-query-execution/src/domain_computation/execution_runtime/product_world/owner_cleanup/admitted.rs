//! Cleanup borrows the original request's bounded native checkpoint before effects.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::execution_denial;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_execution::ExecutionWorkCeiling;

impl WorthQueryProductBranchOwnerCleanup {
    pub(in crate::domain_computation) fn execution_request_in_advancement<'scope>(
        &self,
        phase: &WorthQueryAdvancementPhase<'scope>,
    ) -> Result<
        worth_execution::ExecutionRequest<'scope, 'scope>,
        crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
    > {
        phase.check_live_for(&self.runtime)?;
        phase.checked_execution_request_for(&self.runtime)
    }

    pub fn retry_in_advancement(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryProductBranchOwnerCleanupFailure,
    > {
        let mut retirements = OwnerCleanupRetirements::default();
        let execution = match self.execution_request_in_advancement(phase) {
            Ok(request) => request,
            Err(denial) => {
                return Err(
                    self.failure(WorthQueryProductBranchOwnerCleanupDenial::Execution(denial))
                )
            }
        };
        self.retry_in_request_with_retirements(&execution, &mut retirements)
    }

    pub(crate) fn retry_in_request_with_retirements(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryProductBranchOwnerCleanupFailure,
    > {
        let mut attempted = None;
        let outcome = execution.in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| {
                    attempted = Some(self.runtime.owner_cleanup.retry_admitted(
                        execution,
                        self.identity,
                        &self.runtime,
                        retirements,
                        |units| {
                            context.checkpoint(units).map_err(|stop| {
                                WorthQueryProductBranchOwnerCleanupDenial::Execution(
                                    execution_denial::advancement_checkpoint_stop(stop),
                                )
                            })
                        },
                    ));
                },
            )
        });
        // Registry completion is an actual effect, not a prospective result that
        // a later original-request poll can turn into an unusable retry handle.
        if matches!(attempted, Some(Ok(_))) {
            return attempted
                .expect("captured completed cleanup")
                .map_err(|denial| self.failure(denial));
        }
        match outcome {
            Ok(Ok(_)) => attempted
                .expect("completed scope ran cleanup")
                .map_err(|denial| self.failure(denial)),
            Ok(Err(worth_execution::WorkCeilingDenial::Stopped(
                worth_execution::MapKernelStop::NestedStopped,
            )))
            | Err(worth_execution::WorkCeilingDenial::Stopped(
                worth_execution::MapKernelStop::NestedStopped,
            )) if attempted.is_some() => attempted
                .expect("actual classified refusal")
                .map_err(|denial| self.failure(denial)),
            Ok(Err(denial)) | Err(denial) => Err(self.failure(
                WorthQueryProductBranchOwnerCleanupDenial::Execution(
                    execution_denial::advancement_scope_stop(denial),
                ),
            )),
        }
    }

    fn failure(
        self,
        denial: WorthQueryProductBranchOwnerCleanupDenial,
    ) -> WorthQueryProductBranchOwnerCleanupFailure {
        WorthQueryProductBranchOwnerCleanupFailure {
            denial,
            cleanup: self,
        }
    }
}
