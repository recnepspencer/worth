//! The runtime's consumer methods over `domain_computation::product_publication`; each only calls its subsystem.
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt;
use crate::domain_computation::product_publication::WorthQueryProductUnpublishedRecovery;
use crate::domain_computation::product_publication::WorthQueryProductUnpublishedRecovery as Recovery;
use crate::domain_computation::product_publication::WorthQueryProductUnpublishedRecoveryReleaseFailure;
use crate::domain_computation::product_publication::WorthQueryProductUnpublishedRecoveryReleaseFailure as Failure;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use std::num::NonZeroUsize;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;
use worth_runtime_world::facade::RuntimeWorldRecoveryCursor;
use worth_runtime_world::facade::RuntimeWorldRecoveryDenial;
use worth_runtime_world::facade::RuntimeWorldRecoveryPage;
use worth_runtime_world::facade::RuntimeWorldServiceDenial;
impl<S: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn close_owned_product_publication_recovery(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        recovery: Recovery,
    ) -> Result<WorthQueryProductBranchOwnerCleanupReceipt, Failure> {
        <Self as crate::domain_computation::product_publication::OwnerCloseOps<S>>::close_owned_product_publication_recovery(self, phase, _access, recovery)
    }
}

impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn product_publication_recovery_page(
        &self,
        after: Option<&RuntimeWorldRecoveryCursor>,
        maximum: NonZeroUsize,
    ) -> Result<RuntimeWorldRecoveryPage, RuntimeWorldServiceDenial<RuntimeWorldRecoveryDenial>>
    {
        <Self as crate::domain_computation::product_publication::DiscoveryOps<Schema>>::product_publication_recovery_page(self, after, maximum)
    }

    pub fn readmit_product_publication_recovery(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<WorthQueryProductUnpublishedRecovery, RuntimeWorldRecoveryDenial> {
        <Self as crate::domain_computation::product_publication::DiscoveryOps<Schema>>::readmit_product_publication_recovery(self, handle)
    }

    pub fn release_product_publication_recovery(
        &self,
        recovery: WorthQueryProductUnpublishedRecovery,
        minimum_age_ticks: u64,
    ) -> Result<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryProductUnpublishedRecoveryReleaseFailure,
    >{
        <Self as crate::domain_computation::product_publication::DiscoveryOps<Schema>>::release_product_publication_recovery(self, recovery, minimum_age_ticks)
    }

    pub fn release_product_publication_recovery_in_advancement(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        recovery: WorthQueryProductUnpublishedRecovery,
        minimum_age_ticks: u64,
    ) -> Result<crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt, WorthQueryProductUnpublishedRecoveryReleaseFailure>{
        <Self as crate::domain_computation::product_publication::DiscoveryOps<Schema>>::release_product_publication_recovery_in_advancement(self, phase, recovery, minimum_age_ticks)
    }
}

impl<S: ApplicationSchema>
    crate::domain_computation::primary_graph::application_output_demand::denial::recovery_release::WorthQueryProductRecoveryClose
    for super::super::WorthQueryPrimaryGraphApplicationRuntime<S>
{
    fn close_recovery_in_advancement(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        recovery: crate::domain_computation::product_publication::WorthQueryProductUnpublishedRecovery,
    ) -> Result<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
        crate::domain_computation::product_publication::WorthQueryProductUnpublishedRecoveryReleaseFailure,
    > {
        crate::domain_computation::primary_graph::application_output_demand::denial::recovery_release::WorthQueryProductRecoveryClose::close_recovery_in_advancement(
            crate::domain_computation::product_publication::host::ProductPublicationHost::product_publication_host(self),
            phase,
            recovery,
        )
    }
}
