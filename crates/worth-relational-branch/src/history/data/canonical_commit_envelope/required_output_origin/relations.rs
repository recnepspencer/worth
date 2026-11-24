//! Canonical relation membership and before-write coordinates.
use crate::identity::data::{EntityId, KindId, RelationId};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CanonicalRequiredOutputRelation {
    pub id: RelationId,
    pub kind: KindId,
    pub source: EntityId,
    pub target: EntityId,
    pub birth: crate::history::data::CommitId,
    pub before_write: Option<crate::identity::data::VersionId>,
    pub after_write: crate::identity::data::VersionId,
}
