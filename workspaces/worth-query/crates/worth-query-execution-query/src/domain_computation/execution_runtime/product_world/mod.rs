use std::ops::{Deref, DerefMut};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution_lineage::facade::ports::ProductUnpublishedApplicationInternalPort as _;
use worth_query_execution_runtime::facade::execution_runtime::product_world as rt_pw;
use worth_query_execution_runtime::facade::ports::{
    ProductPublicationBindingInternalPort as _, ProductRuntimeInternalPort as _,
};

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
    WorthQueryProductRuntimeInstallationDenial, WorthQueryProductSharedRoot,
    WorthQueryProductWorldClock, WorthQueryProductWorldResources,
    WorthQueryRelationalSourceInstallationDenial, WorthQueryRelationalSourceOwner,
    WorthQueryReservedProductPublicationReceipt, WorthQueryRetiredProductOccurrence,
};

#[repr(transparent)]
#[derive(Clone)]
pub struct WorthQueryProductRuntime(pub(crate) rt_pw::WorthQueryProductRuntime);

impl Deref for WorthQueryProductRuntime {
    type Target = rt_pw::WorthQueryProductRuntime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WorthQueryProductRuntime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl WorthQueryProductRuntime {
    pub fn with_advancement<R>(
        &self,
        body: impl for<'scope> FnOnce(
            crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope>,
        ) -> R,
    ) -> Result<R, crate::domain_computation::primary_graph::WorthQueryAdvancementDenial> {
        self.0.with_advancement(body)
    }

    pub fn with_request_advancement<R>(
        &self,
        request: &WorthQueryRequestScope,
        body: impl for<'scope> FnOnce(
            crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope>,
        ) -> R,
    ) -> Result<R, crate::domain_computation::primary_graph::WorthQueryAdvancementDenial> {
        self.0.with_request_advancement(request, body)
    }

    pub fn install(
        relational: WorthQueryProductRelationalInstallation,
        bridge: &mut worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        resources: WorthQueryProductWorldResources,
        recovered_relational_authority: Option<
            worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority,
        >,
    ) -> Result<Self, WorthQueryProductRuntimeInstallationDenial> {
        rt_pw::WorthQueryProductRuntime::install(
            relational,
            bridge,
            resources,
            recovered_relational_authority,
        )
        .map(Self)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn attach_home_world_ports(
        relational: WorthQueryProductRelationalInstallation,
        bridge: &worth_runtime_bridge::facade::BridgeReadmittedAuthoredAssembly,
        resources: WorthQueryProductWorldResources,
        publication: worth_runtime_world::facade::RuntimeWorldPublicationPort<(), (), (), (), ()>,
        branch: worth_runtime_world::facade::RuntimeWorldBranchPort,
        observation: worth_runtime_world::facade::RuntimeWorldObservationPort,
        inspection: worth_runtime_world::facade::RuntimeWorldInspectionPort,
        recovery: worth_runtime_world::facade::RuntimeWorldRecoveryPort,
        root: worth_runtime_world::facade::ProductBranchObservation,
        maintenance: std::sync::Arc<dyn WorthQueryHomeMaintenance>,
        branch_program_source_owner: std::sync::Arc<worth_proof::InstalledBranchProgramSourceOwner>,
        branches: &worth_runtime_world::facade::ObservedProductBranches,
    ) -> Result<Self, WorthQueryProductRuntimeInstallationDenial> {
        rt_pw::WorthQueryProductRuntime::attach_home_world_ports(
            relational,
            bridge,
            resources,
            publication,
            branch,
            observation,
            inspection,
            recovery,
            root,
            HomeMaintenanceAdapter::wrap(maintenance),
            branch_program_source_owner,
            branches,
        )
        .map(Self)
    }
}

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
