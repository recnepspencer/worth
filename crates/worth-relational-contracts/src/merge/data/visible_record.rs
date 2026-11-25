use crate::identity::data::{KindId, LineageId};
use crate::storage::data::read_records::{EntityReadRecord, RelationReadRecord};
use crate::transactions::data::primitives::RecordRef;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum VisibleMergeRecordKind {
    Entity,
    Relation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleMergeRecord {
    pub record_ref: RecordRef,
    pub record_kind: VisibleMergeRecordKind,
    pub kind_id: Option<KindId>,
    pub source_kind_id: Option<KindId>,
    pub target_kind_id: Option<KindId>,
    pub lineage_id: Option<LineageId>,
    pub source_lineage_id: Option<LineageId>,
    pub target_lineage_id: Option<LineageId>,
    pub source_entity: Option<EntityReadRecord>,
    pub target_entity: Option<EntityReadRecord>,
    pub source_relation: Option<RelationReadRecord>,
    pub target_relation: Option<RelationReadRecord>,
}
