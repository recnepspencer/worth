//! Crate-boundary internal port for `WorthQueryPrimaryGraph` construction,
//! layout/application-publication access, runtime borrowing, integration handle
//! creation, and home retirement. Never exported on `worth-query-execution::facade`
//! or Host.

use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::runtime::RelationalRuntime;

use super::super::schema_layout::WorthQueryPrimaryGraphLayout;
use super::application_publication::ApplicationPublication;
use super::{WorthQueryPrimaryGraph, WorthQueryPrimaryGraphIntegrationHandle};
use crate::domain_computation::execution_runtime::product_world::WorthQueryRelationalSourceOwner;
use crate::domain_computation::execution_runtime::{
    WorthQueryInvalidationResources, WorthQueryRuntimeAuthorityIdentity,
};
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial, WorthQueryOperationHomeRetirementProgress,
};
use crate::ports::{ManagedDerivedViewPort, WorkflowRetentionPort};

pub trait PrimaryGraphInternalPort: Sized {
    fn from_installed_native(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: ApplicationSchemaBindingIdentity,
        layout: Arc<WorthQueryPrimaryGraphLayout>,
        runtime: RelationalRuntime,
        invalidation_resources: WorthQueryInvalidationResources,
        managed_views: Arc<dyn ManagedDerivedViewPort>,
    ) -> Self;

    fn from_installed_source(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: ApplicationSchemaBindingIdentity,
        layout: Arc<WorthQueryPrimaryGraphLayout>,
        source_owner: WorthQueryRelationalSourceOwner,
    ) -> Self;

    fn with_workflow_retention(self, workflow_retention: Arc<dyn WorkflowRetentionPort>) -> Self;
    fn bind_truth_partition(&mut self, role: worth_foundational::facade::TruthPartitionRole);
    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity;
    fn relational_runtime_instance_id(&self) -> u64;
    fn layout(&self) -> &WorthQueryPrimaryGraphLayout;
    fn retain_layout(&self) -> Arc<WorthQueryPrimaryGraphLayout>;
    fn layout_ref(&self) -> &Arc<WorthQueryPrimaryGraphLayout>;
    fn application_publication_ref(&self) -> Option<&Arc<ApplicationPublication>>;
    fn application_publication_mut(&mut self) -> &mut Option<Arc<ApplicationPublication>>;
    fn set_application_publication(&mut self, publication: Option<Arc<ApplicationPublication>>);
    fn managed_derived_views(&self) -> Arc<dyn ManagedDerivedViewPort>;
    fn integration_handle(&self) -> WorthQueryPrimaryGraphIntegrationHandle;
    fn product_read_session_port(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphReadOwnerPort;
    fn preflight_home_retirement(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryHomeRetirementDenial>;
    fn advance_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<WorthQueryOperationHomeRetirementProgress, WorthQueryHomeRetirementDenial>;
    fn with_runtime<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce(&RelationalRuntime) -> T,
    ) -> T;
}

impl PrimaryGraphInternalPort for WorthQueryPrimaryGraph {
    fn from_installed_native(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: ApplicationSchemaBindingIdentity,
        layout: Arc<WorthQueryPrimaryGraphLayout>,
        runtime: RelationalRuntime,
        invalidation_resources: WorthQueryInvalidationResources,
        managed_views: Arc<dyn ManagedDerivedViewPort>,
    ) -> Self {
        Self::from_installed_native(
            runtime_authority,
            binding_identity,
            layout,
            runtime,
            invalidation_resources,
            managed_views,
        )
    }

    fn from_installed_source(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: ApplicationSchemaBindingIdentity,
        layout: Arc<WorthQueryPrimaryGraphLayout>,
        source_owner: WorthQueryRelationalSourceOwner,
    ) -> Self {
        Self::from_installed_source(runtime_authority, binding_identity, layout, source_owner)
    }

    fn with_workflow_retention(self, workflow_retention: Arc<dyn WorkflowRetentionPort>) -> Self {
        Self::with_workflow_retention(self, workflow_retention)
    }

    fn bind_truth_partition(&mut self, role: worth_foundational::facade::TruthPartitionRole) {
        Self::bind_truth_partition(self, role)
    }

    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        Self::runtime_authority(self)
    }

    fn relational_runtime_instance_id(&self) -> u64 {
        Self::relational_runtime_instance_id(self)
    }

    fn layout(&self) -> &WorthQueryPrimaryGraphLayout {
        Self::layout(self)
    }

    fn retain_layout(&self) -> Arc<WorthQueryPrimaryGraphLayout> {
        Self::retain_layout(self)
    }

    fn layout_ref(&self) -> &Arc<WorthQueryPrimaryGraphLayout> {
        &self.layout
    }

    fn application_publication_ref(&self) -> Option<&Arc<ApplicationPublication>> {
        self.application_publication.as_ref()
    }

    fn application_publication_mut(&mut self) -> &mut Option<Arc<ApplicationPublication>> {
        &mut self.application_publication
    }

    fn set_application_publication(&mut self, publication: Option<Arc<ApplicationPublication>>) {
        self.application_publication = publication;
    }

    fn managed_derived_views(&self) -> Arc<dyn ManagedDerivedViewPort> {
        Self::managed_derived_views(self)
    }

    fn integration_handle(&self) -> WorthQueryPrimaryGraphIntegrationHandle {
        Self::integration_handle(self)
    }

    fn product_read_session_port(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphReadOwnerPort {
        Self::product_read_session_port(self)
    }

    fn preflight_home_retirement(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryHomeRetirementDenial> {
        Self::preflight_home_retirement(self, request)
    }

    fn advance_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<WorthQueryOperationHomeRetirementProgress, WorthQueryHomeRetirementDenial> {
        Self::advance_home_retirement(self, request, native)
    }

    fn with_runtime<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce(&RelationalRuntime) -> T,
    ) -> T {
        Self::with_runtime(self, execution, read)
    }
}
