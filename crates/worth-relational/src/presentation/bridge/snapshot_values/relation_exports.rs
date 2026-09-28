use worth_foundational::facade::{
    AspectKey, AspectValue, ContractValidatedAspectValueView, EntityId as FoundationalEntityId,
    Generation as FoundationalGeneration, LocalSlot as FoundationalLocalSlot,
    PartitionId as FoundationalPartitionId,
};
use worth_runtime_bridge::facade::SnapshotReadValue;

use crate::facade::identity::EntityId;
use crate::facade::runtime::RelationReadRecord;

use super::lifecycle_snapshot_values::lifecycle_aspect_value;

pub(crate) fn export_relation_aspect_snapshot_value(
    record: &RelationReadRecord,
    aspect_key: &AspectKey,
) -> Option<SnapshotReadValue> {
    relation_snapshot_aspect_value(record, aspect_key)
}

fn relation_snapshot_aspect_value(
    record: &RelationReadRecord,
    aspect_key: &AspectKey,
) -> Option<SnapshotReadValue> {
    match aspect_key.as_str() {
        "source" => Some(endpoint_aspect_value(record.source).into()),
        "target" => Some(endpoint_aspect_value(record.target).into()),
        "lifecycle" => Some(lifecycle_aspect_value(record.lifecycle).into()),
        _ => authoritative_relation_aspect_value(record, aspect_key),
    }
}

fn authoritative_relation_aspect_value(
    record: &RelationReadRecord,
    aspect_key: &AspectKey,
) -> Option<SnapshotReadValue> {
    let authoritative_state = record.authoritative_aspect_state.as_ref()?;
    let aspect_entry = authoritative_state.get(aspect_key)?;
    match aspect_entry.view() {
        ContractValidatedAspectValueView::Scalar(value) => Some(value.clone().into()),
        ContractValidatedAspectValueView::Struct(value) => Some(value.clone().into()),
    }
}

fn endpoint_aspect_value(entity_id: EntityId) -> AspectValue {
    AspectValue::EntityRef(FoundationalEntityId {
        partition_id: FoundationalPartitionId(entity_id.partition_id.as_u32()),
        local_slot: FoundationalLocalSlot(entity_id.local_slot_value()),
        generation: FoundationalGeneration(entity_id.generation_value()),
    })
}
