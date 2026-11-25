//! Shared wire & inspection helpers for D14 derived-cache discard tests.
use std::collections::BTreeMap;
use worth_relational::facade::indexes::{
    DerivedIndexDefinition, DerivedIndexEntries, DerivedIndexId, DerivedIndexKind,
};
use worth_store_application::ApplicationHomeCheckpointInspection;

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct ChildSummary {
    pub(super) discard_generation: u64,
    pub(super) commit_id: u64,
    pub(super) index_id: u64,
    pub(super) pre_discard_member_count: usize,
    pub(super) index_entries_json: Vec<u8>,
    pub(super) record_bytes: Vec<u8>,
}

pub(super) fn cadence_by_name(
    inspection: &ApplicationHomeCheckpointInspection,
) -> BTreeMap<&'static str, (u64, u64)> {
    inspection
        .journals
        .iter()
        .map(|journal| (journal.name, (journal.covered, journal.head)))
        .collect()
}

pub(super) fn entity_field_entries_snapshot(
    entries: &DerivedIndexEntries,
) -> Vec<(serde_json::Value, Vec<serde_json::Value>)> {
    match entries {
        DerivedIndexEntries::EntityField(map) => map
            .iter()
            .map(|(key, rows)| {
                (
                    serde_json::to_value(key).unwrap(),
                    rows.iter()
                        .map(|entity| serde_json::to_value(entity).unwrap())
                        .collect(),
                )
            })
            .collect(),
        other => panic!("expected EntityField derived index entries, got {other:?}"),
    }
}

pub(super) fn entity_name_index() -> DerivedIndexDefinition {
    DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "discard-derived-name-index".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: worth_relational::facade::transactions::planned_single_field_locator(
                crate::batches::aspect_key("name"),
                crate::batches::field_key("name"),
            ),
        },
        branch_scoped: true,
    }
}
