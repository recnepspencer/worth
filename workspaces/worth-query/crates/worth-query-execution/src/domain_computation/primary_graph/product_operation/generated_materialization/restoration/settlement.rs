use crate::domain_computation::primary_graph::product_operation::generated_materialization::restoration::republication::RepublicationParentOps;
use crate::domain_computation::execution_runtime::product_world::OwnerCleanupRetirements;
use worth_query_installation::facade::ApplicationSchema;

use super::recovery::{
    failure, RecoveryState, RestorationRetry, WorthQueryGeneratedOutputRestorationRecoveryFailure,
    WorthQueryGeneratedOutputRestorationRecoveryStage,
};
use crate::domain_computation::WorthQueryProductUnpublishedRecovery;

pub(super) struct PublishedRestorationSettlement {
    pub(super) observation: worth_runtime_world::facade::ProductBranchObservation,
    pub(super) commit: worth_relational::facade::transactions::CommitResult,
    pub(super) retry: RestorationRetry,
}

impl PublishedRestorationSettlement {
    pub(super) fn finish<Schema: ApplicationSchema>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &dyn crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>,
    ) -> super::WorthQueryRestoredGeneratedOutput {
        let Self {
            observation,
            commit,
            retry,
        } = self;
        let restored = retry
            .completion
            .complete(commit)
            .expect("World returns the exact prepared relational restoration result");
        runtime.record_restored_generated_output(
            execution,
            &observation,
            retry.correspondence,
            retry.producer,
        );
        super::WorthQueryRestoredGeneratedOutput {
            branch: retry.branch,
            commit: super::WorthQueryGeneratedOutputRestorationReceipt::new(
                restored.commit.clone(),
            ),
        }
    }
}

pub(super) trait SettlementParentOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    fn finish_restoration_handoff(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        old: WorthQueryProductUnpublishedRecovery,
        next: WorthQueryProductUnpublishedRecovery,
        retry: RestorationRetry,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<
        super::WorthQueryRestoredGeneratedOutput,
        WorthQueryGeneratedOutputRestorationRecoveryFailure,
    > {
        match self.application_runtime().release_product_recovery_in_request(execution, old, 0, retirements) {
            Ok(_) => Err(failure(
                WorthQueryGeneratedOutputRestorationRecoveryStage::AdoptionPublication,
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
                WorthQueryGeneratedOutputRestorationRecoveryStage::RecoveryHandoff,
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
                WorthQueryGeneratedOutputRestorationRecoveryStage::OwnerCleanup,
                RecoveryState::HandoffCleanup {
                    cleanup: failed.into_cleanup(),
                    next,
                    retry,
                },
            )),
        }
    }

    fn finish_restoration_published(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recovery: WorthQueryProductUnpublishedRecovery,
        settlement: PublishedRestorationSettlement,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<
        super::WorthQueryRestoredGeneratedOutput,
        WorthQueryGeneratedOutputRestorationRecoveryFailure,
    > {
        match self.application_runtime().release_product_recovery_in_request(execution, recovery, 0, retirements) {
            Ok(_) => Ok(settlement.finish(execution, self.product_operation_host())),
            Err(
                crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(
                    failed,
                ),
            ) => Err(failure(
                WorthQueryGeneratedOutputRestorationRecoveryStage::RecoveryHandoff,
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
                WorthQueryGeneratedOutputRestorationRecoveryStage::OwnerCleanup,
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
    > SettlementParentOps<Schema> for HostRuntime
{
}
