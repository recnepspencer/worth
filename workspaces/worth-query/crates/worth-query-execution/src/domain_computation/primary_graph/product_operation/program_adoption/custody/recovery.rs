use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{NoEffectCompositePublication, RuntimeWorldRecoveryDenial};

use super::super::publication::{
    WorthQueryPerformedBranchAdoption, WorthQueryUnpublishedBranchAdoption,
};
use crate::domain_computation::primary_graph::product_operation::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::{
    WorthQueryProductUnpublishedRecovery, WorthQueryProductUnpublishedRecoveryReleaseDenial,
    WorthQueryProductUnpublishedRecoveryReleaseFailure,
};

const IMMEDIATE_RECOVERY_RELEASE_AGE_TICKS: u64 = 0;

#[must_use = "unpublished adoption recovery owns exact World custody"]
pub struct WorthQueryBranchAdoptionRecovery {
    source: ApplicationProgramRevision,
    target: ApplicationProgramRevision,
    selected_entity_count: usize,
    migration: Option<super::super::preparation::WorthQueryProgramMigrationDescription>,
    custody: super::super::preparation::WorthQueryProgramCustodyDispositionInventory,
    product: WorthQueryProductUnpublishedRecovery,
    support_custody:
        crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportCustody,
}

impl WorthQueryBranchAdoptionRecovery {
    pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) fn new(
        source: ApplicationProgramRevision,
        target: ApplicationProgramRevision,
        selected_entity_count: usize,
        migration: Option<super::super::preparation::WorthQueryProgramMigrationDescription>,
        custody: super::super::preparation::WorthQueryProgramCustodyDispositionInventory,
        product: WorthQueryProductUnpublishedRecovery,
        support_custody: crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportCustody,
    ) -> Self {
        Self {
            source,
            target,
            selected_entity_count,
            migration,
            custody,
            product,
            support_custody,
        }
    }

    pub fn source(&self) -> &ApplicationProgramRevision {
        &self.source
    }

    pub fn target(&self) -> &ApplicationProgramRevision {
        &self.target
    }

    pub const fn migration(
        &self,
    ) -> Option<&super::super::preparation::WorthQueryProgramMigrationDescription> {
        self.migration.as_ref()
    }

    pub const fn custody(
        &self,
    ) -> &super::super::preparation::WorthQueryProgramCustodyDispositionInventory {
        &self.custody
    }
}

pub enum WorthQueryBranchAdoptionRecoveryReleaseFailure {
    Recovery {
        denial: WorthQueryProductUnpublishedRecoveryReleaseDenial,
        recovery: WorthQueryBranchAdoptionRecovery,
    },
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

#[derive(Debug)]
pub enum WorthQueryBranchAdoptionRecoveryDenial {
    ProductAffinityMismatch,
    Inspection(RuntimeWorldRecoveryDenial),
    OwnerSettlement(RuntimeWorldRecoveryDenial),
    AdoptionPreparation(worth_runtime_world::facade::RuntimeWorldSettledRelationalAdoptionDenial),
}

pub struct WorthQueryBranchAdoptionRecoveryFailure {
    denial: WorthQueryBranchAdoptionRecoveryDenial,
    recovery: WorthQueryBranchAdoptionRecovery,
}

impl WorthQueryBranchAdoptionRecoveryFailure {
    pub const fn denial(&self) -> &WorthQueryBranchAdoptionRecoveryDenial {
        &self.denial
    }

    pub fn into_recovery(self) -> WorthQueryBranchAdoptionRecovery {
        self.recovery
    }
}

pub enum WorthQueryBranchAdoptionRecoveryOutcome {
    Performed {
        adoption: WorthQueryPerformedBranchAdoption,
        cleanup: Result<
            crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
            WorthQueryProductUnpublishedRecoveryReleaseFailure,
        >,
    },
    NoEffect {
        no_effect: NoEffectCompositePublication,
        recovery: WorthQueryBranchAdoptionRecovery,
    },
    ProductUnpublished {
        next: WorthQueryUnpublishedBranchAdoption,
        prior_cleanup: Result<
            crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
            WorthQueryProductUnpublishedRecoveryReleaseFailure,
        >,
    },
}

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub fn recover_branch_adoption(
        &self,
        recovery: WorthQueryBranchAdoptionRecovery,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryBranchAdoptionRecoveryOutcome, WorthQueryBranchAdoptionRecoveryFailure>
    {
        let inspected = match recovery.product.inspect() {
            Ok(inspected) => inspected,
            Err(denial) => {
                return Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::Inspection(denial),
                    recovery,
                ))
            }
        };
        if inspected.expected_product() != self.product().publication_binding().observation() {
            drop(inspected);
            return Err(failure(
                WorthQueryBranchAdoptionRecoveryDenial::ProductAffinityMismatch,
                recovery,
            ));
        }
        let needs_settlement = inspected.relational_requires_settlement();
        drop(inspected);
        if needs_settlement {
            if let Err(denial) = recovery.product.continue_owner_settlement() {
                return Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::OwnerSettlement(denial),
                    recovery,
                ));
            }
        }
        let unpublished = match recovery.product.inspect() {
            Ok(unpublished) => unpublished,
            Err(denial) => {
                return Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::Inspection(denial),
                    recovery,
                ))
            }
        };
        let prepared = match self
            .product()
            .publication_binding()
            .prepare_settled_relational_adoption(&unpublished, request)
        {
            Ok(prepared) => prepared,
            Err(denial) => {
                drop(unpublished);
                return Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::AdoptionPreparation(denial),
                    recovery,
                ));
            }
        };
        drop(unpublished);
        let binding = self.product().publication_binding().clone();
        let WorthQueryBranchAdoptionRecovery {
            source,
            target,
            selected_entity_count,
            migration,
            custody,
            product,
            support_custody,
        } = recovery;
        match prepared.execute() {
            worth_runtime_world::facade::RuntimeWorldPublicationOutcome::Performed(performed) => {
                let adoption = WorthQueryPerformedBranchAdoption::new(
                    performed.consume(),
                    source,
                    target,
                    selected_entity_count,
                    migration,
                    custody,
                );
                let cleanup = self.application().release_product_publication_recovery(
                    product,
                    IMMEDIATE_RECOVERY_RELEASE_AGE_TICKS,
                );
                Ok(WorthQueryBranchAdoptionRecoveryOutcome::Performed { adoption, cleanup })
            }
            worth_runtime_world::facade::RuntimeWorldPublicationOutcome::NoEffect(no_effect) => {
                Ok(WorthQueryBranchAdoptionRecoveryOutcome::NoEffect {
                    no_effect,
                    recovery: WorthQueryBranchAdoptionRecovery {
                        source,
                        target,
                        selected_entity_count,
                        migration,
                        custody,
                        product,
                        support_custody,
                    },
                })
            }
            worth_runtime_world::facade::RuntimeWorldPublicationOutcome::ProductUnpublished(
                effects,
            ) => {
                let next = WorthQueryUnpublishedBranchAdoption::new(
                    source,
                    target,
                    selected_entity_count,
                    migration,
                    custody,
                    effects,
                    binding.recovery(),
                    self.application()
                        .primary_provider
                        .unpublished_idempotency_disposition(),
                    support_custody,
                );
                let prior_cleanup = self.application().release_product_publication_recovery(
                    product,
                    IMMEDIATE_RECOVERY_RELEASE_AGE_TICKS,
                );
                Ok(
                    WorthQueryBranchAdoptionRecoveryOutcome::ProductUnpublished {
                        next,
                        prior_cleanup,
                    },
                )
            }
        }
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn integration_recover_branch_adoption(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
        recovery: WorthQueryBranchAdoptionRecovery,
        request: &WorthQueryRequestScope,
    ) -> Result<
        Result<WorthQueryBranchAdoptionRecoveryOutcome, WorthQueryBranchAdoptionRecoveryFailure>,
        (
            crate::basis::WorthQueryProductBranchAdmissionDenial,
            WorthQueryBranchAdoptionRecovery,
        ),
    > {
        let product = match self
            .product_runtime
            .admit_product_occurrence(branch.occurrence())
        {
            Ok(product) => product,
            Err(denial) => return Err((denial, recovery)),
        };
        let selected = match self.on_product_for_adoption_recovery(product) {
            Ok(selected) => selected,
            Err(denial) => return Err((denial, recovery)),
        };
        Ok(selected.recover_branch_adoption(recovery, request))
    }

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

fn failure(
    denial: WorthQueryBranchAdoptionRecoveryDenial,
    recovery: WorthQueryBranchAdoptionRecovery,
) -> WorthQueryBranchAdoptionRecoveryFailure {
    WorthQueryBranchAdoptionRecoveryFailure { denial, recovery }
}
