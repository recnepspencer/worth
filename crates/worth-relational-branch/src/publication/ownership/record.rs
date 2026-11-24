//! The single payload grammar of the Native-private current ownership records.
use crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning;
use crate::{
    history::data::{CanonicalRequiredOutputOrigin, CanonicalRequiredOutputRelation, CommitId},
    identity::data::EntityId,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_foundational::facade::{
    AspectKey, AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView,
    FieldKey,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Catalog {
    pub scope: EntityId,
    pub routes: Vec<Route>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub meaning: RequiredOutputPublicationMeaning,
    pub publication: CommitId,
    pub digest: [u8; 32],
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Publication {
    pub origin: Arc<CanonicalRequiredOutputOrigin>,
    pub ordinal: usize,
    /// Legacy rows implicitly bind claims to their original publication. A
    /// successor carries that generation explicitly while replacing its origin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custody: Option<CommitId>,
}
impl Publication {
    pub fn custody(&self) -> CommitId {
        self.custody.unwrap_or(self.origin.commit.commit_id)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Edge {
    pub publication: CommitId,
    pub edge: CanonicalRequiredOutputRelation,
}
/// Custody binds identity, birth and endpoints. Write continuity belongs to the
/// latest canonical origin. The stored witness retains its original receipt
/// bytes for codec compatibility; those historical write positions do not
/// require rewriting an unchanged custody claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationClaim {
    pub id: crate::identity::data::RelationId,
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
    pub fn matches(&self, edge: &CanonicalRequiredOutputRelation) -> bool {
        self == &Self::from(edge)
    }
}

pub fn fields<'a>(
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
    let payload = payload.raw_text()?;
    Some((CommitId(*publication), payload.as_bytes()))
}

/// The catalog and independent presence both describe this immutable row.
pub fn route_matches(row: &Publication, digest: &[u8; 32], route: &Route) -> bool {
    row.origin.meaning == route.meaning
        && row.origin.commit.commit_id == route.publication
        && digest == &route.digest
}
