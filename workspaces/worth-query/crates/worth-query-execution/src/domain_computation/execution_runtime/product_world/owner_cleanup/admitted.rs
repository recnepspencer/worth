//! Cleanup borrows the original request's bounded native checkpoint before effects.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::execution_denial;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_execution::ExecutionWorkCeiling;

impl WorthQueryProductBranchOwnerCleanup {
    pub fn retry_in_advancement(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryProductBranchOwnerCleanupFailure,
    > {
        let execution = match phase.checked_execution_request_for(&self.runtime) {
            Ok(request) => request,
            Err(denial) => {
                return Err(
                    self.failure(WorthQueryProductBranchOwnerCleanupDenial::Execution(denial))
                )
            }
        };
        let outcome = execution.in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| {
                    self.runtime.owner_cleanup.retry_admitted(
                        &execution,
                        self.identity,
                        &self.runtime,
                        |units| {
                            context.checkpoint(units).map_err(|stop| {
                                WorthQueryProductBranchOwnerCleanupDenial::Execution(
                                    execution_denial::advancement_checkpoint_stop(stop),
                                )
                            })
                        },
                    )
                },
            )
        });
        match outcome {
            Ok(Ok((Ok(receipt), _))) => Ok(receipt),
            Ok(Ok((Err(denial), _))) => Err(self.failure(denial)),
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
