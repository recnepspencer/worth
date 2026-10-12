use std::sync::Arc;

use crate::branch::RelationalBranchRoot;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::storage::overlay::{PartitionAccess, PartitionState};
use crate::visibility::snapshot_states::HistoricalVisibilityBasis;

use super::read_record_identity_ordering::relation_identity_order_key;
use super::VisibilityProjectionView;

pub(super) struct HistoricalProjectionReader<'view, 'runtime> {
    view: &'view VisibilityProjectionView<'runtime>,
    basis: &'view HistoricalVisibilityBasis,
}

pub(super) enum HistoricalProjectionStorage<'view> {
    Retained(&'view Arc<RelationalBranchRoot>),
    EmptyGenesis,
}

impl PartitionAccess for HistoricalProjectionStorage<'_> {
    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        match self {
            Self::Retained(root) => root.get_partition(partition_id),
            Self::EmptyGenesis => None,
        }
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        match self {
            Self::Retained(root) => root.partition_ids(),
            Self::EmptyGenesis => Vec::new(),
        }
    }

    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        match self {
            Self::Retained(root) => Box::new(root.partition_ids_iter()),
            Self::EmptyGenesis => Box::new(std::iter::empty()),
        }
    }
}

impl<'view, 'runtime> HistoricalProjectionReader<'view, 'runtime> {
    pub(super) const fn new(
        view: &'view VisibilityProjectionView<'runtime>,
        basis: &'view HistoricalVisibilityBasis,
    ) -> Self {
        Self { view, basis }
    }

    pub(super) fn entity_records(
        &self,
        kind: KindId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    ) -> Result<Vec<EntityReadRecord>, worth_execution::MapKernelStop> {
        self.collect_entity_records(Some(kind), charge)
    }
    pub(super) fn all_entity_records(
        &self,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    ) -> Result<Vec<EntityReadRecord>, worth_execution::MapKernelStop> {
        self.collect_entity_records(None, charge)
    }
    fn collect_entity_records(
        &self,
        kind: Option<KindId>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    ) -> Result<Vec<EntityReadRecord>, worth_execution::MapKernelStop> {
        let mut records = Vec::new();
        for partition in self.storage().partition_ids() {
            records.extend(self.entity_records_in_selected_partition(partition, kind, charge)?);
        }
        debug_assert!(
            super::read_record_identity_ordering::authoritative_entity_records_are_identity_ordered(
                &records
            )
        );
        Ok(records)
    }
    pub(super) fn entity_records_in(
        &self,
        partition: PartitionId,
        kind: KindId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    ) -> Result<Vec<EntityReadRecord>, worth_execution::MapKernelStop> {
        self.entity_records_in_selected_partition(partition, Some(kind), charge)
    }
    fn entity_records_in_selected_partition(
        &self,
        partition: PartitionId,
        kind: Option<KindId>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    ) -> Result<Vec<EntityReadRecord>, worth_execution::MapKernelStop> {
        self.view.reader().paid_entities_in_partition(
            &self.storage(),
            self.registry(),
            partition,
            kind,
            super::super::reader::KindScanVisibility::for_version(
                self.view.runtime,
                self.view.version_id(),
            ),
            charge,
        )
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
    ) -> Result<Vec<RelationReadRecord>, worth_execution::MapKernelStop> {
        self.collect_relation_records(Some(kind), charge)
    }
    pub(super) fn all_relation_records(
        &self,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    ) -> Result<Vec<RelationReadRecord>, worth_execution::MapKernelStop> {
        self.collect_relation_records(None, charge)
    }
    fn collect_relation_records(
        &self,
        kind: Option<KindId>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    ) -> Result<Vec<RelationReadRecord>, worth_execution::MapKernelStop> {
        let mut records = Vec::new();
        for partition in self.storage().partition_ids() {
            records.extend(self.relation_records_in_selected_partition(partition, kind, charge)?);
        }
        records.sort_by_key(relation_identity_order_key);
        debug_assert!(super::read_record_identity_ordering::authoritative_relation_records_are_identity_ordered(&records));
        Ok(records)
    }
    pub(super) fn relation_records_in(
        &self,
        partition: PartitionId,
        kind: KindId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    ) -> Result<Vec<RelationReadRecord>, worth_execution::MapKernelStop> {
        self.relation_records_in_selected_partition(partition, Some(kind), charge)
    }
    fn relation_records_in_selected_partition(
        &self,
        partition: PartitionId,
        kind: Option<KindId>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    ) -> Result<Vec<RelationReadRecord>, worth_execution::MapKernelStop> {
        self.view.reader().paid_relations_in_partition(
            &self.storage(),
            self.registry(),
            partition,
            kind,
            super::super::reader::KindScanVisibility::for_version(
                self.view.runtime,
                self.view.version_id(),
            ),
            charge,
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

    pub(super) fn storage(&self) -> HistoricalProjectionStorage<'_> {
        match self.basis.root() {
            Some(root) => HistoricalProjectionStorage::Retained(root),
            None => HistoricalProjectionStorage::EmptyGenesis,
        }
    }

    pub(super) fn registry(&self) -> &crate::schema::data::RelationalSchemaRegistry {
        self.basis
            .root()
            .map_or(&self.view.runtime.config.schema.registry, |root| {
                root.schema_authority().registry()
            })
    }
}
