use crate::identity::data::{KindId, LineageId};
use crate::storage::data::read_records::{EntityReadRecord, RelationReadRecord};
use crate::transactions::data::primitives::RecordRef;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub(crate) enum VisibleMergeRecordKind {
    Entity,
    Relation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VisibleMergeRecord {
    pub(crate) record_ref: RecordRef,
    pub(crate) record_kind: VisibleMergeRecordKind,
    pub(crate) kind_id: Option<KindId>,
    pub(crate) source_kind_id: Option<KindId>,
    pub(crate) target_kind_id: Option<KindId>,
    pub(crate) lineage_id: Option<LineageId>,
    pub(crate) source_lineage_id: Option<LineageId>,
    pub(crate) target_lineage_id: Option<LineageId>,
    pub(crate) source_entity: Option<EntityReadRecord>,
    pub(crate) target_entity: Option<EntityReadRecord>,
    pub(crate) source_relation: Option<RelationReadRecord>,
    pub(crate) target_relation: Option<RelationReadRecord>,
}
