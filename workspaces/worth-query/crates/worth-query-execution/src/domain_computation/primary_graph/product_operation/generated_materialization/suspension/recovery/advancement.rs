//! Request admission and one owned generated recovery step.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Continues retained recovery with the caller's explicit physical request.
    pub fn continue_generated_output_suspension_recovery<Producer>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recovery: WorthQueryGeneratedOutputSuspensionRecovery,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQuerySuspendedGeneratedOutput,
        WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    >
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        self.continue_suspension_recovery_with::<Producer>(Some(execution), recovery, request)
    }

    /// Continues the exact retained recovery under the installed runtime's
    /// request policy. Opening refusal returns the original recovery owner.
    pub fn resume_generated_output_suspension_recovery<Producer>(
        &self,
        recovery: WorthQueryGeneratedOutputSuspensionRecovery,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQuerySuspendedGeneratedOutput,
        WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    >
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        self.continue_suspension_recovery_with::<Producer>(None, recovery, request)
    }

    fn continue_suspension_recovery_with<Producer>(
        &self,
        execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        recovery: WorthQueryGeneratedOutputSuspensionRecovery,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQuerySuspendedGeneratedOutput,
        WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    >
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        let mut retained = Some(recovery);
        let result = self.with_application_advancement(request, |phase| {
            let phase = &phase;
            let admitted_execution = match phase.execution_request_for(&self.product_runtime) {
                Ok(execution) => execution,
                Err(cause) => {
                    return Err(failure(
                        WorthQueryGeneratedOutputSuspensionRecoveryStage::ExecutionDenied(
                            cause.into(),
                        ),
                        retained.take().expect("one refused recovery").state,
                    ));
                }
            };
            let execution = execution.unwrap_or(&admitted_execution);
            let recovery = retained.take().expect("one admitted recovery");

            if !retry_matches::<Schema, Producer>(self, recovery.state.retry()) {
                return Err(failure(
                    WorthQueryGeneratedOutputSuspensionRecoveryStage::StaleOccurrence,
                    recovery.state,
                ));
            }
            match recovery.state {
                RecoveryState::Unpublished { product, retry } => {
                    let needs_settlement = product.relational_requires_settlement();
                    self.continue_world(
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
                                WorthQueryGeneratedOutputSuspensionRecoveryStage::Inspection,
                                RecoveryState::World { recovery, retry },
                            ));
                        }
                    };
                    self.continue_world(
                        execution,
                        phase,
                        recovery,
                        retry,
                        needs_settlement,
                        request,
                    )
                }
                RecoveryState::HandoffWorld { old, next, retry } => {
                    self.finish_handoff(old, next, retry)
                }
                RecoveryState::HandoffCleanup {
                    cleanup,
                    next,
                    retry,
                } => match cleanup.retry() {
                    Ok(_) => Err(failure(
                        WorthQueryGeneratedOutputSuspensionRecoveryStage::AdoptionPublication,
                        RecoveryState::World {
                            recovery: next,
                            retry,
                        },
                    )),
                    Err(failed) => Err(failure(
                        WorthQueryGeneratedOutputSuspensionRecoveryStage::OwnerCleanup,
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
                } => self.finish_published(recovery, settlement),
                RecoveryState::PublishedCleanup {
                    cleanup,
                    settlement,
                } => match cleanup.retry() {
                    Ok(_) => Ok(settlement.finish()),
                    Err(failed) => Err(failure(
                        WorthQueryGeneratedOutputSuspensionRecoveryStage::OwnerCleanup,
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
                WorthQueryGeneratedOutputSuspensionRecoveryStage::ExecutionDenied(cause),
                retained.expect("refusal precedes admission").state,
            )),
        }
    }
}
