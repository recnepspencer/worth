use crate::branch::root::projection::SnapshotProjectionRoot;
use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};

use super::read_record_identity_ordering::relation_identity_order_key;
use super::VisibilityProjectionView;

pub(in crate::internal_port::visibility::materialization::read_records) struct ExactProjectionReader<
    'view,
> {
    view: &'view VisibilityProjectionView,
    root: SnapshotProjectionRoot<'view>,
}

mod census;
impl<'view> ExactProjectionReader<'view> {
    pub(super) const fn new(
        view: &'view VisibilityProjectionView,
        root: SnapshotProjectionRoot<'view>,
    ) -> Self {
        Self { view, root }
    }
    pub(in crate::internal_port::visibility::materialization::read_records) fn registry(
        &self,
    ) -> &crate::schema::data::RelationalSchemaRegistry {
        self.root.schema_authority().registry()
    }
    pub(in crate::internal_port::visibility::materialization::read_records) fn entity_ids(
        &self,
        kind: KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = EntityId> + '_ {
        self.root.entity_ids_of_kind(kind, partition)
    }
    pub(in crate::internal_port::visibility::materialization::read_records) fn relation_ids(
        &self,
        kind: KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = crate::identity::data::RelationId> + '_ {
        self.root.relation_ids_of_kind(kind, partition)
    }
    /// One rolling-window step: an owned Head borrow of partition that the
    /// scan drops before borrowing the next, so a scan never pins the root.
    /// A pinned read lends from its own pin custody instead.
    pub(in crate::internal_port::visibility::materialization::read_records) fn borrow_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition: PartitionId,
    ) -> Result<
        crate::storage::residency::RelationalResidencyLease<crate::storage::residency::Head>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        if let Some(resident) = &self.view.resident {
            resident.demand_regions_head(execution, [partition])?;
            return resident
                .basis()
                .projection_root()
                .accepted_head_region_witness(partition)
                .ok()
                .flatten()
                .ok_or_else(|| self.membership_failure(partition));
        }
        let lease = self
            .root
            .borrow_region_head(execution, partition)?
            .ok_or_else(|| self.membership_failure(partition))?;
        lease.admit_symbols(execution, &self.view.symbols)?;
        Ok(lease)
    }

    pub(in crate::internal_port::visibility::materialization::read_records) fn membership_failure(
        &self,
        partition: PartitionId,
    ) -> crate::execution::denial_cause::RelationalExecutionDenialCause {
        crate::execution::denial_cause::RelationalExecutionDenialCause::StorageUnavailable(
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
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
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
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
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
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
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
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
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
    pub(super) fn all_entity_records(
        &self,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        self.collect_entity_records(None, charge, memory)
    }
    pub(super) fn all_relation_records(
        &self,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.collect_relation_records(None, charge, memory)
    }
}
fn unbounded_stop<L>(
    stop: super::super::native_scan_stop::NativeScanStop<L>,
) -> crate::execution::denial_cause::RelationalExecutionDenialCause {
    match stop {
        super::super::native_scan_stop::NativeScanStop::Request(cause) => cause,
        super::super::native_scan_stop::NativeScanStop::Limit(_) => {
            crate::execution::denial_cause::RelationalExecutionDenialCause::WorkExhausted
        }
    }
}
