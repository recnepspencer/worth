//! Releasing the World custody held by an unpublished branch adoption.

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

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) fn release_branch_adoption_custody(
        &self,
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
        match self.release_product_publication_recovery(product, minimum_age_ticks) {
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

    pub fn release_branch_adoption_recovery(
        &self,
        recovery: WorthQueryBranchAdoptionRecovery,
        minimum_age_ticks: u64,
    ) -> Result<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryBranchAdoptionRecoveryReleaseFailure,
    >{
        self.release_branch_adoption_custody(recovery, minimum_age_ticks)
    }
}
