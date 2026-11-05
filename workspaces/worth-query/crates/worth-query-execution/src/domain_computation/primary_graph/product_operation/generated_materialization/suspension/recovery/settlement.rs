use crate::domain_computation::execution_runtime::product_world::OwnerCleanupRetirements;
use worth_query_installation::facade::ApplicationSchema;

use super::{
    failure, RecoveryState, SuspensionRetry, WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    WorthQueryGeneratedOutputSuspensionRecoveryStage,
};
use crate::domain_computation::WorthQueryProductUnpublishedRecovery;

pub(super) struct PublishedSuspensionSettlement {
    pub(super) publication:
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    pub(super) commit: worth_relational::facade::transactions::CommitResult,
    pub(super) retry: SuspensionRetry,
}

impl PublishedSuspensionSettlement {
    pub(super) fn finish(self) -> super::WorthQuerySuspendedGeneratedOutput {
        let Self {
            publication,
            commit,
            retry,
        } = self;
        super::super::super::suspended_output(
            publication,
            retry.branch,
            retry.correspondence,
            retry.producer,
            retry.completion,
            commit,
        )
    }
}

pub(super) trait RecoverySettlementParentOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    fn finish_handoff(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        old: WorthQueryProductUnpublishedRecovery,
        next: WorthQueryProductUnpublishedRecovery,
        retry: SuspensionRetry,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<
        super::WorthQuerySuspendedGeneratedOutput,
        WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    > {
        match self.application_runtime().release_product_recovery_in_request(execution, old, 0, retirements) {
            Ok(_) => Err(failure(
                WorthQueryGeneratedOutputSuspensionRecoveryStage::AdoptionPublication,
                RecoveryState::World {
                    recovery: next,
                    retry,
                },
            )),
            Err(
                crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(
                    failed,
                ),
            ) => Err(failure(
                WorthQueryGeneratedOutputSuspensionRecoveryStage::RecoveryHandoff,
                RecoveryState::HandoffWorld {
                    old: failed.into_recovery(),
                    next,
                    retry,
                },
            )),
            Err(
                crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup(
                    failed,
                ),
            ) => Err(failure(
                WorthQueryGeneratedOutputSuspensionRecoveryStage::OwnerCleanup,
                RecoveryState::HandoffCleanup {
                    cleanup: failed.into_cleanup(),
                    next,
                    retry,
                },
            )),
        }
    }

    fn finish_published(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recovery: WorthQueryProductUnpublishedRecovery,
        settlement: PublishedSuspensionSettlement,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<
        super::WorthQuerySuspendedGeneratedOutput,
        WorthQueryGeneratedOutputSuspensionRecoveryFailure,
    > {
        match self.application_runtime().release_product_recovery_in_request(execution, recovery, 0, retirements) {
            Ok(_) => Ok(settlement.finish()),
            Err(
                crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(
                    failed,
                ),
            ) => Err(failure(
                WorthQueryGeneratedOutputSuspensionRecoveryStage::RecoveryHandoff,
                RecoveryState::PublishedWorld {
                    recovery: failed.into_recovery(),
                    settlement,
                },
            )),
            Err(
                crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup(
                    failed,
                ),
            ) => Err(failure(
                WorthQueryGeneratedOutputSuspensionRecoveryStage::OwnerCleanup,
                RecoveryState::PublishedCleanup {
                    cleanup: failed.into_cleanup(),
                    settlement,
                },
            )),
        }
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                Schema,
            > + ?Sized,
    > RecoverySettlementParentOps<Schema> for HostRuntime
{
}
