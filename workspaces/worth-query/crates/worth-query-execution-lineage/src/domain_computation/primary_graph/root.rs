pub mod application_publication;
pub mod home_retirement;
pub mod runtime_access;
pub mod snapshot_release;

use std::sync::{Arc, Mutex};

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::runtime::RelationalRuntime;

use super::schema_layout::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::execution_runtime::product_world::WorthQueryRelationalSourceOwner;

pub mod index_installation;
pub mod selected_runtime;
pub mod test_inspection;
use index_installation::primary_index_inventory;

/// Execution-owned primary logical graph for one installed application schema.
///
/// This surface exposes graph identity rather than raw Relational access.
/// Query integration code receives a separate hidden handle so product
/// consumers cannot bypass installed Query authority.
pub struct WorthQueryPrimaryGraph {
    pub application_publication:
        Option<Arc<application_publication::ApplicationPublication>>,
    runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    relational_runtime_instance_id: u64,
    binding_identity: ApplicationSchemaBindingIdentity,
    pub layout: Arc<WorthQueryPrimaryGraphLayout>,
    source_owner: home_retirement::PrimaryGraphSourceCustody,
    aggregate_projections: Arc<Mutex<super::aggregate_projection::WorthQueryAggregateProjections>>,
    output_lineage: Arc<Mutex<super::output_lineage::WorthQueryApplicationOutputLineage>>,
    managed_derived_views: Arc<dyn crate::ports::ManagedDerivedViewPort>,
    workflow_retention: Arc<dyn crate::ports::WorkflowRetentionPort>,
    truth_partition_role: Option<worth_foundational::facade::TruthPartitionRole>,
}

impl WorthQueryPrimaryGraph {
    pub fn from_installed_native(
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: ApplicationSchemaBindingIdentity,
        layout: Arc<WorthQueryPrimaryGraphLayout>,
        runtime: RelationalRuntime,
        invalidation_resources: crate::domain_computation::execution_runtime::WorthQueryInvalidationResources,
        managed_views: Arc<dyn crate::ports::ManagedDerivedViewPort>,
    ) -> Self {
        let runtime_id = runtime.main_branch_identity().runtime_instance_id();
        let invalidation_owner = Arc::new(super::SourceInvalidationOwner::with_managed_views(
            invalidation_resources,
            runtime_id,
            managed_views,
        ));
        let source_owner = WorthQueryRelationalSourceOwner::with_invalidation_owner(
            runtime,
            "primary",
            invalidation_owner,
        )
        .expect("the installed primary graph role is canonical");
        Self::from_installed_source(runtime_authority, binding_identity, layout, source_owner)
    }

    pub fn from_installed_source(
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: ApplicationSchemaBindingIdentity,
        layout: Arc<WorthQueryPrimaryGraphLayout>,
        source_owner: WorthQueryRelationalSourceOwner,
    ) -> Self {
        let relational_runtime_instance_id = source_owner.runtime_instance_id;
        let managed_derived_views = Arc::clone(
            &source_owner
                .invalidation_owner
                .as_any()
                .downcast_ref::<super::SourceInvalidationOwner>()
                .expect("primary graph source carries SourceInvalidationOwner")
                .managed_views,
        );
        Self {
            application_publication: None,
            runtime_authority,
            relational_runtime_instance_id,
            binding_identity,
            layout,
            source_owner: home_retirement::PrimaryGraphSourceCustody::new(source_owner),
            aggregate_projections: Arc::new(Mutex::new(
                super::aggregate_projection::WorthQueryAggregateProjections::default(),
            )),
            output_lineage: Arc::new(Mutex::new(Default::default())),
            managed_derived_views,
            workflow_retention: Arc::new(crate::ports::EmptyWorkflowRetentionPort),
            truth_partition_role: None,
        }
    }

    pub fn with_workflow_retention(
        mut self,
        workflow_retention: Arc<dyn crate::ports::WorkflowRetentionPort>,
    ) -> Self {
        self.workflow_retention = workflow_retention;
        self
    }

    pub fn bind_truth_partition(
        &mut self,
        role: worth_foundational::facade::TruthPartitionRole,
    ) {
        self.source_owner
            .bind_truth_partition(
                "primary",
                worth_relational::facade::identity::PartitionId::main(),
                role.clone(),
            )
            .expect("the installed primary graph partition role is canonical");
        self.truth_partition_role = Some(role);
    }

    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub const fn runtime_authority(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.runtime_authority
    }

    pub const fn relational_runtime_instance_id(&self) -> u64 {
        self.relational_runtime_instance_id
    }

    pub fn layout(&self) -> &WorthQueryPrimaryGraphLayout {
        &self.layout
    }

    pub fn retain_layout(&self) -> Arc<WorthQueryPrimaryGraphLayout> {
        Arc::clone(&self.layout)
    }

    pub fn managed_derived_views(
        &self,
    ) -> Arc<dyn crate::ports::ManagedDerivedViewPort> {
        Arc::clone(&self.managed_derived_views)
    }

    pub fn integration_handle(&self) -> WorthQueryPrimaryGraphIntegrationHandle {
        let primary_index_ids = primary_index_inventory(&self.layout);
        WorthQueryPrimaryGraphIntegrationHandle {
            application_publication: self.application_publication.clone(),
            source_owner: self.source_owner.clone(),
            layout: Arc::clone(&self.layout),
            primary_index_ids,
            aggregate_projections: Arc::clone(&self.aggregate_projections),
            output_lineage: Arc::clone(&self.output_lineage),
            managed_derived_views: Arc::clone(&self.managed_derived_views),
            workflow_retention: Arc::clone(&self.workflow_retention),
            truth_partition_role: self.truth_partition_role.clone(),
        }
    }

    pub fn product_read_session_port(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphReadOwnerPort {
        crate::domain_computation::provider_session::WorthQueryGraphReadOwnerPort::new(
            self.binding_identity.clone(),
            self.source_owner.retain_live(),
            Arc::clone(&self.layout),
        )
    }
}

impl std::fmt::Debug for WorthQueryPrimaryGraph {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryPrimaryGraph")
            .field("binding_identity", &self.binding_identity)
            .finish_non_exhaustive()
    }
}

/// Hidden composition handle for sharing the one primary graph with Query's
/// existing runtime backend.
#[doc(hidden)]
#[derive(Clone)]
pub struct WorthQueryPrimaryGraphIntegrationHandle {
    pub application_publication: Option<Arc<application_publication::ApplicationPublication>>,
    pub source_owner: home_retirement::PrimaryGraphSourceCustody,
    pub layout: Arc<WorthQueryPrimaryGraphLayout>,
    pub primary_index_ids: Arc<[worth_relational::facade::indexes::DerivedIndexId]>,
    pub aggregate_projections:
        Arc<Mutex<super::aggregate_projection::WorthQueryAggregateProjections>>,
    pub output_lineage: Arc<Mutex<super::output_lineage::WorthQueryApplicationOutputLineage>>,
    pub managed_derived_views: Arc<dyn crate::ports::ManagedDerivedViewPort>,
    pub workflow_retention: Arc<dyn crate::ports::WorkflowRetentionPort>,
    pub truth_partition_role: Option<worth_foundational::facade::TruthPartitionRole>,
}

impl WorthQueryPrimaryGraphIntegrationHandle {
    pub fn managed_derived_views(
        &self,
    ) -> Arc<dyn crate::ports::ManagedDerivedViewPort> {
        Arc::clone(&self.managed_derived_views)
    }

    pub fn workflow_retention(&self) -> &Arc<dyn crate::ports::WorkflowRetentionPort> {
        &self.workflow_retention
    }

    #[doc(hidden)]
    pub fn workflow_compilation_reuse_counters(
        &self,
    ) -> super::WorthQueryWorkflowCompilationReuseCounters {
        self.workflow_retention.compilation_reuse_counters()
    }

    #[doc(hidden)]
    pub fn workflow_instance_progress_counters(
        &self,
    ) -> super::WorthQueryWorkflowInstanceProgressCounters {
        self.workflow_retention.instance_progress_counters()
    }

    pub fn release_workflow_instance_progress(&self) {
        self.workflow_retention.release_instance_progress();
    }

    pub fn release_workflow_instance_progress_for_branch(&self, branch_occurrence: u64) {
        self.workflow_retention
            .release_instance_progress_for_branch(branch_occurrence);
    }

    /// Retains the shared relational source for a host-owned runtime Bridge
    /// that must observe this exact primary graph.
    ///
    /// The source carries graph access, not invalidation admission authority.
    #[doc(hidden)]
    pub fn relational_bridge_source(
        &self,
    ) -> worth_runtime_bridge::facade::RuntimeBridgeRelationalSource {
        self.source_owner.bridge_source()
    }

    #[doc(hidden)]
    pub fn current_truth_snapshot(
        &self,
        branch: &worth_runtime_bridge::facade::TruthBranchIdentity,
    ) -> Option<worth_runtime_bridge::facade::TruthSnapshotIdentity> {
        self.source_owner.current_truth_snapshot(branch)
    }

    pub fn bind_current_truth_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch: &worth_relational::facade::history::BranchId,
    ) -> Result<
        worth_runtime_bridge::facade::TruthSnapshotIdentity,
        worth_relational::facade::branch::RelationalBranchBasisDenial,
    > {
        self.source_owner
            .bind_current_truth_head_with_request(execution, branch)
    }

    pub fn bind_truth_head_basis_in_runtime(
        &self,
        runtime: &RelationalRuntime,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    ) -> Result<
        worth_runtime_bridge::facade::TruthSnapshotIdentity,
        worth_relational::facade::branch::RelationalBranchBasisDenial,
    > {
        self.source_owner
            .bind_truth_head_basis_in_runtime(runtime, basis)
    }

    /// Admits product installation without exposing a mutation route that could
    /// bypass this primary graph's required index maintenance.
    #[doc(hidden)]
    pub fn prepare_product_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch: &worth_relational::facade::branch::RelationalBranchIdentity,
    ) -> Result<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductRelationalInstallation,
        worth_relational::facade::branch::RelationalBranchBasisDenial,
    >{
        self.source_owner.prepare_product_source(execution, branch)
    }

    pub fn prepare_product_source_at_basis(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    ) -> Result<crate::domain_computation::execution_runtime::product_world::WorthQueryProductRelationalInstallation,
    worth_relational::facade::branch::RelationalBranchBasisDenial>{
        self.source_owner
            .prepare_product_source_at_basis(execution, basis)
    }

    pub const fn truth_partition_role(
        &self,
    ) -> Option<&worth_foundational::facade::TruthPartitionRole> {
        self.truth_partition_role.as_ref()
    }
}

mod court_doors_for_test;
