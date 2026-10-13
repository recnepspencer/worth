//! Request admission and one owned generated recovery step.
use super::super::WorthQueryRestoredGeneratedOutput;
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Continues retained recovery with the caller's explicit physical request.
    pub fn continue_generated_output_restoration_recovery(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recovery: WorthQueryGeneratedOutputRestorationRecovery,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQueryRestoredGeneratedOutput,
        WorthQueryGeneratedOutputRestorationRecoveryFailure,
    > {
        self.continue_restoration_recovery_with(Some(execution), recovery, request)
    }

    /// Continues the exact retained recovery under the installed runtime's
    /// request policy. Opening refusal returns the original recovery owner.
    pub fn resume_generated_output_restoration_recovery(
        &self,
        recovery: WorthQueryGeneratedOutputRestorationRecovery,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQueryRestoredGeneratedOutput,
        WorthQueryGeneratedOutputRestorationRecoveryFailure,
    > {
        self.continue_restoration_recovery_with(None, recovery, request)
    }

    fn continue_restoration_recovery_with(
        &self,
        execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        recovery: WorthQueryGeneratedOutputRestorationRecovery,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQueryRestoredGeneratedOutput,
        WorthQueryGeneratedOutputRestorationRecoveryFailure,
    > {
        let mut retained = Some(recovery);
        let result = self.with_application_advancement(request, |phase| {
            let phase = &phase;
            let admitted_execution = match phase.execution_request_for(&self.product_runtime) {
                Ok(execution) => execution,
                Err(cause) => {
                    return Err(failure(
                        WorthQueryGeneratedOutputRestorationRecoveryStage::ExecutionDenied(
                            cause.into(),
                        ),
                        retained.take().expect("one refused recovery").state,
                    ));
                }
            };
            let execution = execution.unwrap_or(&admitted_execution);
            let recovery = retained.take().expect("one admitted recovery");

            if !restoration_retry_matches(self, recovery.state.retry()) {
                return Err(failure(
                    WorthQueryGeneratedOutputRestorationRecoveryStage::StaleOccurrence,
                    recovery.state,
                ));
            }
            match recovery.state {
                RecoveryState::Unpublished { unpublished } => {
                    let (product, retry) = unpublished.into_parts();
                    let needs_settlement = product.relational_requires_settlement();
                    self.continue_restoration_world(
                        execution,
                        phase,
                        product.into_recovery(),
                        retry,
                        needs_settlement,
                        request,
                    )
                }
                RecoveryState::World { recovery, retry } => {
                    let needs_settlement = match recovery.inspect() {
                        Ok(product) => product.relational_requires_settlement(),
                        Err(_) => {
                            return Err(failure(
                                WorthQueryGeneratedOutputRestorationRecoveryStage::Inspection,
                                RecoveryState::World { recovery, retry },
                            ));
                        }
                    };
                    self.continue_restoration_world(
                        execution,
                        phase,
                        recovery,
                        retry,
                        needs_settlement,
                        request,
                    )
                }
                RecoveryState::HandoffWorld { old, next, retry } => {
                    self.finish_restoration_handoff(old, next, retry)
                }
                RecoveryState::HandoffCleanup {
                    cleanup,
                    next,
                    retry,
                } => match cleanup.retry() {
                    Ok(_) => Err(failure(
                        WorthQueryGeneratedOutputRestorationRecoveryStage::AdoptionPublication,
                        RecoveryState::World {
                            recovery: next,
                            retry,
                        },
                    )),
                    Err(failed) => Err(failure(
                        WorthQueryGeneratedOutputRestorationRecoveryStage::OwnerCleanup,
                        RecoveryState::HandoffCleanup {
                            cleanup: failed.into_cleanup(),
                            next,
                            retry,
                        },
                    )),
                },
                RecoveryState::PublishedWorld {
                    recovery,
                    settlement,
                } => self.finish_restoration_published(execution, recovery, settlement),
                RecoveryState::PublishedCleanup {
                    cleanup,
                    settlement,
                } => match cleanup.retry() {
                    Ok(_) => Ok(settlement.finish(execution, self)),
                    Err(failed) => Err(failure(
                        WorthQueryGeneratedOutputRestorationRecoveryStage::OwnerCleanup,
                        RecoveryState::PublishedCleanup {
                            cleanup: failed.into_cleanup(),
                            settlement,
                        },
                    )),
                },
            }
        });
        match result {
            Ok(result) => result,
            Err(cause) => Err(failure(
                WorthQueryGeneratedOutputRestorationRecoveryStage::ExecutionDenied(cause),
                retained.expect("refusal precedes admission").state,
            )),
        }
    }
}
