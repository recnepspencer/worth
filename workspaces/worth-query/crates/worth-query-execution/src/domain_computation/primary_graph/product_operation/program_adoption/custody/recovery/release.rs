//! Releasing the World custody held by an unpublished branch adoption.
use crate::domain_computation::execution_runtime::product_world::OwnerCleanupRetirements;

use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryBranchAdoptionRecovery;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::{
    WorthQueryProductUnpublishedRecoveryReleaseDenial,
    WorthQueryProductUnpublishedRecoveryReleaseFailure,
};

/// Why `release_branch_adoption_recovery` did not return a cleanup receipt.
pub enum WorthQueryBranchAdoptionRecoveryReleaseFailure {
    /// The recovery record was not released; the recovery is handed back
    /// unchanged to retry.
    Recovery {
        denial: WorthQueryProductUnpublishedRecoveryReleaseDenial,
        recovery: WorthQueryBranchAdoptionRecovery,
    },
    /// The recovery record was released, but owner cleanup did not finish. The
    /// cleanup is carried for retry.
    OwnerCleanup(
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupFailure,
    ),
}

impl WorthQueryBranchAdoptionRecoveryReleaseFailure {
    pub fn into_recovery(self) -> Option<WorthQueryBranchAdoptionRecovery> {
        match self {
            Self::Recovery { recovery, .. } => Some(recovery),
            Self::OwnerCleanup(_) => None,
        }
    }

    pub fn into_owner_cleanup(
        self,
    ) -> Option<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanup,
    >{
        match self {
            Self::Recovery { .. } => None,
            Self::OwnerCleanup(failure) => Some(failure.into_cleanup()),
        }
    }
}

impl std::fmt::Debug for WorthQueryBranchAdoptionRecoveryReleaseFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Recovery { denial, .. } => formatter
                .debug_struct("Recovery")
                .field("denial", denial)
                .finish_non_exhaustive(),
            Self::OwnerCleanup(failure) => formatter
                .debug_tuple("OwnerCleanup")
                .field(failure)
                .finish(),
        }
    }
}

pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) trait ReleaseScopedOps<
    Schema: ApplicationSchema,
>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    fn release_branch_adoption_custody(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retirements: &mut OwnerCleanupRetirements,
        recovery: WorthQueryBranchAdoptionRecovery,
        minimum_age_ticks: u64,
    ) -> Result<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryBranchAdoptionRecoveryReleaseFailure,
    >{
        let WorthQueryBranchAdoptionRecovery {
            source,
            target,
            selected_entity_count,
            migration,
            custody,
            product,
            support_custody,
        } = recovery;
        match self
            .application_runtime()
            .release_product_recovery_in_request(execution, product, minimum_age_ticks, retirements)
        {
            Ok(receipt) => Ok(receipt),
            Err(WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(failure)) => {
                let denial = failure.denial();
                Err(WorthQueryBranchAdoptionRecoveryReleaseFailure::Recovery {
                    denial,
                    recovery: WorthQueryBranchAdoptionRecovery {
                        source,
                        target,
                        selected_entity_count,
                        migration,
                        custody,
                        product: failure.into_recovery(),
                        support_custody,
                    },
                })
            }
            Err(WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup(failure)) => {
                Err(WorthQueryBranchAdoptionRecoveryReleaseFailure::OwnerCleanup(failure))
            }
        }
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                Schema,
            > + ?Sized,
    > ReleaseScopedOps<Schema> for HostRuntime
{
}

pub trait ReleaseOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    fn release_branch_adoption_recovery(
        &self,
        recovery: WorthQueryBranchAdoptionRecovery,
        minimum_age_ticks: u64,
    ) -> Result<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryBranchAdoptionRecoveryReleaseFailure,
    >{
        let mut retirements = OwnerCleanupRetirements::default();
        let host = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        );
        self.release_branch_adoption_custody(
            &worth_execution::ExecutionRequest::serial(&host),
            &mut retirements,
            recovery,
            minimum_age_ticks,
        )
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                Schema,
            > + ?Sized,
    > ReleaseOps<Schema> for HostRuntime
{
}
