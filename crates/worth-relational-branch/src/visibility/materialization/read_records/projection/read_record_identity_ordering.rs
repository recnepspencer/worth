use crate::storage::data::{EntityReadRecord, RelationReadRecord};

pub(super) fn authoritative_entity_records_are_identity_ordered(
    records: &[EntityReadRecord],
) -> bool {
    records
        .windows(2)
        .all(|window| window[0].entity_id <= window[1].entity_id)
}

pub(super) fn authoritative_relation_records_are_identity_ordered(
    records: &[RelationReadRecord],
) -> bool {
    records.windows(2).all(|window| {
        relation_identity_order_key(&window[0]) <= relation_identity_order_key(&window[1])
    })
}

pub(super) fn relation_identity_order_key(
    record: &RelationReadRecord,
) -> (u32, u64, u32, u64, u32, u64) {
    (
        record.source.partition_value(),
        record.source.local_slot_value(),
        record.target.partition_value(),
        record.target.local_slot_value(),
        record.relation_id.partition_value(),
        record.relation_id.local_slot_value(),
    )
}

#[cfg(test)]
mod tests {
    use super::relation_identity_order_key;
    use crate::identity::data::{EntityId, PartitionId, RelationId};
    use crate::identity::data::{KindId, VersionId};
    use crate::schema::data::{KindResolution, SchemaId, SchemaVersionId};
    use crate::storage::data::{RecordLifecycleState, RelationReadRecord};

    #[test]
    fn relation_identity_order_key_uses_named_record_identity_accessors() {
        let record = RelationReadRecord {
            relation_id: RelationId::new(PartitionId::new(9), 5, 2),
            kind: KindResolution {
                kind_id: KindId::new(6),
                kind_name: "connected_to".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
            },
            lifecycle: RecordLifecycleState::Live,
            created_at_version: VersionId::new(2),
            retired_at_version: None,
            source: EntityId::new(PartitionId::new(3), 11, 7),
            target: EntityId::new(PartitionId::new(4), 12, 8),
            authoritative_aspect_state: None,
        };

        assert_eq!(relation_identity_order_key(&record), (3, 11, 4, 12, 9, 5));
    }
}
