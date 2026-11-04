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
}
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Edge {
    pub(crate) publication: CommitId,
    pub(crate) edge: CanonicalRequiredOutputRelation,
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
    let worth_foundational::facade::InternedString::Raw(payload) = payload else {
        return None;
    };
    Some((CommitId(*publication), payload.as_bytes()))
}
