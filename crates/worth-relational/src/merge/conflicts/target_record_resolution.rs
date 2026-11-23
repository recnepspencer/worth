use crate::merge::data::{VisibleMergeRecord, VisibleMergeRecordKind};
use crate::transactions::data::RecordRef;
use crate::visibility::snapshot_read_view::RelationalReadView;

pub(super) fn visible_target_record(
    target_view: &RelationalReadView,
    target_record: &RecordRef,
) -> Option<VisibleMergeRecord> {
    match target_record {
        RecordRef::Entity(entity_id) => {
            let entity = target_view.get_entity(*entity_id).cloned()?;
            Some(VisibleMergeRecord {
                record_ref: RecordRef::Entity(*entity_id),
                record_kind: VisibleMergeRecordKind::Entity,
                kind_id: Some(entity.kind.kind_id),
                source_kind_id: None,
                target_kind_id: Some(entity.kind.kind_id),
                lineage_id: entity.lineage_id,
                source_lineage_id: None,
                target_lineage_id: entity.lineage_id,
                source_entity: None,
                target_entity: Some(entity),
                source_relation: None,
                target_relation: None,
            })
        }
        RecordRef::Relation(relation_id) => {
            let relation = target_view.get_relation(*relation_id).cloned()?;
            Some(VisibleMergeRecord {
                record_ref: RecordRef::Relation(*relation_id),
                record_kind: VisibleMergeRecordKind::Relation,
                kind_id: Some(relation.kind.kind_id),
                source_kind_id: None,
                target_kind_id: Some(relation.kind.kind_id),
                lineage_id: None,
                source_lineage_id: None,
                target_lineage_id: None,
                source_entity: None,
                target_entity: None,
                source_relation: None,
                target_relation: Some(relation),
            })
        }
    }
}
