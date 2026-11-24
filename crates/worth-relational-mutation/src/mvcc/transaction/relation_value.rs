//! The selected relation and effective staged endpoints.
use crate::contracts::storage::data::read_records::RelationReadRecord;
use crate::contracts::transactions::data::primitives::EntityReference;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalTransactionRelationValue {
    pub relation_id: crate::contracts::identity::data::record_ids::RelationId,
    pub kind: crate::contracts::schema::data::KindResolution,
    pub lifecycle: crate::contracts::storage::data::read_records::RecordLifecycleState,
    pub created_at_version: crate::contracts::identity::data::primitives::VersionId,
    pub retired_at_version: Option<crate::contracts::identity::data::primitives::VersionId>,
    pub source: EntityReference,
    pub target: EntityReference,
    pub authoritative_aspect_state:
        Option<worth_foundational::facade::AuthoritativeRecordAspectState>,
}

impl From<RelationReadRecord> for RelationalTransactionRelationValue {
    fn from(record: RelationReadRecord) -> Self {
        Self {
            relation_id: record.relation_id,
            kind: record.kind,
            lifecycle: record.lifecycle,
            created_at_version: record.created_at_version,
            retired_at_version: record.retired_at_version,
            source: EntityReference::Existing(record.source),
            target: EntityReference::Existing(record.target),
            authoritative_aspect_state: record.authoritative_aspect_state,
        }
    }
}
