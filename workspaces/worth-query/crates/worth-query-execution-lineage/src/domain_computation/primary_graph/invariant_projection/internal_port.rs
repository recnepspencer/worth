//! Crate-boundary internal ports for `WorthQueryApplicationInvariantProjectionAuthority`
//! lifecycle/snapshot access and `WorthQueryApplicationInvariantProjectionSnapshot`.
//! Never exported on `worth-query-execution::facade` or Host.

mod projection;
pub use projection::ApplicationInvariantProjectionExecutionInternalPort;

use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationSchema, ApplicationSchemaBindingIdentity, InstalledOperationCatalog,
    WorthQueryCompiledApplicationOperationContracts,
    WorthQueryInstalledApplicationValueBindingCatalog,
};

use super::operation_reader::{
    DecisionReads, WorthQueryApplicationOperationInvariantProjectionReader,
};
use super::{
    ConsumedOutputEvidence, WorthQueryApplicationInvariantProjectionAuthority,
    WorthQueryApplicationInvariantProjectionReader,
    WorthQueryApplicationInvariantProjectionSnapshot, WorthQueryRealizedProjectionScope,
};
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::RetainedSourceFacts;
use crate::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease;
use crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramActivationCell;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial, WorthQueryInstalledEntityResolutionContext,
    WorthQueryOperationHomeRetirementProgress, WorthQueryPrimaryGraphIntegrationHandle,
};

pub trait ApplicationInvariantProjectionAuthorityInternalPort<Schema: ApplicationSchema>:
    Sized
{
    #[allow(clippy::too_many_arguments)]
    fn from_installation_parts(
        installed_operations: Arc<InstalledOperationCatalog>,
        value_bindings: WorthQueryInstalledApplicationValueBindingCatalog,
        graph: WorthQueryPrimaryGraphIntegrationHandle,
        layout: Arc<WorthQueryPrimaryGraphLayout>,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: ApplicationSchemaBindingIdentity,
        entity_resolution: WorthQueryInstalledEntityResolutionContext,
        program_activation: Option<WorthQueryProgramActivationCell>,
    ) -> Self;

    fn retain_for_installation(&self) -> Self;

    fn belongs_to_installation(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool;

    fn active_snapshot_count(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> usize;

    fn preflight_home_retirement(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryHomeRetirementDenial>;

    fn advance_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<WorthQueryOperationHomeRetirementProgress, WorthQueryHomeRetirementDenial>;

    fn inspect_reader<O, T>(
        &self,
        reader: &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        contracts: Option<Arc<WorthQueryCompiledApplicationOperationContracts>>,
        product: Option<&crate::basis::WorthQueryProductBranchLease>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, O>,
        ) -> T,
    ) -> (T, DecisionReads);
}

impl<Schema: ApplicationSchema> ApplicationInvariantProjectionAuthorityInternalPort<Schema>
    for WorthQueryApplicationInvariantProjectionAuthority<Schema>
{
    fn from_installation_parts(
        installed_operations: Arc<InstalledOperationCatalog>,
        value_bindings: WorthQueryInstalledApplicationValueBindingCatalog,
        graph: WorthQueryPrimaryGraphIntegrationHandle,
        layout: Arc<WorthQueryPrimaryGraphLayout>,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: ApplicationSchemaBindingIdentity,
        entity_resolution: WorthQueryInstalledEntityResolutionContext,
        program_activation: Option<WorthQueryProgramActivationCell>,
    ) -> Self {
        Self::from_installation_parts(
            installed_operations,
            value_bindings,
            graph,
            layout,
            runtime_authority,
            binding_identity,
            entity_resolution,
            program_activation,
        )
    }

    fn retain_for_installation(&self) -> Self {
        Self::retain_for_installation(self)
    }

    fn belongs_to_installation(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        Self::belongs_to_installation(self, runtime_authority, binding_identity)
    }

    fn active_snapshot_count(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> usize {
        Self::active_snapshot_count(self, request_execution)
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
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<WorthQueryOperationHomeRetirementProgress, WorthQueryHomeRetirementDenial> {
        Self::advance_home_retirement(self, request, native)
    }

    fn inspect_reader<O, T>(
        &self,
        reader: &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        contracts: Option<Arc<WorthQueryCompiledApplicationOperationContracts>>,
        product: Option<&crate::basis::WorthQueryProductBranchLease>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, O>,
        ) -> T,
    ) -> (T, DecisionReads) {
        Self::inspect_reader(self, reader, contracts, product, projection)
    }
}

pub trait ApplicationInvariantProjectionSnapshotInternalPort<Schema: ApplicationSchema> {
    fn belongs_to(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool;

    fn into_lease_and_realized_scope(
        self,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> (
        WorthQueryApplicationSnapshotLease,
        WorthQueryRealizedProjectionScope,
        Option<RetainedSourceFacts>,
        Vec<ConsumedOutputEvidence>,
    );
}

impl<Schema: ApplicationSchema> ApplicationInvariantProjectionSnapshotInternalPort<Schema>
    for WorthQueryApplicationInvariantProjectionSnapshot<Schema>
{
    fn belongs_to(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        Self::belongs_to(self, runtime_authority, binding_identity)
    }

    fn into_lease_and_realized_scope(
        self,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> (
        WorthQueryApplicationSnapshotLease,
        WorthQueryRealizedProjectionScope,
        Option<RetainedSourceFacts>,
        Vec<ConsumedOutputEvidence>,
    ) {
        Self::into_lease_and_realized_scope(self, product)
    }
}
