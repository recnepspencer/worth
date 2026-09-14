use sha2::{Digest, Sha256};
use worth_foundational::facade::{
    AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView,
    StructAspectValue,
};
use worth_harness::facade::{BinaryValue, SnapshotPayload};

use crate::facade::harness::RelationalHarnessError;
use crate::identity::data::{EntityId, RelationId};
use crate::storage::data::{EntityReadRecord, RecordLifecycleState, RelationReadRecord};

const SNAPSHOT_MEDIA_TYPE: &str =
    "application/vnd.worth.relational.harness.aspect-snapshot.v1+octet-stream";
const SNAPSHOT_MAGIC: &[u8] = b"worth.relational.harness.aspect-snapshot.v1";
const RECORD_KIND_ENTITY: u8 = 1;
const RECORD_KIND_RELATION: u8 = 2;
const ASPECT_VALUE_SCALAR: u8 = 1;
const ASPECT_VALUE_STRUCT: u8 = 2;

pub(super) fn entity_aspect_snapshot_binary(
    record: &EntityReadRecord,
) -> Result<SnapshotPayload, RelationalHarnessError> {
    let mut encoder = AspectSnapshotBinaryEncoder::new(RECORD_KIND_ENTITY);
    encoder.entity_identity(record.entity_id);
    encoder.u64(u64::from(record.kind.kind_id.0));
    encoder.lifecycle(record.lifecycle);
    encoder.u64(record.created_at_version.0);
    encoder.optional_u64(record.retired_at_version.map(|version| version.0));
    encoder.authoritative_aspect_state(record.authoritative_aspect_state.as_ref())?;
    Ok(harness_snapshot_binary(encoder.finish()))
}

pub(super) fn relation_aspect_snapshot_binary(
    record: &RelationReadRecord,
) -> Result<SnapshotPayload, RelationalHarnessError> {
    let mut encoder = AspectSnapshotBinaryEncoder::new(RECORD_KIND_RELATION);
    encoder.relation_identity(record.relation_id);
    encoder.u64(u64::from(record.kind.kind_id.0));
    encoder.entity_identity(record.source);
    encoder.entity_identity(record.target);
    encoder.lifecycle(record.lifecycle);
    encoder.u64(record.created_at_version.0);
    encoder.optional_u64(record.retired_at_version.map(|version| version.0));
    encoder.authoritative_aspect_state(record.authoritative_aspect_state.as_ref())?;
    Ok(harness_snapshot_binary(encoder.finish()))
}

fn harness_snapshot_binary(bytes: Vec<u8>) -> SnapshotPayload {
    SnapshotPayload::Binary(BinaryValue {
        media_type: SNAPSHOT_MEDIA_TYPE.to_string(),
        content_hash: Some(sha256_hex(&bytes)),
        size_bytes: Some(bytes.len() as u64),
        bytes,
    })
}

struct AspectSnapshotBinaryEncoder {
    bytes: Vec<u8>,
}

impl AspectSnapshotBinaryEncoder {
    fn new(record_kind_tag: u8) -> Self {
        let mut encoder = Self { bytes: Vec::new() };
        encoder.raw_bytes(SNAPSHOT_MAGIC);
        encoder.u8(record_kind_tag);
        encoder
    }

    fn authoritative_aspect_state(
        &mut self,
        state: Option<&AuthoritativeRecordAspectState>,
    ) -> Result<(), RelationalHarnessError> {
        let Some(state) = state else {
            self.u32(0);
            return Ok(());
        };
        let entries = state.aspects().entries().collect::<Vec<_>>();
        self.u32(entries.len() as u32);
        for (aspect_key, value) in entries {
            self.string(aspect_key.as_str());
            self.validated_value(value.view())?;
        }
        Ok(())
    }

    fn validated_value(
        &mut self,
        value: ContractValidatedAspectValueView<'_>,
    ) -> Result<(), RelationalHarnessError> {
        match value {
            ContractValidatedAspectValueView::Scalar(scalar) => {
                self.u8(ASPECT_VALUE_SCALAR);
                self.aspect_value(scalar)
            }
            ContractValidatedAspectValueView::Struct(struct_value) => {
                self.u8(ASPECT_VALUE_STRUCT);
                self.struct_aspect_value(struct_value)
            }
        }
    }

    fn struct_aspect_value(
        &mut self,
        value: &StructAspectValue,
    ) -> Result<(), RelationalHarnessError> {
        let fields = value.fields().collect::<Vec<_>>();
        self.u32(fields.len() as u32);
        for (field_key, scalar) in fields {
            self.string(field_key.as_str());
            self.aspect_value(scalar)?;
        }
        Ok(())
    }

    fn aspect_value(&mut self, value: &AspectValue) -> Result<(), RelationalHarnessError> {
        let canonical_bytes = crate::aspect_wire::encode_aspect_value(value);
        self.bytes(&canonical_bytes);
        Ok(())
    }

    fn entity_identity(&mut self, entity_id: EntityId) {
        self.u64(u64::from(entity_id.partition_id.0));
        self.u64(entity_id.local_slot.0);
        self.u64(u64::from(entity_id.generation.0));
    }

    fn relation_identity(&mut self, relation_id: RelationId) {
        self.u64(u64::from(relation_id.partition_id.0));
        self.u64(relation_id.local_slot.0);
        self.u64(u64::from(relation_id.generation.0));
    }

    fn lifecycle(&mut self, lifecycle: RecordLifecycleState) {
        self.u8(match lifecycle {
            RecordLifecycleState::Live => 1,
            RecordLifecycleState::DeletedRetained => 2,
            RecordLifecycleState::RetainedDanglingForAudit => 3,
            RecordLifecycleState::PinnedBySnapshot => 4,
            RecordLifecycleState::PinnedByBranch => 5,
            RecordLifecycleState::PinnedByReplayRetention => 6,
            RecordLifecycleState::Reclaimable => 7,
            RecordLifecycleState::Reusable => 8,
            RecordLifecycleState::MaterializationUnavailable => 9,
        });
    }

    fn optional_u64(&mut self, value: Option<u64>) {
        match value {
            Some(value) => {
                self.u8(1);
                self.u64(value);
            }
            None => self.u8(0),
        }
    }

    fn string(&mut self, value: &str) {
        self.bytes(value.as_bytes());
    }

    fn bytes(&mut self, value: &[u8]) {
        self.u32(value.len() as u32);
        self.bytes.extend_from_slice(value);
    }

    fn raw_bytes(&mut self, value: &[u8]) {
        self.bytes.extend_from_slice(value);
    }

    fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{digest:x}")
}
