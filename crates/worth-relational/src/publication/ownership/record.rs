//! The single payload grammar of the Native-private current ownership records.
use crate::{
    history::data::{CanonicalRequiredOutputOrigin, CanonicalRequiredOutputRelation, CommitId},
    identity::data::EntityId,
    mvcc::RequiredOutputPublicationMeaning,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_foundational::facade::{
    AspectKey, AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView,
    FieldKey,
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Catalog {
    pub(crate) scope: EntityId,
    pub(crate) routes: Vec<Route>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Route {
    pub(crate) meaning: RequiredOutputPublicationMeaning,
    pub(crate) publication: CommitId,
    pub(crate) digest: [u8; 32],
}
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Publication {
    pub(crate) origin: Arc<CanonicalRequiredOutputOrigin>,
    pub(crate) ordinal: usize,
    /// Legacy rows implicitly bind claims to their original publication. A
    /// successor carries that generation explicitly while replacing its origin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) custody: Option<CommitId>,
}
impl Publication {
    pub(crate) fn custody(&self) -> CommitId {
        self.custody.unwrap_or(self.origin.commit.commit_id)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Edge {
    pub(crate) publication: CommitId,
    pub(crate) edge: CanonicalRequiredOutputRelation,
}
/// Custody binds identity, birth and endpoints. Write continuity belongs to the
/// latest canonical origin. The stored witness retains its original receipt
/// bytes for codec compatibility; those historical write positions do not
/// require rewriting an unchanged custody claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RelationClaim {
    pub(crate) id: crate::identity::data::RelationId,
    kind: crate::identity::data::KindId,
    source: EntityId,
    target: EntityId,
    birth: CommitId,
}
impl From<&CanonicalRequiredOutputRelation> for RelationClaim {
    fn from(edge: &CanonicalRequiredOutputRelation) -> Self {
        Self {
            id: edge.id,
            kind: edge.kind,
            source: edge.source,
            target: edge.target,
            birth: edge.birth,
        }
    }
}
impl RelationClaim {
    pub(crate) fn matches(&self, edge: &CanonicalRequiredOutputRelation) -> bool {
        self == &Self::from(edge)
    }
}

pub(crate) fn fields<'a>(
    state: &'a AuthoritativeRecordAspectState,
    aspect: &str,
) -> Option<(CommitId, &'a [u8])> {
    let aspect = state.get(&AspectKey::new(aspect)?)?;
    let ContractValidatedAspectValueView::Struct(fields) = aspect.view() else {
        return None;
    };
    let AspectValue::UInt64(publication) = fields.get(&FieldKey::new("publication")?)? else {
        return None;
    };
    let AspectValue::String(payload) = fields.get(&FieldKey::new("payload")?)? else {
        return None;
    };
    let Some(payload) = (payload).raw_text() else {
        return None;
    };
    Some((CommitId(*publication), payload.as_bytes()))
}

/// The catalog and independent presence both describe this immutable row.
pub(crate) fn route_matches(row: &Publication, digest: &[u8; 32], route: &Route) -> bool {
    row.origin.meaning == route.meaning
        && row.origin.commit.commit_id == route.publication
        && digest == &route.digest
}
