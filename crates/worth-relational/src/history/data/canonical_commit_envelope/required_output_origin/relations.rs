//! Canonical relation membership and before-write coordinates.
use crate::identity::data::{EntityId, KindId, RelationId};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CanonicalRequiredOutputRelation {
    pub(crate) id: RelationId,
    pub(crate) kind: KindId,
    pub(crate) source: EntityId,
    pub(crate) target: EntityId,
    pub(crate) birth: crate::history::data::CommitId,
    pub(crate) before_write: Option<crate::identity::data::VersionId>,
    pub(crate) after_write: crate::identity::data::VersionId,
}
