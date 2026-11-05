//! Admission and custody progression under one runtime-issued advancement.
use crate::domain_computation::primary_graph::product_operation::generated_materialization::suspension::recovery::settlement::RecoverySettlementParentOps;
use super::*;

pub trait RecoveryAdvancementOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    fn continue_generated_output_suspension_recovery<Producer>(
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
                                WorthQueryGeneratedOutputSuspensionRecoveryStage::ExecutionDenied(
                                    cause.into(),
                                ),
                                retained.take().expect("one refused recovery").state,
                            ));
                        }
                    };
                    let execution = &execution;
                    let recovery = retained.take().expect("one admitted recovery");

                    if !retry_matches::<Schema, Producer>(
                        self.product_operation_host(),
                        recovery.state.retry(),
                    ) {
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
                                    WorthQueryGeneratedOutputSuspensionRecoveryStage::Inspection,
                                    RecoveryState::World { recovery, retry },
                                ));
                            }
                        };
                        self.continue_world(
                            execution,
                            recovery,
                            retry,
                            needs_settlement,
                            request,
                            &mut retirements,
                        )
                    }
                    RecoveryState::HandoffWorld { old, next, retry } => {
                        self.finish_handoff(execution, old, next, retry, &mut retirements)
                    }
                    RecoveryState::HandoffCleanup {
                        cleanup,
                        next,
                        retry,
                    } => match cleanup
                        .retry_in_request_with_retirements(execution, &mut retirements)
                    {
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
                    } => self.finish_published(execution, recovery, settlement, &mut retirements),
                    RecoveryState::PublishedCleanup {
                        cleanup,
                        settlement,
                    } => match cleanup
                        .retry_in_request_with_retirements(execution, &mut retirements)
                    {
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
                })());
            });
        if let Some(actual) = attempted {
            return actual;
        }
        match result {
            Ok(()) => unreachable!("completed recovery callback ran"),
            Err(cause) => Err(failure(
                WorthQueryGeneratedOutputSuspensionRecoveryStage::ExecutionDenied(cause),
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
    > RecoveryAdvancementOps<Schema> for HostRuntime
{
}
