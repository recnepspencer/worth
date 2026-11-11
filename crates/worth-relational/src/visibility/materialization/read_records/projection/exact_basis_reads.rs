use std::sync::Arc;

use crate::branch::RelationalBranchRoot;
use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};

#[cfg(test)]
use super::read_record_identity_ordering::relation_identity_order_key;
use super::VisibilityProjectionView;

pub(in crate::visibility::materialization::read_records) struct ExactProjectionReader<'view> {
    view: &'view VisibilityProjectionView,
    root: &'view Arc<RelationalBranchRoot>,
}

#[cfg(test)]
mod census;
impl<'view> ExactProjectionReader<'view> {
    pub(super) const fn new(
        view: &'view VisibilityProjectionView,
        root: &'view Arc<RelationalBranchRoot>,
    ) -> Self {
        Self { view, root }
    }
    pub(in crate::visibility::materialization::read_records) fn registry(
        &self,
    ) -> &crate::schema::data::RelationalSchemaRegistry {
        self.root.schema_authority().registry()
    }
    pub(in crate::visibility::materialization::read_records) fn entity_ids(
        &self,
        kind: KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = EntityId> + '_ {
        self.root.entity_ids_of_kind(kind, partition)
    }
    pub(in crate::visibility::materialization::read_records) fn relation_ids(
        &self,
        kind: KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = crate::identity::data::RelationId> + '_ {
        self.root.relation_ids_of_kind(kind, partition)
    }
    pub(in crate::visibility::materialization::read_records) fn demand_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition: PartitionId,
    ) -> Result<
        crate::storage::residency::RelationalRegionHeadView<'_>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        self.view.demand_regions_head(execution, [partition])?;
        self.view
            .admitted_head_region(partition)
            .ok_or_else(|| self.membership_failure(partition))
    }
    pub(in crate::visibility::materialization::read_records) fn membership_failure(
        &self,
        partition: PartitionId,
    ) -> crate::execution::RelationalExecutionDenialCause {
        crate::execution::RelationalExecutionDenialCause::StorageUnavailable(
            crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(partition),
                cause: crate::storage::data::RelationalStorageUnavailableCause::IntegrityMismatch,
            },
        )
    }
    pub(super) fn entity_records(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind: KindId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        self.view
            .reader()
            .entities_from_directory(execution, self, kind, None, None, charge, memory)
            .map(|read| read.into_records())
            .map_err(unbounded_stop)
    }
    pub(super) fn entity_records_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition: PartitionId,
        kind: KindId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        self.view
            .reader()
            .entities_from_directory(execution, self, kind, Some(partition), None, charge, memory)
            .map(|read| read.into_records())
            .map_err(unbounded_stop)
    }
    pub(super) fn relation_records(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind: KindId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        self.view
            .reader()
            .relations_from_directory(execution, self, kind, None, None, charge, memory)
            .map(|read| read.into_records())
            .map_err(unbounded_stop)
    }
    pub(super) fn relation_records_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition: PartitionId,
        kind: KindId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        self.view
            .reader()
            .relations_from_directory(execution, self, kind, Some(partition), None, charge, memory)
            .map(|read| read.into_records())
            .map_err(unbounded_stop)
    }
    pub(super) fn entity_record(&self, entity_id: EntityId) -> Option<EntityReadRecord> {
        self.view
            .admitted_head_region(entity_id.partition_id)?
            .entity_record(self.registry(), entity_id)
    }
    #[cfg(test)]
    pub(super) fn all_entity_records(
        &self,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        self.collect_entity_records(None, charge, memory)
    }
    #[cfg(test)]
    pub(super) fn all_relation_records(
        &self,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        self.collect_relation_records(None, charge, memory)
    }
}
fn unbounded_stop<L>(
    stop: super::super::native_scan_stop::NativeScanStop<L>,
) -> crate::execution::RelationalExecutionDenialCause {
    match stop {
        super::super::native_scan_stop::NativeScanStop::Request(cause) => cause,
        super::super::native_scan_stop::NativeScanStop::Limit(_) => {
            crate::execution::RelationalExecutionDenialCause::WorkExhausted
        }
    }
}
