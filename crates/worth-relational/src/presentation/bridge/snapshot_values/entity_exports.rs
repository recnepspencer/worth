use worth_foundational::facade::{AspectKey, ContractValidatedAspectValueView};
use worth_runtime_bridge::facade::SnapshotReadValue;

use crate::facade::runtime::EntityReadRecord;

use super::lifecycle_snapshot_values::lifecycle_aspect_value;

pub(crate) fn export_entity_aspect_snapshot_value(
    record: &EntityReadRecord,
    aspect_key: &AspectKey,
) -> Option<SnapshotReadValue> {
    Some(if aspect_key.as_str() == "lifecycle" {
        lifecycle_aspect_value(record.lifecycle).into()
    } else {
        authoritative_entity_aspect_value(record, aspect_key)?
    })
}

fn authoritative_entity_aspect_value(
    record: &EntityReadRecord,
    aspect_key: &AspectKey,
) -> Option<SnapshotReadValue> {
    let authoritative_state = record.authoritative_aspect_state.as_ref()?;
    let aspect_entry = authoritative_state.get(aspect_key)?;
    match aspect_entry.view() {
        ContractValidatedAspectValueView::Scalar(value) => Some(value.clone().into()),
        ContractValidatedAspectValueView::Struct(value) => Some(value.clone().into()),
    }
}
