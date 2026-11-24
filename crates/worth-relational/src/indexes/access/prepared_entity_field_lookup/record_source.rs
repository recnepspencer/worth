//! Stream from the exact source already admitted during lookup preparation.
use super::*;

impl crate::visibility::materialization::read_records::projection::ProjectionRecordSource
    for PreparedEntityFieldLookup
{
    // Construction already admitted and retained the complete immutable source.
    // Reading an accepted pin checks request life, without reacquiring cold truth.
    fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::execution::denial_cause::RelationalExecutionDenialCause> {
        crate::execution::request_custody::check_request(execution)
    }
    fn demand_regions_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        _partitions: impl IntoIterator<Item = crate::identity::data::PartitionId>,
    ) -> Result<(), crate::execution::denial_cause::RelationalExecutionDenialCause> {
        crate::execution::request_custody::check_request(execution)
    }
    fn demand_regions_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: impl IntoIterator<Item = crate::identity::data::PartitionId>,
    ) -> Result<(), crate::execution::denial_cause::RelationalExecutionDenialCause> {
        self.basis
            .demand_regions_head(execution, partitions)
            .map_err(Into::into)
    }
    fn admitted_head_region(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.basis.head_region(id)
    }
    fn selected_schema_registry(&self) -> Option<&crate::schema::data::RelationalSchemaRegistry> {
        Some(self.basis.root().schema_authority().registry())
    }

    fn try_for_each_selected_partition<E>(
        &self,
        visit: impl FnMut(crate::identity::data::PartitionId) -> Result<(), E>,
    ) -> Result<(), E> {
        self.basis.root().try_for_each_partition_id(visit)
    }

    fn selected_entity_ids(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<crate::identity::data::PartitionId>,
    ) -> impl Iterator<Item = crate::identity::data::EntityId> + '_ {
        self.basis.root().entity_ids_of_kind(kind, partition)
    }

    fn selected_relation_ids(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<crate::identity::data::PartitionId>,
    ) -> impl Iterator<Item = crate::identity::data::RelationId> + '_ {
        self.basis.root().relation_ids_of_kind(kind, partition)
    }

    fn retained_head_region(
        &self,
        partition: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.basis.root().accepted_head_region_view(partition)
    }
    fn selected_storage(&self) -> Option<&dyn crate::storage::overlay::PartitionAccess> {
        Some(self.basis.root())
    }
    fn is_exact_basis(&self) -> bool {
        true
    }
    fn version_id(&self) -> crate::identity::data::VersionId {
        self.basis.version_id()
    }
}
