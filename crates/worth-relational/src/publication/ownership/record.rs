//! The single payload grammar of the Native-private current ownership records.
use crate::{
    history::data::{CanonicalRequiredOutputOrigin, CanonicalRequiredOutputRelation, CommitId},
    identity::data::EntityId,
    mvcc::RequiredOutputPublicationMeaning,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, AuthoritativeRecordAspectState, CanonicalFieldPath,
    ContractValidatedAspectValueView, FieldKey, LocatorAuthority,
};

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Catalog {
    pub(super) scope: EntityId,
    pub(super) routes: Vec<Route>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Route {
    pub(super) meaning: RequiredOutputPublicationMeaning,
    pub(super) publication: CommitId,
    pub(super) digest: [u8; 32],
}
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Publication {
    pub(super) origin: Arc<CanonicalRequiredOutputOrigin>,
    pub(super) ordinal: usize,
}
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Edge {
    pub(super) publication: CommitId,
    pub(super) edge: CanonicalRequiredOutputRelation,
}

pub(super) fn fields<'a>(
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
    let worth_foundational::facade::InternedString::Raw(payload) = payload else {
        return None;
    };
    Some((CommitId(*publication), payload.as_bytes()))
}

pub(super) fn locator(aspect: &str, field: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new(aspect).expect("fixed ownership aspect"),
        CanonicalFieldPath::single(FieldKey::new(field).expect("fixed ownership field")),
    )
}
