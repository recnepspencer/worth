//! Request-funded record walks have no independent work limit.
use super::super::read_prices::{EXAMINE_SLOT, RESERVE_RECORD};
use super::*;
impl ProjectionRecordReader<'_> {
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
                        self.instrumentation.count(|counters| {
                            counters.visible_authoritative_entity_records_materialized += 1;
                        });
                        records.push(record);
                    }
                }
            }
            KindScanVisibility::AtVersion(version) => {
                for slot in partition.entity_arena.occupied_slots_iter() {
                    charge(EXAMINE_SLOT)?;
                    self.instrumentation.count(|counters| {
                        counters.visibility_entity_slot_scans += 1;
                    });
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
                                self.current_version,
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
                        self.instrumentation.count(|counters| {
                            counters.visible_authoritative_entity_records_materialized += 1;
                        });
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
                        self.instrumentation.count(|counters| {
                            counters.visible_authoritative_relation_records_materialized += 1;
                        });
                        records.push(record);
                    }
                }
            }
            KindScanVisibility::AtVersion(version) => {
                for slot in partition.relation_arena.occupied_slots_iter() {
                    charge(EXAMINE_SLOT)?;
                    self.instrumentation.count(|counters| {
                        counters.visibility_relation_slot_scans += 1;
                    });
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
                                self.current_version,
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
                        self.instrumentation.count(|counters| {
                            counters.visible_authoritative_relation_records_materialized += 1;
                        });
                        records.push(record);
                    }
                }
            }
        }
        super::truth_record_access::sort_authoritative_relation_records(&mut records);
        Ok(records)
    }
}

impl VisibilityReadContext<'_> {
    pub(crate) fn paid_entities_in_partition(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        partition: crate::identity::data::PartitionId,
        kind: Option<crate::identity::data::KindId>,
        visibility: KindScanVisibility,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    ) -> Result<Vec<EntityReadRecord>, worth_execution::MapKernelStop> {
        ProjectionRecordReader::new(
            self.runtime.current_version_id(),
            &self.runtime.services.instrumentation,
        )
        .paid_entities_in_partition(state, registry, partition, kind, visibility, charge)
    }
    pub(crate) fn paid_relations_in_partition(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        partition: crate::identity::data::PartitionId,
        kind: Option<crate::identity::data::KindId>,
        visibility: KindScanVisibility,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    ) -> Result<Vec<RelationReadRecord>, worth_execution::MapKernelStop> {
        ProjectionRecordReader::new(
            self.runtime.current_version_id(),
            &self.runtime.services.instrumentation,
        )
        .paid_relations_in_partition(state, registry, partition, kind, visibility, charge)
    }
}

#[cfg(test)]
mod tests {
    use crate::identity::data::KindId;
    use crate::tests::support::*;
    #[test]
    fn paid_record_walk_reports_materializations_and_historical_slot_visits() {
        let host = crate::test_execution_policy::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let runtime = runtime_with_test_schema();
        let left = create_entity(&execution, &runtime, "left");
        let right = create_entity(&execution, &runtime, "right");
        let relation = create_relation_outcome(&execution, &runtime, left, right, "edge");
        let later = create_entity_outcome(&execution, &runtime, "later");
        let truth = runtime.read_truth();
        let current = truth.project_snapshot(&later.snapshot, &execution).unwrap();
        let historical = truth.project_historical_version(relation.version_id);
        for (view, entities, entity_scans, relation_scans) in
            [(&current, 3, 0, 0), (&historical, 2, 2, 1)]
        {
            runtime.performance_access().reset_counters();
            assert_eq!(
                view.authoritative_entity_records(&execution, KindId(1))
                    .unwrap()
                    .len(),
                entities
            );
            assert_eq!(
                view.authoritative_relation_records(&execution, KindId(2))
                    .unwrap()
                    .len(),
                1
            );
            let counters = runtime.performance_access().counters();
            assert_eq!(
                counters.visible_authoritative_entity_records_materialized,
                entities
            );
            assert_eq!(
                counters.visible_authoritative_relation_records_materialized,
                1
            );
            assert_eq!(counters.visibility_entity_slot_scans, entity_scans);
            assert_eq!(counters.visibility_relation_slot_scans, relation_scans);
        }
        release_test_commit_snapshot(&runtime, &relation);
        release_test_commit_snapshot(&runtime, &later);
    }
}
