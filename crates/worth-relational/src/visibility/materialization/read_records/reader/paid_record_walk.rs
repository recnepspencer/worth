//! Request-funded record walks have no independent work limit.
use super::super::materialization::{
    materialize_authoritative_entity_record_at_version,
    materialize_authoritative_relation_record_at_version,
};
use super::super::read_prices::{EXAMINE_SLOT, RESERVE_RECORD};
use super::super::visibility::{
    entity_slot_matches_kind_at_version, relation_slot_matches_kind_at_version,
};
use super::{
    kind_scan_visibility::KindScanVisibility, projection_record_reader::ProjectionRecordReader,
};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::storage::overlay::PartitionAccess;
impl ProjectionRecordReader<'_> {
    pub(crate) fn paid_entities_in_partition(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        partition_id: crate::identity::data::PartitionId,
        kind: Option<crate::identity::data::KindId>,
        visibility: KindScanVisibility,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        if matches!(visibility, KindScanVisibility::Live) {
            return self.paid_current_entities(
                state.get_current_partition(partition_id),
                registry,
                kind,
                charge,
                memory,
            );
        }
        let Some(partition) = state.get_partition(partition_id) else {
            return Ok(Vec::new());
        };
        let mut records = Vec::new();
        match visibility {
            KindScanVisibility::Live => unreachable!(
                /* Proof: The current-read dispatch returns via Head before this historical branch; only an immutable historical basis reaches this match. */
                "current reads returned through Head"
            ),
            KindScanVisibility::AtVersion(version) => {
                let (slots, held) = super::super::read_work::record_memory::occupied_slots(
                    &partition.entity_arena,
                    memory,
                )?;
                for slot in slots {
                    charge(EXAMINE_SLOT).map_err(crate::execution::denial::kernel_stop)?;
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
                    charge(RESERVE_RECORD).map_err(crate::execution::denial::kernel_stop)?;
                    let clone_bytes = super::super::read_work::record_quote::entity(
                        registry, partition, slot, visibility,
                    )?;
                    super::super::read_work::record_memory::prepare_push(
                        memory,
                        &mut records,
                        clone_bytes,
                    )?;
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
                        super::super::read_work::record_memory::settle_clone(
                            memory,
                            clone_bytes,
                            super::super::read_footprint::entity(&record),
                        )?;
                        records.push(record);
                    } else {
                        super::super::read_work::record_memory::release(memory, clone_bytes)?;
                    }
                }
                super::super::read_work::record_memory::release(memory, held)?;
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
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        if matches!(visibility, KindScanVisibility::Live) {
            return self.paid_current_relations(
                state.get_current_partition(partition_id),
                registry,
                kind,
                charge,
                memory,
            );
        }
        let Some(partition) = state.get_partition(partition_id) else {
            return Ok(Vec::new());
        };
        let mut records = Vec::new();
        match visibility {
            KindScanVisibility::Live => unreachable!(
                /* Proof: The current-read dispatch returns via Head before this historical branch; only an immutable historical basis reaches this match. */
                "current reads returned through Head"
            ),
            KindScanVisibility::AtVersion(version) => {
                let (slots, held) = super::super::read_work::record_memory::occupied_slots(
                    &partition.relation_arena,
                    memory,
                )?;
                for slot in slots {
                    charge(EXAMINE_SLOT).map_err(crate::execution::denial::kernel_stop)?;
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
                    charge(RESERVE_RECORD).map_err(crate::execution::denial::kernel_stop)?;
                    let clone_bytes = super::super::read_work::record_quote::relation(
                        registry, partition, slot, visibility,
                    )?;
                    super::super::read_work::record_memory::prepare_push(
                        memory,
                        &mut records,
                        clone_bytes,
                    )?;
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
                        super::super::read_work::record_memory::settle_clone(
                            memory,
                            clone_bytes,
                            super::super::read_footprint::relation(&record),
                        )?;
                        records.push(record);
                    } else {
                        super::super::read_work::record_memory::release(memory, clone_bytes)?;
                    }
                }
                super::super::read_work::record_memory::release(memory, held)?;
            }
        }
        super::truth_record_access::sort_authoritative_relation_records(&mut records);
        Ok(records)
    }
}

#[path = "paid_record_walk/runtime_reads.rs"]
mod runtime_reads;

#[cfg(test)]
mod tests {
    use super::{KindScanVisibility, ProjectionRecordReader};
    use crate::identity::data::{KindId, PartitionId, VersionId};
    use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
    use crate::schema::data::{
        EntityKindRegistration, RelationKindRegistration, RelationalSchemaRegistry, SchemaId,
        SchemaVersionId,
    };
    use crate::storage::overlay::small_relation_partition::relation_partition;
    use crate::visibility::materialization::read_records::read_work;
    use std::collections::BTreeMap;

    #[test]
    fn paid_record_walk_reports_materializations_and_historical_slot_visits() {
        let host = crate::execution::test_policy::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let registry = RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(1),
                kind_name: "record".into(),
                schema_id: SchemaId("record-walk".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: Default::default(),
            })
            .unwrap()
            .register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "edge".into(),
                schema_id: SchemaId("record-walk".into()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: crate::config::data::CrossContextPolicy::Forbid,
                cascade_delete_policy:
                    crate::config::data::CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: Default::default(),
                relation_integrity: Default::default(),
            })
            .unwrap();
        let partition_id = PartitionId::main();
        for (visibility, entities, entity_scans, relation_scans) in [
            (KindScanVisibility::Live, 3, 0, 0),
            (KindScanVisibility::AtVersion(VersionId(1)), 2, 2, 1),
        ] {
            let (partition, _, _) =
                relation_partition(partition_id, entities, &[(0, 1, KindId(2))]);
            let partitions = BTreeMap::from([(partition_id, partition)]);
            let instrumentation = RuntimeInstrumentation::default();
            let reader = ProjectionRecordReader::new(VersionId(2), &instrumentation);
            let found_entities = read_work::metered_with_memory(
                &execution,
                |context, memory| {
                    reader
                        .paid_entities_in_partition(
                            &partitions,
                            &registry,
                            partition_id,
                            Some(KindId(1)),
                            visibility,
                            &mut |units| context.checkpoint(units),
                            memory,
                        )
                        .map(|rows| rows.len())
                },
                |_| 0,
            )
            .unwrap()
            .unwrap();
            assert_eq!(found_entities, entities);
            let found_relations = read_work::metered_with_memory(
                &execution,
                |context, memory| {
                    reader
                        .paid_relations_in_partition(
                            &partitions,
                            &registry,
                            partition_id,
                            Some(KindId(2)),
                            visibility,
                            &mut |units| context.checkpoint(units),
                            memory,
                        )
                        .map(|rows| rows.len())
                },
                |_| 0,
            )
            .unwrap()
            .unwrap();
            assert_eq!(found_relations, 1);
            let counters = instrumentation.complexity_snapshot();
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
    }
}

#[cfg(test)]
mod preclaim_tests;
