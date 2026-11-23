//! The selected relation and effective staged endpoints.
use crate::storage::data::RelationReadRecord;
use crate::transactions::data::primitives::EntityReference;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalTransactionRelationValue {
    pub relation_id: crate::identity::data::RelationId,
    pub kind: crate::schema::data::KindResolution,
    pub lifecycle: crate::storage::data::RecordLifecycleState,
    pub created_at_version: crate::identity::data::VersionId,
    pub retired_at_version: Option<crate::identity::data::VersionId>,
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
