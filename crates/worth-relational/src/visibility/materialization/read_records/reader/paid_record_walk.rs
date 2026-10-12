//! Request-funded record walks have no independent work limit.
use super::super::read_prices::{EXAMINE_SLOT, RESERVE_RECORD};
use super::*;
impl VisibilityReadContext<'_> {
    pub(crate) fn paid_entities_in_partition(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        partition_id: crate::identity::data::PartitionId,
        kind: Option<crate::identity::data::KindId>,
        visibility: KindScanVisibility,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    ) -> Result<Vec<EntityReadRecord>, worth_execution::MapKernelStop> {
        let Some(partition) = state.get_partition(partition_id) else {
            return Ok(Vec::new());
        };
        let mut records = Vec::new();
        match visibility {
            KindScanVisibility::Live => {
                for slot in partition.entity_arena.live_bitset.set_slots() {
                    charge(EXAMINE_SLOT)?;
                    if kind.is_some_and(|kind| {
                        !slot_kind_matches_current(&partition.entity_arena, slot, kind)
                    }) {
                        continue;
                    }
                    charge(RESERVE_RECORD)?;
                    if let Some(record) = materialize_current_authoritative_entity_record(
                        registry,
                        partition,
                        partition_id,
                        slot,
                    ) {
                        records.push(record);
                    }
                }
            }
            KindScanVisibility::AtVersion(version) => {
                for slot in partition.entity_arena.occupied_slots_iter() {
                    charge(EXAMINE_SLOT)?;
                    let visible = kind.map_or_else(
                        || {
                            super::super::visibility::entity_visible_in_partition_at_version(
                                partition, slot, version,
                            )
                        },
                        |kind| {
                            entity_slot_matches_kind_at_version(
                                partition,
                                slot,
                                kind,
                                version,
                                self.runtime.current_version_id(),
                            )
                        },
                    );
                    if !visible {
                        continue;
                    }
                    charge(RESERVE_RECORD)?;
                    if let Some(record) = materialize_authoritative_entity_record_at_version(
                        registry,
                        partition,
                        partition_id,
                        slot,
                        version,
                    ) {
                        records.push(record);
                    }
                }
            }
        }
        Ok(records)
    }
    pub(crate) fn paid_relations_in_partition(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        partition_id: crate::identity::data::PartitionId,
        kind: Option<crate::identity::data::KindId>,
        visibility: KindScanVisibility,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    ) -> Result<Vec<RelationReadRecord>, worth_execution::MapKernelStop> {
        let Some(partition) = state.get_partition(partition_id) else {
            return Ok(Vec::new());
        };
        let mut records = Vec::new();
        match visibility {
            KindScanVisibility::Live => {
                for slot in partition.relation_arena.live_bitset.set_slots() {
                    charge(EXAMINE_SLOT)?;
                    if kind.is_some_and(|kind| {
                        !slot_kind_matches_current(&partition.relation_arena, slot, kind)
                    }) {
                        continue;
                    }
                    charge(RESERVE_RECORD)?;
                    if let Some(record) = materialize_current_authoritative_relation_record(
                        registry,
                        partition,
                        partition_id,
                        slot,
                    ) {
                        records.push(record);
                    }
                }
            }
            KindScanVisibility::AtVersion(version) => {
                for slot in partition.relation_arena.occupied_slots_iter() {
                    charge(EXAMINE_SLOT)?;
                    let visible = kind.map_or_else(
                        || {
                            super::super::visibility::relation_visible_in_partition_at_version(
                                partition, slot, version,
                            )
                        },
                        |kind| {
                            relation_slot_matches_kind_at_version(
                                partition,
                                slot,
                                kind,
                                version,
                                self.runtime.current_version_id(),
                            )
                        },
                    );
                    if !visible {
                        continue;
                    }
                    charge(RESERVE_RECORD)?;
                    if let Some(record) = materialize_authoritative_relation_record_at_version(
                        registry,
                        partition,
                        partition_id,
                        slot,
                        version,
                    ) {
                        records.push(record);
                    }
                }
            }
        }
        super::truth_record_access::sort_authoritative_relation_records(&mut records);
        Ok(records)
    }
}
