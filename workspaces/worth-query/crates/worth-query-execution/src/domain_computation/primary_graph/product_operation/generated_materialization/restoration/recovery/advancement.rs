//! Admission and custody progression under one runtime-issued advancement.
use crate::domain_computation::primary_graph::product_operation::generated_materialization::restoration::settlement::SettlementParentOps;
use super::super::WorthQueryRestoredGeneratedOutput;
use super::*;

pub trait AdvancementOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    fn continue_generated_output_restoration_recovery(
        &self,
        recovery: WorthQueryGeneratedOutputRestorationRecovery,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<
        WorthQueryRestoredGeneratedOutput,
        WorthQueryGeneratedOutputRestorationRecoveryFailure,
    > {
        let mut retirements = OwnerCleanupRetirements::default();
        let mut retained = Some(recovery);
        let mut attempted = None;
        let result = self
            .application_runtime()
            .with_application_advancement(request, |phase| {
                attempted = Some((|| {
                    let execution = match phase.execution_request_for(self.product_runtime()) {
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
                    let execution = &execution;
                    let recovery = retained.take().expect("one admitted recovery");

                    if !restoration_retry_matches(
                        self.product_operation_host(),
                        recovery.state.retry(),
                    ) {
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
                            product.into_recovery(),
                            retry,
                            needs_settlement,
                            request,
                            &mut retirements,
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
                            recovery,
                            retry,
                            needs_settlement,
                            request,
                            &mut retirements,
                        )
                    }
                    RecoveryState::HandoffWorld { old, next, retry } => self
                        .finish_restoration_handoff(execution, old, next, retry, &mut retirements),
                    RecoveryState::HandoffCleanup {
                        cleanup,
                        next,
                        retry,
                    } => match cleanup
                        .retry_in_request_with_retirements(execution, &mut retirements)
                    {
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
                    } => self.finish_restoration_published(
                        execution,
                        recovery,
                        settlement,
                        &mut retirements,
                    ),
                    RecoveryState::PublishedCleanup {
                        cleanup,
                        settlement,
                    } => match cleanup
                        .retry_in_request_with_retirements(execution, &mut retirements)
                    {
                        Ok(_) => Ok(settlement.finish(execution, self.product_operation_host())),
                        Err(failed) => Err(failure(
                            WorthQueryGeneratedOutputRestorationRecoveryStage::OwnerCleanup,
                            RecoveryState::PublishedCleanup {
                                cleanup: failed.into_cleanup(),
                                settlement,
                            },
                        )),
                    },
                }
                })());
            });
        if let Some(actual) = attempted {
            return actual;
        }
        match result {
            Ok(()) => unreachable!("completed recovery callback ran"),
            Err(cause) => Err(failure(
                WorthQueryGeneratedOutputRestorationRecoveryStage::ExecutionDenied(cause),
                retained.expect("refusal precedes admission").state,
            )),
        }
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                Schema,
            > + ?Sized,
    > AdvancementOps<Schema> for HostRuntime
{
}
