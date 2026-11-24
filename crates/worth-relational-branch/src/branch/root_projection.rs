//! Selected records and schema are lent within the original retained root.
use crate::branch::root_regions::backing::RelationalPersistentRegionSet;
use crate::branch::symbol_read::SymbolAdmittedRootRead;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::query::data::QueryPlanEvidenceBasis;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::schema::data::{
    AspectContractPlanCatalog, DescriptorSemanticsVersion, LoweredAspectContractPlan,
    RelationalSchemaRegistry, SchemaVersionId,
};
use crate::storage::overlay::{
    CurrentPartitionAccess, PartitionAccess, PartitionIdMemory, PartitionIdsDenial, PartitionState,
};
use crate::storage::residency::{
    Full, Head, LiveRecordCounts, PartitionHeadProbe, PartitionProbe, RelationalRegionHeadView,
    RelationalResidencyDemandFailure, RelationalResidencyLease,
};
use worth_execution::ExecutionRequest;

/// A restricted loan of the exact root while its original owner retains it.
#[derive(Clone, Debug)]
pub struct SnapshotProjectionRoot<'a> {
    pub(super) regions: &'a RelationalPersistentRegionSet,
    pub(super) schema: SnapshotProjectionSchema<'a>,
    pub(super) id: u64,
    pub(super) storage_version: Option<VersionId>,
}

impl<'a> SnapshotProjectionRoot<'a> {
    pub fn partition_access(&self) -> &'a dyn PartitionAccess {
        self.regions
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn storage_version(&self) -> Option<VersionId> {
        self.storage_version
    }

    pub fn schema_authority(&self) -> SnapshotProjectionSchema<'a> {
        self.schema.clone()
    }

    pub fn query_schema_context(
        &self,
    ) -> (
        crate::schema::data::SchemaVersionId,
        crate::schema::data::DescriptorSemanticsVersion,
        crate::query::data::QueryPlanEvidenceBasis,
    ) {
        (
            self.schema.version,
            self.schema.semantics,
            self.schema.evidence.clone(),
        )
    }

    pub fn live_counts(&self) -> LiveRecordCounts {
        self.regions.live_counts()
    }

    pub fn has_materialization_unavailable(&self) -> bool {
        self.regions.materialization_unavailable_records() != 0
    }

    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    pub fn entity_regions_of_kind(
        &self,
        kind: KindId,
    ) -> impl Iterator<Item = PartitionId> + 'a + use<'a> {
        self.regions.entity_regions_of_kind(kind)
    }

    pub fn relation_regions_of_kind(
        &self,
        kind: KindId,
    ) -> impl Iterator<Item = PartitionId> + 'a + use<'a> {
        self.regions.relation_regions_of_kind(kind)
    }

    pub fn entity_ids_of_kind(
        &self,
        kind: KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = EntityId> + 'a + use<'a> {
        self.regions.entity_ids_of_kind(kind, partition)
    }

    pub fn relation_ids_of_kind(
        &self,
        kind: KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = RelationId> + 'a + use<'a> {
        self.regions.relation_ids_of_kind(kind, partition)
    }

    pub fn try_for_each_partition_id<E>(
        &self,
        visit: impl FnMut(PartitionId) -> Result<(), E>,
    ) -> Result<(), E> {
        self.regions.try_for_each_partition_id(visit)
    }

    pub fn partition_ids_iter(&self) -> impl Iterator<Item = PartitionId> + 'a + use<'a> {
        self.regions.partition_ids_iter()
    }

    pub fn accepted_full_region(&self, id: PartitionId) -> Option<&'a PartitionState> {
        self.regions.accepted_full_region(id)
    }

    pub fn accepted_head_region_view(
        &self,
        id: PartitionId,
    ) -> Option<RelationalRegionHeadView<'a>> {
        self.regions.accepted_head_region_view(id)
    }

    pub fn accepted_head_region_witness(
        &self,
        id: PartitionId,
    ) -> Result<Option<RelationalResidencyLease<Head>>, PartitionId> {
        self.regions.accepted_head_region_witness(id)
    }

    pub fn demand_region_head(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<Option<RelationalResidencyLease<Head>>, RelationalResidencyDemandFailure> {
        self.regions.demand_region_head(execution, id)
    }

    pub fn demand_region_full(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<Option<RelationalResidencyLease<Full>>, RelationalResidencyDemandFailure> {
        self.regions.demand_region_full(execution, id)
    }

    pub fn demand_all_full(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &RuntimeSymbolTable,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.regions.demand_all_full(execution, symbols)
    }

    pub fn demand_regions_head(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &RuntimeSymbolTable,
        ids: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.regions.demand_regions_head(execution, symbols, ids)
    }

    pub fn demand_regions_full(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &RuntimeSymbolTable,
        ids: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.regions.demand_regions_full(execution, symbols, ids)
    }

    pub fn symbol_admitted_projection(
        &self,
        symbols: &'a RuntimeSymbolTable,
    ) -> impl CurrentPartitionAccess + PartitionAccess + 'a + use<'a> {
        SymbolAdmittedRootRead::new(self.regions, symbols)
    }
}

/// Immutable compiled meaning and coordinates carried by this selected basis.
#[derive(Clone, Debug)]
pub struct SnapshotProjectionSchema<'a> {
    pub(super) registry: &'a RelationalSchemaRegistry,
    pub(super) plans: &'a AspectContractPlanCatalog,
    pub(super) version: SchemaVersionId,
    pub(super) semantics: DescriptorSemanticsVersion,
    pub(super) digest: [u8; 32],
    pub(super) evidence: QueryPlanEvidenceBasis,
}

impl<'a> SnapshotProjectionSchema<'a> {
    pub fn registry(&self) -> &'a RelationalSchemaRegistry {
        self.registry
    }
    pub fn aspect_plans(&self) -> &'a AspectContractPlanCatalog {
        self.plans
    }
    pub fn schema_version(&self) -> SchemaVersionId {
        self.version
    }
    pub fn authority_digest(&self) -> [u8; 32] {
        self.digest
    }
    pub fn entity_aspect_plan(&self, kind: KindId) -> Option<&'a LoweredAspectContractPlan> {
        self.plans.entity_plans.get(&kind)
    }
    pub fn relation_aspect_plan(&self, kind: KindId) -> Option<&'a LoweredAspectContractPlan> {
        self.plans.relation_plans.get(&kind)
    }
    pub fn entity_aspect_plan_count(&self) -> usize {
        self.plans.entity_plans.len()
    }
    pub fn relation_aspect_plan_count(&self) -> usize {
        self.plans.relation_plans.len()
    }
}

impl PartitionAccess for SnapshotProjectionRoot<'_> {
    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        self.regions.get_partition(id)
    }
    fn probe_full_partition(&self, id: PartitionId) -> PartitionProbe<'_> {
        self.regions.probe_full_partition(id)
    }
    fn demand_full_partition(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.regions.demand_full_partition(execution, id)
    }
}

impl CurrentPartitionAccess for SnapshotProjectionRoot<'_> {
    fn get_current_partition(&self, id: PartitionId) -> Option<RelationalRegionHeadView<'_>> {
        self.regions.get_current_partition(id)
    }
    fn probe_head_partition(&self, id: PartitionId) -> PartitionHeadProbe<'_> {
        self.regions.probe_head_partition(id)
    }
    fn demand_head_partition(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.regions.demand_region_head_cached(execution, id)
    }
    fn partition_ids(&self) -> Vec<PartitionId> {
        self.regions.partition_ids_iter().collect()
    }
    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, PartitionIdsDenial> {
        self.regions.partition_ids_with_memory(memory)
    }
    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(self.regions.partition_ids_iter())
    }
}
