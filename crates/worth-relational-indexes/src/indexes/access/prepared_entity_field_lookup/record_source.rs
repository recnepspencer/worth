//! Stream from the exact source already admitted during lookup preparation.
use super::*;

impl worth_relational_branch::internal_port::visibility::materialization::read_records::projection::streamed_basis_reads::ProjectionRecordSource
    for PreparedEntityFieldLookup
{
    // Construction already admitted and retained the complete immutable source.
    // Reading an accepted pin checks request life, without reacquiring cold truth.
    fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::contracts::execution::denial_cause::RelationalExecutionDenialCause> {
        crate::contracts::execution::request_custody::check_request(execution)
    }
    fn demand_regions_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        _partitions: impl IntoIterator<Item = crate::contracts::identity::data::primitives::PartitionId>,
    ) -> Result<(), crate::contracts::execution::denial_cause::RelationalExecutionDenialCause> {
        crate::contracts::execution::request_custody::check_request(execution)
    }
    fn demand_regions_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: impl IntoIterator<Item = crate::contracts::identity::data::primitives::PartitionId>,
    ) -> Result<(), crate::contracts::execution::denial_cause::RelationalExecutionDenialCause> {
        self.basis
            .demand_regions_head(execution, partitions)
            .map_err(Into::into)
    }
    fn admitted_head_region(
        &self,
        id: crate::contracts::identity::data::primitives::PartitionId,
    ) -> Option<worth_relational_storage::internal_port::storage::residency::views::RelationalRegionHeadView<'_>> {
        self.basis.head_region(id)
    }
    fn selected_schema_registry(&self) -> Option<&crate::contracts::schema::data::RelationalSchemaRegistry> {
        Some(self.basis.root().schema_authority().registry())
    }

    fn try_for_each_selected_partition<E>(
        &self,
        visit: impl FnMut(crate::contracts::identity::data::primitives::PartitionId) -> Result<(), E>,
    ) -> Result<(), E> {
        self.basis.root().try_for_each_partition_id(visit)
    }

    fn selected_entity_ids(
        &self,
        kind: crate::contracts::identity::data::primitives::KindId,
        partition: Option<crate::contracts::identity::data::primitives::PartitionId>,
    ) -> impl Iterator<Item = crate::contracts::identity::data::record_ids::EntityId> + '_ {
        self.basis.root().entity_ids_of_kind(kind, partition)
    }

    fn selected_relation_ids(
        &self,
        kind: crate::contracts::identity::data::primitives::KindId,
        partition: Option<crate::contracts::identity::data::primitives::PartitionId>,
    ) -> impl Iterator<Item = crate::contracts::identity::data::record_ids::RelationId> + '_ {
        self.basis.root().relation_ids_of_kind(kind, partition)
    }

    fn retained_head_region(
        &self,
        partition: crate::contracts::identity::data::primitives::PartitionId,
    ) -> Option<worth_relational_storage::internal_port::storage::residency::views::RelationalRegionHeadView<'_>> {
        self.basis
            .basis()
            .projection_root()
            .accepted_head_region_view(partition)
    }
    fn selected_storage(&self) -> Option<&dyn worth_relational_storage::internal_port::storage::overlay::access::PartitionAccess> {
        Some(self.basis.basis())
    }
    fn is_exact_basis(&self) -> bool {
        true
    }
    fn version_id(&self) -> crate::contracts::identity::data::primitives::VersionId {
        self.basis.version_id()
    }
}
