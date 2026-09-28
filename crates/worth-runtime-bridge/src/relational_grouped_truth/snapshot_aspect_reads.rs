use crate::snapshot::{SnapshotReadRecord, SnapshotReadValue};
use worth_foundational::facade::AspectValue;

use super::grouped_projection::RelationalGroupedTruthError;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SnapshotAspectReadValue {
    value: SnapshotReadValue,
}

impl SnapshotAspectReadValue {
    pub(super) fn value(&self) -> &SnapshotReadValue {
        &self.value
    }
}

pub(super) fn decode_snapshot_aspect_read_value(
    record: &SnapshotReadRecord,
) -> Result<Option<SnapshotAspectReadValue>, RelationalGroupedTruthError> {
    Ok(record
        .read_value_posture()
        .cloned()
        .map(|value| SnapshotAspectReadValue { value }))
}

pub fn encode_snapshot_aspect_read_value(value: &AspectValue) -> AspectValue {
    value.clone()
}
