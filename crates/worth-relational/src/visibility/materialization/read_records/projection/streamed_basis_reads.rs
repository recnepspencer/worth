use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::storage::overlay::PartitionAccess;
use crate::storage::overlay::PartitionState;
use crate::visibility::materialization::read_records::visibility::{
    visible_metadata, visible_relation_metadata,
};

use super::VisibilityProjectionView;

impl VisibilityProjectionView<'_> {
    pub(crate) fn try_for_each_entity_record_in<E>(
        &self,
        partition_id: PartitionId,
        kind_id: Option<KindId>,
        mut before_candidate: impl FnMut(u64) -> Result<(), E>,
        mut visit: impl FnMut(&EntityReadRecord) -> Result<(), E>,
    ) -> Result<(), E> {
        let Some(root) = self.selected_root() else {
            return Ok(());
        };
        let Some(partition) = root.get_partition(partition_id) else {
            return Ok(());
        };
        let mut examine = |slot: usize| {
            before_candidate(self.candidate_entity_bytes(partition, slot))?;
            if let Some(record) =
                self.authoritative_entity_record(EntityId::new(partition_id, slot as u64, 0))
            {
                if kind_id.is_none_or(|kind_id| record.kind.kind_id == kind_id) {
                    visit(&record)?;
                }
            }
            Ok(())
        };
        if self.is_exact_basis() {
            for slot in partition.entity_arena.live_bitset.set_slots() {
                examine(slot)?;
            }
        } else {
            for slot in partition.entity_arena.occupied_slots_iter() {
                examine(slot)?;
            }
        }
        Ok(())
    }

    pub(crate) fn try_for_each_relation_record_in<E>(
        &self,
        partition_id: PartitionId,
        kind_id: Option<KindId>,
        mut before_candidate: impl FnMut(u64) -> Result<(), E>,
        mut visit: impl FnMut(&RelationReadRecord) -> Result<(), E>,
    ) -> Result<(), E> {
        let Some(root) = self.selected_root() else {
            return Ok(());
        };
        let Some(partition) = root.get_partition(partition_id) else {
            return Ok(());
        };
        let mut examine = |slot: usize| {
            before_candidate(self.candidate_relation_bytes(partition, slot))?;
            if let Some(record) =
                self.authoritative_relation_record(RelationId::new(partition_id, slot as u64, 0))
            {
                if kind_id.is_none_or(|kind_id| record.kind.kind_id == kind_id) {
                    visit(&record)?;
                }
            }
            Ok(())
        };
        if self.is_exact_basis() {
            for slot in partition.relation_arena.live_bitset.set_slots() {
                examine(slot)?;
            }
        } else {
            for slot in partition.relation_arena.occupied_slots_iter() {
                examine(slot)?;
            }
        }
        Ok(())
    }

    pub(crate) fn try_for_each_entity_record<E>(
        &self,
        kind_id: Option<KindId>,
        mut before_candidate: impl FnMut(u64) -> Result<(), E>,
        mut visit: impl FnMut(&EntityReadRecord) -> Result<(), E>,
    ) -> Result<(), E> {
        let Some(root) = self.selected_root() else {
            return Ok(());
        };
        root.try_for_each_partition_id(|partition_id| {
            self.try_for_each_entity_record_in(
                partition_id,
                kind_id,
                &mut before_candidate,
                &mut visit,
            )
        })
    }

    pub(crate) fn try_for_each_relation_record<E>(
        &self,
        kind_id: Option<KindId>,
        mut before_candidate: impl FnMut(u64) -> Result<(), E>,
        mut visit: impl FnMut(&RelationReadRecord) -> Result<(), E>,
    ) -> Result<(), E> {
        let Some(root) = self.selected_root() else {
            return Ok(());
        };
        root.try_for_each_partition_id(|partition_id| {
            self.try_for_each_relation_record_in(
                partition_id,
                kind_id,
                &mut before_candidate,
                &mut visit,
            )
        })
    }

    pub(crate) fn candidate_entity_bytes_for_id(&self, entity_id: EntityId) -> u64 {
        self.selected_root()
            .and_then(|root| root.get_partition(entity_id.partition_id))
            .map_or(0, |partition| {
                self.candidate_entity_bytes(partition, entity_id.slot_index())
            })
    }

    pub(crate) fn candidate_relation_bytes_for_id(&self, relation_id: RelationId) -> u64 {
        self.selected_root()
            .and_then(|root| root.get_partition(relation_id.partition_id))
            .map_or(0, |partition| {
                self.candidate_relation_bytes(partition, relation_id.slot_index())
            })
    }

    fn candidate_entity_bytes(&self, partition: &PartitionState, slot: usize) -> u64 {
        let selected = if self.is_exact_basis() {
            partition.entity_arena.get_slot(slot).and_then(|record| {
                Some((
                    record.kind_id()?,
                    record
                        .extra()
                        .authoritative_aspect_state
                        .as_ref()
                        .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64),
                ))
            })
        } else {
            partition
                .entity_arena
                .metadata_history_at(slot)
                .and_then(|history| visible_metadata(history, self.version_id()))
                .map(|metadata| {
                    (
                        metadata.kind_id,
                        metadata
                            .authoritative_aspect_state
                            .as_ref()
                            .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64),
                    )
                })
        };
        let Some((kind_id, state_bytes)) = selected else {
            return 0;
        };
        let names = self
            .selected_schema_authority()
            .and_then(|authority| authority.registry().entity_registration(kind_id).ok())
            .map_or(0, |registration| {
                registration
                    .kind_name
                    .capacity()
                    .saturating_add(registration.schema_id.0.capacity()) as u64
            });
        (std::mem::size_of::<EntityReadRecord>() as u64)
            .saturating_add(names)
            .saturating_add(state_bytes)
    }

    fn candidate_relation_bytes(&self, partition: &PartitionState, slot: usize) -> u64 {
        let selected = if self.is_exact_basis() {
            partition.relation_arena.get_slot(slot).and_then(|record| {
                Some((
                    record.kind_id()?,
                    record
                        .extra()
                        .authoritative_aspect_state
                        .as_ref()
                        .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64),
                ))
            })
        } else {
            visible_relation_metadata(partition, slot, self.version_id()).map(|metadata| {
                (
                    metadata.kind_id,
                    metadata
                        .authoritative_aspect_state
                        .as_ref()
                        .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64),
                )
            })
        };
        let Some((kind_id, state_bytes)) = selected else {
            return 0;
        };
        let names = self
            .selected_schema_authority()
            .and_then(|authority| authority.registry().relation_registration(kind_id).ok())
            .map_or(0, |registration| {
                registration
                    .kind_name
                    .capacity()
                    .saturating_add(registration.schema_id.0.capacity()) as u64
            });
        (std::mem::size_of::<RelationReadRecord>() as u64)
            .saturating_add(names)
            .saturating_add(state_bytes)
    }
}
