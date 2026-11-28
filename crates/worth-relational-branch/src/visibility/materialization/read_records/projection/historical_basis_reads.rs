use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::storage::overlay::{PartitionAccess, PartitionState};
use crate::visibility::snapshot_states::HistoricalVisibilityBasis;

use super::super::read_work::record_memory;
use super::super::reader::kind_scan_visibility::KindScanVisibility;
use super::super::reader::record_class::{Entity, RecordClassOps, Relation};
use super::VisibilityProjectionView;
use crate::execution::RelationalExecutionDenialCause as Stop;
use worth_execution::{ExecutionResultMemory, MapKernelStop};

pub(super) struct HistoricalProjectionReader<'view> {
    view: &'view VisibilityProjectionView,
    basis: &'view HistoricalVisibilityBasis,
}

enum HistoricalProjectionStorage<T> {
    Retained(T),
    EmptyGenesis,
}

impl<T: PartitionAccess> PartitionAccess for HistoricalProjectionStorage<T> {
    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        match self {
            Self::Retained(root) => root.get_partition(partition_id),
            Self::EmptyGenesis => None,
        }
    }
}

impl<T: PartitionAccess> crate::storage::overlay::CurrentPartitionAccess
    for HistoricalProjectionStorage<T>
{
    fn get_current_partition(
        &self,
        id: PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        match self {
            Self::Retained(root) => root.get_current_partition(id),
            Self::EmptyGenesis => None,
        }
    }
    fn probe_head_partition(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        match self {
            Self::Retained(root) => root.probe_head_partition(id),
            Self::EmptyGenesis => crate::storage::residency::PartitionHeadProbe::Absent,
        }
    }
    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        match self {
            Self::Retained(root) => root.demand_head_partition(execution, id),
            Self::EmptyGenesis => crate::execution::request_custody::check_request(execution)
                .map_err(crate::storage::residency::RelationalResidencyDemandFailure::Request),
        }
    }
    fn partition_ids(&self) -> Vec<PartitionId> {
        match self {
            Self::Retained(root) => root.partition_ids(),
            Self::EmptyGenesis => Vec::new(),
        }
    }

    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        match self {
            Self::Retained(root) => root.partition_ids_with_memory(memory),
            Self::EmptyGenesis => Ok(Vec::new()),
        }
    }

    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        match self {
            Self::Retained(root) => root.partition_ids_iter(),
            Self::EmptyGenesis => Box::new(std::iter::empty()),
        }
    }
}

impl<'view> HistoricalProjectionReader<'view> {
    pub(super) const fn new(
        view: &'view VisibilityProjectionView,
        basis: &'view HistoricalVisibilityBasis,
    ) -> Self {
        Self { view, basis }
    }

    pub(super) fn entity_records(
        &self,
        kind: KindId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        self.collect_records::<Entity>(Some(kind), charge, memory)
    }
    pub(super) fn all_entity_records(
        &self,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        self.collect_records::<Entity>(None, charge, memory)
    }
    pub(super) fn entity_records_in(
        &self,
        partition: PartitionId,
        kind: KindId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        self.records_in_partition::<Entity>(&self.storage(), partition, Some(kind), charge, memory)
    }
    pub(super) fn entity_record(&self, entity_id: EntityId) -> Option<EntityReadRecord> {
        let storage = self.storage();
        let registry = self.registry();
        self.view
            .reader()
            .authoritative_entity_record_for_id_at_version_with_registry(
                &storage,
                registry,
                entity_id,
                self.view.version_id(),
            )
    }

    pub(super) fn relation_records(
        &self,
        kind: KindId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.collect_records::<Relation>(Some(kind), charge, memory)
    }
    pub(super) fn all_relation_records(
        &self,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.collect_records::<Relation>(None, charge, memory)
    }
    pub(super) fn relation_records_in(
        &self,
        partition: PartitionId,
        kind: KindId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.records_in_partition::<Relation>(
            &self.storage(),
            partition,
            Some(kind),
            charge,
            memory,
        )
    }
    pub(super) fn relation_record(&self, relation_id: RelationId) -> Option<RelationReadRecord> {
        let storage = self.storage();
        let registry = self.registry();
        self.view
            .reader()
            .authoritative_relation_record_for_id_at_version_with_registry(
                &storage,
                registry,
                relation_id,
                self.view.version_id(),
            )
    }

    fn collect_records<C: RecordClassOps>(
        &self,
        kind: Option<KindId>,
        charge: &mut dyn FnMut(u64) -> Result<(), MapKernelStop>,
        memory: &mut ExecutionResultMemory<'_>,
    ) -> Result<Vec<C::Record>, Stop> {
        let storage = self.storage();
        let mut records = Vec::new();
        let (partitions, partition_bytes) = record_memory::partition_ids(&storage, memory)?;
        for partition in partitions {
            let incoming =
                self.records_in_partition::<C>(&storage, partition, kind, charge, memory)?;
            record_memory::append(memory, &mut records, incoming)?;
        }
        record_memory::release(memory, partition_bytes)?;
        C::order(&mut records);
        Ok(records)
    }

    fn records_in_partition<C: RecordClassOps>(
        &self,
        storage: &impl PartitionAccess,
        partition: PartitionId,
        kind: Option<KindId>,
        charge: &mut dyn FnMut(u64) -> Result<(), MapKernelStop>,
        memory: &mut ExecutionResultMemory<'_>,
    ) -> Result<Vec<C::Record>, Stop> {
        self.view.reader().paid_records_in_partition::<C>(
            storage,
            self.registry(),
            partition,
            kind,
            KindScanVisibility::for_selected_version(
                self.view.current_version,
                self.view.version_id(),
            ),
            charge,
            memory,
        )
    }

    pub(super) fn storage(&self) -> impl PartitionAccess + '_ {
        match self.basis.root() {
            Some(root) => HistoricalProjectionStorage::Retained(root),
            None => HistoricalProjectionStorage::EmptyGenesis,
        }
    }

    pub(super) fn registry(&self) -> &crate::schema::data::RelationalSchemaRegistry {
        match self.basis.root() {
            Some(root) => root.schema_authority().registry(),
            None => self
                .view
                .empty_history_schema
                .as_deref()
                .expect("an empty historical projection retains its fallback schema"),
        }
    }
}
