use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution_lineage::facade::ports::ProductUnpublishedApplicationInternalPort as _;
use worth_query_execution_runtime::facade::execution_runtime::product_world as rt_pw;
use worth_query_execution_runtime::facade::ports::ProductPublicationBindingInternalPort as _;

mod maintenance;
pub use maintenance::{HomeMaintenanceAdapter, WorthQueryHomeMaintenance};

pub use rt_pw::{
    activation, branch_close, installed_source_reads_on_this_thread_for_test, owner_cleanup,
    prepare_conditional_publication, publication_binding, publication_denial,
    publish_conditional_definition, published_effect, read_consult, read_lock_probe,
    record_installed_source_read_for_test, relational_publication, request_control, resources,
    test_product_world_resources, test_product_world_resources_with_history_limit_for_test,
    OwnerCleanupRetirements, ReadLockProbeForTest, RetiringRelationalSource,
    WorthQueryCommitHistoryHold, WorthQueryDurableStateAdmissionDenial,
    WorthQueryPerformedRelationalProductChange,
    WorthQueryPerformedRelationalProductChangeDeliveryDenial,
    WorthQueryPerformedRelationalProductChangeDeliveryDenialKind,
    WorthQueryPerformedRelationalProductChangeDeliveryOutcome,
    WorthQueryPreparedProductPublication, WorthQueryProductBranchCloseDenial,
    WorthQueryProductBranchCloseReceipt, WorthQueryProductBranchCloseScope,
    WorthQueryProductBranchCreationDenial, WorthQueryProductBranchCreationNoEffect,
    WorthQueryProductBranchCreationOutcome, WorthQueryProductBranchCreationRelease,
    WorthQueryProductBranchOwnerCleanup, WorthQueryProductBranchOwnerCleanupDenial,
    WorthQueryProductBranchOwnerCleanupFailure, WorthQueryProductBranchOwnerCleanupReceipt,
    WorthQueryProductBranchOwnerCleanupWork, WorthQueryProductPublicationBinding,
    WorthQueryProductPublicationReceipt, WorthQueryProductPublicationRequestControl,
    WorthQueryProductRelationalInstallation, WorthQueryProductRetentionFollower,
    WorthQueryProductRuntime, WorthQueryProductRuntimeInstallationDenial,
    WorthQueryProductSharedRoot, WorthQueryProductWorldClock, WorthQueryProductWorldResources,
    WorthQueryRelationalSourceInstallationDenial, WorthQueryRelationalSourceOwner,
    WorthQueryReservedProductPublicationReceipt, WorthQueryRetiredProductOccurrence,
};

pub trait ProductPublicationBindingPrimaryExt {
    fn prepare_settled_relational_adoption(
        &self,
        unpublished: &crate::domain_computation::WorthQueryProductUnpublishedApplication,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryPreparedProductPublication,
        publication_denial::WorthQueryProductPublicationDenial<
            worth_runtime_world::facade::RuntimeWorldSettledRelationalAdoptionDenial,
        >,
    >;
}

impl ProductPublicationBindingPrimaryExt for WorthQueryProductPublicationBinding {
    fn prepare_settled_relational_adoption(
        &self,
        unpublished: &crate::domain_computation::WorthQueryProductUnpublishedApplication,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryPreparedProductPublication,
        publication_denial::WorthQueryProductPublicationDenial<
            worth_runtime_world::facade::RuntimeWorldSettledRelationalAdoptionDenial,
        >,
    > {
        self.prepare_settled_relational_adoption_with(request, |cancellation, deadline| {
            unpublished.prepare_settled_relational_adoption(cancellation, deadline)
        })
    }
}

pub(crate) mod internal_port;

#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
