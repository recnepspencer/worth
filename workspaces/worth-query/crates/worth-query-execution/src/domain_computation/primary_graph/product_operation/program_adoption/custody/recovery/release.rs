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
    /// World release completed; Query discharge still awaits acknowledgement.
    ObligationDischargePending {
        cause: crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial,
        cleanup: Result<crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupFailure>,
    },
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
            Self::OwnerCleanup(_) | Self::ObligationDischargePending { .. } => None,
        }
    }

    pub fn into_owner_cleanup(
        self,
    ) -> Option<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanup,
    >{
        match self {
            Self::Recovery { .. } | Self::ObligationDischargePending { cleanup: Ok(_), .. } => None,
            Self::ObligationDischargePending {
                cleanup: Err(failure),
                ..
            } => Some(failure.into_cleanup()),
            Self::OwnerCleanup(failure) => Some(failure.into_cleanup()),
        }
    }
}

impl std::fmt::Debug for WorthQueryBranchAdoptionRecoveryReleaseFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ObligationDischargePending { cause, .. } => formatter
                .debug_tuple("ObligationDischargePending")
                .field(cause)
                .finish(),
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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retirements: &mut OwnerCleanupRetirements,
        recovery: WorthQueryBranchAdoptionRecovery,
        minimum_age_ticks: u64,
    ) -> Result<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryBranchAdoptionRecoveryReleaseFailure,
    >{
        let attempt = match recovery.product.inspect() {
            Ok(effects) => {
                let actual = effects.attempt_identity();
                actual.owner_identity().durable().map(|owner| worth_foundational::facade::DurableCompositePublicationAttemptIdentity::from_recorded(owner,actual.ordinal()))
            }
            Err(cause) => {
                return Err(WorthQueryBranchAdoptionRecoveryReleaseFailure::Recovery {
                    denial: WorthQueryProductUnpublishedRecoveryReleaseDenial::World(cause),
                    recovery,
                })
            }
        };
        let WorthQueryBranchAdoptionRecovery {
            source,
            target,
            selected_entity_count,
            migration,
            custody,
            product,
            support_custody,
        } = recovery;
        let result = match self.release_product_recovery_in_request(
            execution,
            product,
            minimum_age_ticks,
            retirements,
        ) {
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
        };
        if matches!(
            &result,
            Ok(_) | Err(WorthQueryBranchAdoptionRecoveryReleaseFailure::OwnerCleanup(_))
        ) {
            if let Some(attempt) = attempt {
                if let Err(cause) = self
                    .installed_program_support()
                    .expect("adoption support")
                    .lifecycle()
                    .release_branch_adoption(execution, attempt)
                {
                    let cleanup = match result {
                        Ok(receipt) => Ok(receipt),
                        Err(WorthQueryBranchAdoptionRecoveryReleaseFailure::OwnerCleanup(
                            failure,
                        )) => Err(failure),
                        _ => unreachable!("only released World custody is discharged"),
                    };
                    return Err(WorthQueryBranchAdoptionRecoveryReleaseFailure::ObligationDischargePending { cause,cleanup });
                }
            }
        }
        result
    }

    pub fn release_branch_adoption_recovery(
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
