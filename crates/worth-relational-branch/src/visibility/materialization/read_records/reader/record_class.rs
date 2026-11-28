//! Entity and relation operations for owner-funded scans of exact or historical records.
use super::super::{materialization, read_footprint, read_work::record_quote, visibility};
use super::kind_scan_visibility::KindScanVisibility;
use crate::execution::RelationalExecutionDenialCause as Stop;
use crate::identity::data::{EntityId, KindId, PartitionId, RecordId, VersionId};
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::storage::overlay::PartitionState;
use crate::storage::residency::{RelationalRecordHeadArenaView, RelationalRegionHeadView};
use crate::storage::substrate::{EntityRecordKind, RecordArena, RecordKind, RelationRecordKind};

pub(in crate::internal_port::visibility::materialization::read_records) trait RecordClassOps {
    type Kind: RecordKind;
    type Record;

    fn head_arena<'a>(
        head: RelationalRegionHeadView<'a>,
    ) -> RelationalRecordHeadArenaView<'a, Self::Kind>;
    fn arena(partition: &PartitionState) -> &RecordArena<Self::Kind>;
    fn visible(
        partition: &PartitionState,
        slot: usize,
        kind: Option<KindId>,
        version: VersionId,
        current: VersionId,
    ) -> bool;
    fn quote_head(
        registry: &RelationalSchemaRegistry,
        head: RelationalRegionHeadView<'_>,
        slot: usize,
    ) -> Result<u64, Stop>;
    fn quote_history(
        registry: &RelationalSchemaRegistry,
        partition: &PartitionState,
        slot: usize,
        visibility: KindScanVisibility,
    ) -> Result<u64, Stop>;
    fn read_head(
        registry: &RelationalSchemaRegistry,
        head: RelationalRegionHeadView<'_>,
        slot: usize,
    ) -> Option<Self::Record> {
        Self::read_exact_head(
            registry,
            head,
            RecordId::new(head.partition_id(), slot as u64, 0),
        )
    }
    fn read_exact_head(
        registry: &RelationalSchemaRegistry,
        head: RelationalRegionHeadView<'_>,
        id: RecordId<<Self::Kind as RecordKind>::Domain>,
    ) -> Option<Self::Record>;
    fn read_history(
        registry: &RelationalSchemaRegistry,
        partition: &PartitionState,
        id: PartitionId,
        slot: usize,
        version: VersionId,
    ) -> Option<Self::Record>;
    fn record_bytes(record: &Self::Record) -> u64;
    fn count_materialization(instrumentation: &RuntimeInstrumentation);
    fn count_historical_slot(instrumentation: &RuntimeInstrumentation);
    fn order(records: &mut [Self::Record]);
}

pub(in crate::internal_port::visibility::materialization::read_records) struct Entity;
impl RecordClassOps for Entity {
    type Kind = EntityRecordKind;
    type Record = EntityReadRecord;

    fn head_arena<'a>(
        head: RelationalRegionHeadView<'a>,
    ) -> RelationalRecordHeadArenaView<'a, Self::Kind> {
        head.entity_arena()
    }
    fn arena(partition: &PartitionState) -> &RecordArena<Self::Kind> {
        &partition.entity_arena
    }
    fn visible(
        partition: &PartitionState,
        slot: usize,
        kind: Option<KindId>,
        version: VersionId,
        current: VersionId,
    ) -> bool {
        match kind {
            None => visibility::entity_visible_in_partition_at_version(partition, slot, version),
            Some(kind) => visibility::entity_slot_matches_kind_at_version(
                partition, slot, kind, version, current,
            ),
        }
    }
    fn quote_head(
        registry: &RelationalSchemaRegistry,
        head: RelationalRegionHeadView<'_>,
        slot: usize,
    ) -> Result<u64, Stop> {
        record_quote::entity_head(registry, head, slot)
    }
    fn quote_history(
        registry: &RelationalSchemaRegistry,
        partition: &PartitionState,
        slot: usize,
        visibility: KindScanVisibility,
    ) -> Result<u64, Stop> {
        record_quote::entity(registry, partition, slot, visibility)
    }
    fn read_exact_head(
        registry: &RelationalSchemaRegistry,
        head: RelationalRegionHeadView<'_>,
        id: EntityId,
    ) -> Option<Self::Record> {
        head.entity_record(registry, id)
    }
    fn read_history(
        registry: &RelationalSchemaRegistry,
        partition: &PartitionState,
        id: PartitionId,
        slot: usize,
        version: VersionId,
    ) -> Option<Self::Record> {
        materialization::materialize_authoritative_entity_record_at_version(
            registry, partition, id, slot, version,
        )
    }
    fn record_bytes(record: &Self::Record) -> u64 {
        read_footprint::entity(record)
    }
    fn count_materialization(instrumentation: &RuntimeInstrumentation) {
        instrumentation
            .count(|counters| counters.visible_authoritative_entity_records_materialized += 1);
    }
    fn count_historical_slot(instrumentation: &RuntimeInstrumentation) {
        instrumentation.count(|counters| counters.visibility_entity_slot_scans += 1);
    }
    fn order(_records: &mut [Self::Record]) {}
}

pub(in crate::internal_port::visibility::materialization::read_records) struct Relation;
impl RecordClassOps for Relation {
    type Kind = RelationRecordKind;
    type Record = RelationReadRecord;

    fn head_arena<'a>(
        head: RelationalRegionHeadView<'a>,
    ) -> RelationalRecordHeadArenaView<'a, Self::Kind> {
        head.relation_arena()
    }
    fn arena(partition: &PartitionState) -> &RecordArena<Self::Kind> {
        &partition.relation_arena
    }
    fn visible(
        partition: &PartitionState,
        slot: usize,
        kind: Option<KindId>,
        version: VersionId,
        current: VersionId,
    ) -> bool {
        match kind {
            None => visibility::relation_visible_in_partition_at_version(partition, slot, version),
            Some(kind) => visibility::relation_slot_matches_kind_at_version(
                partition, slot, kind, version, current,
            ),
        }
    }
    fn quote_head(
        registry: &RelationalSchemaRegistry,
        head: RelationalRegionHeadView<'_>,
        slot: usize,
    ) -> Result<u64, Stop> {
        record_quote::relation_head(registry, head, slot)
    }
    fn quote_history(
        registry: &RelationalSchemaRegistry,
        partition: &PartitionState,
        slot: usize,
        visibility: KindScanVisibility,
    ) -> Result<u64, Stop> {
        record_quote::relation(registry, partition, slot, visibility)
    }
    fn read_exact_head(
        registry: &RelationalSchemaRegistry,
        head: RelationalRegionHeadView<'_>,
        id: crate::identity::data::RelationId,
    ) -> Option<Self::Record> {
        head.relation_record(registry, id)
    }
    fn read_history(
        registry: &RelationalSchemaRegistry,
        partition: &PartitionState,
        id: PartitionId,
        slot: usize,
        version: VersionId,
    ) -> Option<Self::Record> {
        materialization::materialize_authoritative_relation_record_at_version(
            registry, partition, id, slot, version,
        )
    }
    fn record_bytes(record: &Self::Record) -> u64 {
        read_footprint::relation(record)
    }
    fn count_materialization(instrumentation: &RuntimeInstrumentation) {
        instrumentation
            .count(|counters| counters.visible_authoritative_relation_records_materialized += 1);
    }
    fn count_historical_slot(instrumentation: &RuntimeInstrumentation) {
        instrumentation.count(|counters| counters.visibility_relation_slot_scans += 1);
    }
    fn order(records: &mut [Self::Record]) {
        super::truth_record_access::sort_authoritative_relation_records(records);
    }
}
