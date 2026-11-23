//! The snapshot owner lends record and schema reads from the exact retained root.
use crate::branch::root::{RelationalBranchRoot, RelationalBranchRootSchemaAuthority};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::schema::data::{
    AspectContractPlanCatalog, LoweredAspectContractPlan, RelationalSchemaRegistry, SchemaVersionId,
};
use crate::storage::overlay::{
    CurrentPartitionAccess, PartitionAccess, PartitionIdMemory, PartitionIdsDenial, PartitionState,
};
use crate::storage::residency::{
    Full, Head, LiveRecordCounts, PartitionHeadProbe, PartitionProbe, RelationalRegionHeadView,
    RelationalResidencyDemandFailure, RelationalResidencyLease,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;

/// A restricted loan of the exact root while its original owner retains it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SnapshotProjectionRoot<'a>(&'a Arc<RelationalBranchRoot>);

impl<'a> SnapshotProjectionRoot<'a> {
    pub(super) const fn borrowed(root: &'a Arc<RelationalBranchRoot>) -> Self {
        Self(root)
    }

    pub(crate) fn root_custody(self) -> &'a Arc<RelationalBranchRoot> {
        self.0
    }

    pub(crate) fn partition_access(self) -> &'a dyn PartitionAccess {
        self.0.as_ref()
    }

    pub(crate) fn id(self) -> u64 {
        self.0.id()
    }

    pub(crate) fn storage_version(self) -> Option<VersionId> {
        self.0.axes().map(|axes| VersionId(axes.storage_version))
    }

    pub(crate) fn schema_authority(self) -> SnapshotProjectionSchema<'a> {
        SnapshotProjectionSchema(self.0.schema_authority())
    }

    pub(crate) fn query_schema_context(
        self,
    ) -> (
        crate::schema::data::SchemaVersionId,
        crate::schema::data::DescriptorSemanticsVersion,
        crate::query::data::QueryPlanEvidenceBasis,
    ) {
        self.0.query_schema_context()
    }

    pub(crate) fn live_counts(self) -> LiveRecordCounts {
        self.0.live_counts()
    }

    pub(crate) fn has_materialization_unavailable(self) -> bool {
        self.0.has_materialization_unavailable()
    }

    pub(crate) fn region_count(self) -> usize {
        self.0.region_count()
    }

    pub(crate) fn entity_regions_of_kind(
        self,
        kind: KindId,
    ) -> impl Iterator<Item = PartitionId> + 'a {
        self.0.entity_regions_of_kind(kind)
    }

    pub(crate) fn relation_regions_of_kind(
        self,
        kind: KindId,
    ) -> impl Iterator<Item = PartitionId> + 'a {
        self.0.relation_regions_of_kind(kind)
    }

    pub(crate) fn entity_ids_of_kind(
        self,
        kind: KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = EntityId> + 'a {
        self.0.entity_ids_of_kind(kind, partition)
    }

    pub(crate) fn relation_ids_of_kind(
        self,
        kind: KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = RelationId> + 'a {
        self.0.relation_ids_of_kind(kind, partition)
    }

    pub(crate) fn try_for_each_partition_id<E>(
        self,
        visit: impl FnMut(PartitionId) -> Result<(), E>,
    ) -> Result<(), E> {
        self.0.try_for_each_partition_id(visit)
    }

    pub(crate) fn partition_ids_iter(self) -> impl Iterator<Item = PartitionId> + 'a {
        self.0.partition_ids_iter()
    }

    pub(crate) fn accepted_full_region(self, id: PartitionId) -> Option<&'a PartitionState> {
        self.0.accepted_full_region(id)
    }

    pub(crate) fn accepted_head_region_view(
        self,
        id: PartitionId,
    ) -> Option<RelationalRegionHeadView<'a>> {
        self.0.accepted_head_region_view(id)
    }

    pub(crate) fn accepted_head_region_witness(
        self,
        id: PartitionId,
    ) -> Result<Option<RelationalResidencyLease<Head>>, PartitionId> {
        self.0.accepted_head_region_witness(id)
    }

    pub(crate) fn demand_region_head(
        self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<Option<RelationalResidencyLease<Head>>, RelationalResidencyDemandFailure> {
        self.0.demand_region_head(execution, id)
    }

    pub(crate) fn demand_region_full(
        self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<Option<RelationalResidencyLease<Full>>, RelationalResidencyDemandFailure> {
        self.0.demand_region_full(execution, id)
    }

    pub(crate) fn demand_all_full(
        self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &RuntimeSymbolTable,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.0.demand_all_full(execution, symbols)
    }

    pub(crate) fn demand_regions_head(
        self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &RuntimeSymbolTable,
        ids: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.0.demand_regions_head(execution, symbols, ids)
    }

    pub(crate) fn demand_regions_full(
        self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &RuntimeSymbolTable,
        ids: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.0.demand_regions_full(execution, symbols, ids)
    }

    pub(crate) fn symbol_admitted_projection(
        self,
        symbols: &'a RuntimeSymbolTable,
    ) -> impl CurrentPartitionAccess + PartitionAccess + 'a {
        self.0.symbol_admitted_projection(symbols)
    }
}

/// Immutable compiled meaning and coordinates carried by this selected basis.
pub(crate) struct SnapshotProjectionSchema<'a>(&'a RelationalBranchRootSchemaAuthority);

impl<'a> SnapshotProjectionSchema<'a> {
    pub(crate) fn registry(&self) -> &'a RelationalSchemaRegistry {
        self.0.registry()
    }
    pub(crate) fn aspect_plans(&self) -> &'a AspectContractPlanCatalog {
        self.0.aspect_plans()
    }
    pub(crate) fn schema_version(&self) -> SchemaVersionId {
        self.0.schema_version()
    }
    pub(crate) fn authority_digest(&self) -> [u8; 32] {
        self.0.authority_digest()
    }
    pub(crate) fn entity_aspect_plan(&self, kind: KindId) -> Option<&'a LoweredAspectContractPlan> {
        self.0.entity_aspect_plan(kind)
    }
    pub(crate) fn relation_aspect_plan(
        &self,
        kind: KindId,
    ) -> Option<&'a LoweredAspectContractPlan> {
        self.0.relation_aspect_plan(kind)
    }
    pub(crate) fn entity_aspect_plan_count(&self) -> usize {
        self.0.entity_aspect_plan_count()
    }
    pub(crate) fn relation_aspect_plan_count(&self) -> usize {
        self.0.relation_aspect_plan_count()
    }
}

impl PartitionAccess for SnapshotProjectionRoot<'_> {
    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        self.0.get_partition(id)
    }
    fn probe_full_partition(&self, id: PartitionId) -> PartitionProbe<'_> {
        self.0.probe_full_partition(id)
    }
    fn demand_full_partition(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.0.demand_full_partition(execution, id)
    }
}

impl CurrentPartitionAccess for SnapshotProjectionRoot<'_> {
    fn get_current_partition(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        self.0.get_current_partition(id)
    }
    fn probe_head_partition(&self, id: PartitionId) -> PartitionHeadProbe<'_> {
        self.0.probe_head_partition(id)
    }
    fn demand_head_partition(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.0.demand_head_partition(execution, id)
    }
    fn partition_ids(&self) -> Vec<PartitionId> {
        self.0.partition_ids()
    }
    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, PartitionIdsDenial> {
        self.0.partition_ids_with_memory(memory)
    }
    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        CurrentPartitionAccess::partition_ids_iter(self.0.as_ref())
    }
}
