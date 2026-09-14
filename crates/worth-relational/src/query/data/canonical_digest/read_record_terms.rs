use crate::identity::data::{LineageId, VersionId};
use crate::schema::data::KindResolution;
use crate::storage::data::{EntityReadRecord, RecordLifecycleState, RelationReadRecord};
use worth_foundational::facade::{
    AuthoritativeRecordAspectState, ContractValidatedAspectValueView, StructAspectValue,
};

use super::primitive_terms::{
    encode_entity_id, encode_field_key, encode_kind_id, encode_length_prefixed_aspect_value,
    encode_relation_id, encode_schema_id, encode_schema_version_id, encode_string, encode_u64,
    encode_usize, encode_version_id,
};

pub(super) fn encode_authoritative_entity_records(
    bytes: &mut Vec<u8>,
    records: &[EntityReadRecord],
) {
    encode_usize(bytes, records.len());
    for record in records {
        encode_entity_read_record(bytes, record);
    }
}

pub(super) fn encode_authoritative_relation_records(
    bytes: &mut Vec<u8>,
    records: &[RelationReadRecord],
) {
    encode_usize(bytes, records.len());
    for record in records {
        encode_relation_read_record(bytes, record);
    }
}

pub(super) fn encode_entity_read_record(bytes: &mut Vec<u8>, record: &EntityReadRecord) {
    encode_entity_id(bytes, record.entity_id);
    encode_optional_lineage_id(bytes, record.lineage_id);
    encode_kind_resolution(bytes, &record.kind);
    encode_record_lifecycle_state(bytes, record.lifecycle);
    encode_version_id(bytes, record.created_at_version);
    encode_optional_version_id(bytes, record.retired_at_version);
    encode_optional_authoritative_aspect_state(bytes, record.authoritative_aspect_state.as_ref());
}

pub(super) fn encode_relation_read_record(bytes: &mut Vec<u8>, record: &RelationReadRecord) {
    encode_relation_id(bytes, record.relation_id);
    encode_kind_resolution(bytes, &record.kind);
    encode_record_lifecycle_state(bytes, record.lifecycle);
    encode_version_id(bytes, record.created_at_version);
    encode_optional_version_id(bytes, record.retired_at_version);
    encode_entity_id(bytes, record.source);
    encode_entity_id(bytes, record.target);
    encode_optional_authoritative_aspect_state(bytes, record.authoritative_aspect_state.as_ref());
}

fn encode_kind_resolution(bytes: &mut Vec<u8>, kind: &KindResolution) {
    encode_kind_id(bytes, kind.kind_id);
    encode_string(bytes, &kind.kind_name);
    encode_schema_id(bytes, &kind.schema_id);
    encode_schema_version_id(bytes, kind.schema_version_id);
}

fn encode_optional_authoritative_aspect_state(
    bytes: &mut Vec<u8>,
    state: Option<&AuthoritativeRecordAspectState>,
) {
    match state {
        Some(state) => {
            bytes.push(1);
            encode_authoritative_aspect_state(bytes, state);
        }
        None => bytes.push(0),
    }
}

fn encode_authoritative_aspect_state(bytes: &mut Vec<u8>, state: &AuthoritativeRecordAspectState) {
    encode_usize(bytes, state.aspects().len());
    for (aspect_key, value) in state.aspects().entries() {
        encode_string(bytes, aspect_key.as_str());
        encode_u64(bytes, value.contract_revision().0);
        match value.view() {
            ContractValidatedAspectValueView::Scalar(value) => {
                bytes.push(0);
                encode_length_prefixed_aspect_value(bytes, value);
            }
            ContractValidatedAspectValueView::Struct(value) => {
                bytes.push(1);
                encode_struct_aspect_value(bytes, value);
            }
        }
    }
}

fn encode_struct_aspect_value(bytes: &mut Vec<u8>, value: &StructAspectValue) {
    encode_usize(bytes, value.fields().count());
    for (field, field_value) in value.fields() {
        encode_field_key(bytes, field);
        encode_length_prefixed_aspect_value(bytes, field_value);
    }
}

fn encode_optional_lineage_id(bytes: &mut Vec<u8>, lineage_id: Option<LineageId>) {
    match lineage_id {
        Some(lineage_id) => {
            bytes.push(1);
            encode_u64(bytes, lineage_id.0);
        }
        None => bytes.push(0),
    }
}

fn encode_optional_version_id(bytes: &mut Vec<u8>, version_id: Option<VersionId>) {
    match version_id {
        Some(version_id) => {
            bytes.push(1);
            encode_version_id(bytes, version_id);
        }
        None => bytes.push(0),
    }
}

fn encode_record_lifecycle_state(bytes: &mut Vec<u8>, state: RecordLifecycleState) {
    bytes.push(match state {
        RecordLifecycleState::Live => 0,
        RecordLifecycleState::DeletedRetained => 1,
        RecordLifecycleState::RetainedDanglingForAudit => 2,
        RecordLifecycleState::PinnedBySnapshot => 3,
        RecordLifecycleState::PinnedByBranch => 4,
        RecordLifecycleState::PinnedByReplayRetention => 5,
        RecordLifecycleState::Reclaimable => 6,
        RecordLifecycleState::Reusable => 7,
        RecordLifecycleState::MaterializationUnavailable => 8,
    });
}
