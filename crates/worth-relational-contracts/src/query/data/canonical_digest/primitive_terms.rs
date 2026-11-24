use crate::aspect_wire;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::schema::data::{DescriptorSemanticsVersion, SchemaId, SchemaVersionId};
use worth_foundational::facade::{AspectFieldLocator, AspectValue, FieldKey};

pub(super) fn encode_string(bytes: &mut Vec<u8>, value: &str) {
    aspect_wire::encode_string(bytes, value);
}

pub(super) fn encode_entity_ids(bytes: &mut Vec<u8>, ids: &[EntityId]) {
    encode_usize(bytes, ids.len());
    for id in ids {
        encode_entity_id(bytes, *id);
    }
}

pub(super) fn encode_partition_ids(bytes: &mut Vec<u8>, partitions: &[PartitionId]) {
    encode_usize(bytes, partitions.len());
    for partition in partitions {
        encode_partition_id(bytes, *partition);
    }
}

pub(super) fn encode_field_key(bytes: &mut Vec<u8>, field: &FieldKey) {
    encode_string(bytes, field.as_str());
}

pub(super) fn encode_entity_id(bytes: &mut Vec<u8>, id: EntityId) {
    encode_partition_id(bytes, id.partition_id);
    encode_u64(bytes, id.local_slot.0);
    encode_u32(bytes, id.generation.0);
}

pub(super) fn encode_relation_id(bytes: &mut Vec<u8>, id: RelationId) {
    encode_partition_id(bytes, id.partition_id);
    encode_u64(bytes, id.local_slot.0);
    encode_u32(bytes, id.generation.0);
}

pub(super) fn encode_partition_id(bytes: &mut Vec<u8>, partition_id: PartitionId) {
    encode_u32(bytes, partition_id.0);
}

pub(super) fn encode_kind_id(bytes: &mut Vec<u8>, kind_id: KindId) {
    encode_u32(bytes, kind_id.0);
}

pub(super) fn encode_version_id(bytes: &mut Vec<u8>, version_id: VersionId) {
    encode_u64(bytes, version_id.0);
}

pub(super) fn encode_schema_id(bytes: &mut Vec<u8>, schema_id: &SchemaId) {
    encode_string(bytes, &schema_id.0);
}

pub(super) fn encode_schema_version_id(bytes: &mut Vec<u8>, schema_version: SchemaVersionId) {
    encode_u32(bytes, schema_version.0);
}

pub(super) fn encode_descriptor_semantics_version(
    bytes: &mut Vec<u8>,
    version: DescriptorSemanticsVersion,
) {
    encode_u32(bytes, version.0);
}

pub(super) fn encode_length_prefixed_aspect_value(bytes: &mut Vec<u8>, value: &AspectValue) {
    let value_bytes = aspect_wire::encode_aspect_value(value);
    encode_usize(bytes, value_bytes.len());
    bytes.extend_from_slice(&value_bytes);
}

pub(super) fn encode_length_prefixed_aspect_field_locator(
    bytes: &mut Vec<u8>,
    field_locator: &AspectFieldLocator,
) {
    let locator_bytes = aspect_wire::encode_aspect_field_locator(field_locator);
    encode_usize(bytes, locator_bytes.len());
    bytes.extend_from_slice(&locator_bytes);
}

pub(super) fn encode_optional_kind_id(bytes: &mut Vec<u8>, kind_id: Option<KindId>) {
    match kind_id {
        Some(kind_id) => {
            bytes.push(1);
            encode_kind_id(bytes, kind_id);
        }
        None => bytes.push(0),
    }
}

pub(super) fn encode_optional_u32(bytes: &mut Vec<u8>, value: Option<u32>) {
    match value {
        Some(value) => {
            bytes.push(1);
            encode_u32(bytes, value);
        }
        None => bytes.push(0),
    }
}

pub(super) fn encode_u32(bytes: &mut Vec<u8>, value: u32) {
    aspect_wire::encode_u32(bytes, value);
}

pub(super) fn encode_usize(bytes: &mut Vec<u8>, value: usize) {
    encode_u64(bytes, value as u64);
}

pub(super) fn encode_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub(super) fn encode_u128(bytes: &mut Vec<u8>, value: u128) {
    bytes.extend_from_slice(&value.to_be_bytes());
}
