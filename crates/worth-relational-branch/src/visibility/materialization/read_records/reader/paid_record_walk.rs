//! Request-funded record walks have no independent work limit.
use super::super::{
    read_prices::{EXAMINE_SLOT, RESERVE_RECORD},
    read_work::record_memory,
};
use super::record_class::{Entity, RecordClassOps, Relation};
use super::{
    kind_scan_visibility::KindScanVisibility, projection_record_reader::ProjectionRecordReader,
};
use crate::execution::RelationalExecutionDenialCause as Stop;
use crate::identity::data::{KindId, PartitionId};
use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::{
    data::{EntityReadRecord, RelationReadRecord},
    overlay::PartitionAccess,
};

impl ProjectionRecordReader<'_> {
    pub fn paid_entities_in_partition(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &RelationalSchemaRegistry,
        partition_id: PartitionId,
        kind: Option<KindId>,
        visibility: KindScanVisibility,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, Stop> {
        self.paid_records_in_partition::<Entity>(
            state,
            registry,
            partition_id,
            kind,
            visibility,
            charge,
            memory,
        )
    }

    pub fn paid_relations_in_partition(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &RelationalSchemaRegistry,
        partition_id: PartitionId,
        kind: Option<KindId>,
        visibility: KindScanVisibility,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<RelationReadRecord>, Stop> {
        self.paid_records_in_partition::<Relation>(
            state,
            registry,
            partition_id,
            kind,
            visibility,
            charge,
            memory,
        )
    }

    pub(in crate::internal_port::visibility::materialization::read_records) fn paid_records_in_partition<
        C: RecordClassOps,
    >(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &RelationalSchemaRegistry,
        partition_id: PartitionId,
        kind: Option<KindId>,
        visibility: KindScanVisibility,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<C::Record>, Stop> {
        let KindScanVisibility::AtVersion(version) = visibility else {
            return self.paid_current_records::<C>(
                state.get_current_partition(partition_id),
                registry,
                kind,
                charge,
                memory,
            );
        };
        let Some(partition) = state.get_partition(partition_id) else {
            return Ok(Vec::new());
        };
        let mut records = Vec::new();
        let (slots, held) = record_memory::occupied_slots(C::arena(partition), memory)?;
        for slot in slots {
            charge(EXAMINE_SLOT).map_err(crate::execution::denial::kernel_stop)?;
            C::count_historical_slot(self.instrumentation);
            if !C::visible(partition, slot, kind, version, self.current_version) {
                continue;
            }
            charge(RESERVE_RECORD).map_err(crate::execution::denial::kernel_stop)?;
            let clone_bytes = C::quote_history(registry, partition, slot, visibility)?;
            record_memory::prepare_push(memory, &mut records, clone_bytes)?;
            if let Some(record) = C::read_history(registry, partition, partition_id, slot, version)
            {
                C::count_materialization(self.instrumentation);
                record_memory::settle_clone(memory, clone_bytes, C::record_bytes(&record))?;
                records.push(record);
            } else {
                record_memory::release(memory, clone_bytes)?;
            }
        }
        record_memory::release(memory, held)?;
        C::order(&mut records);
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::{KindScanVisibility, ProjectionRecordReader};
    use crate::identity::data::{KindId, PartitionId, VersionId};
    use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
    use crate::schema::data::{
        EntityKindRegistration, RelationKindRegistration, RelationalSchemaRegistry, SchemaId,
        SchemaVersionId,
    };
    use crate::storage::overlay::small_relation_partition::relation_partition_for_test;
    use crate::visibility::materialization::read_records::read_work;
    use std::collections::BTreeMap;

    #[test]
    fn paid_record_walk_reports_materializations_and_historical_slot_visits() {
        let host = worth_harness::facade::execution_fixture::serial_request();
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
                relation_partition_for_test(partition_id, entities, &[(0, 1, KindId(2))]);
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
