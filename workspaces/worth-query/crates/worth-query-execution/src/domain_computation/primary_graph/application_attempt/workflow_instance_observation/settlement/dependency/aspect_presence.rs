//! Reconstruct the exact aspect predicate, never a field-presence default.
use super::denial;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
};
use worth_foundational::facade::AspectKey;
use worth_relational::facade::identity::{EntityId, KindId};
#[cfg(test)]
mod tests;

pub(super) fn decode(
    entity_id: EntityId,
    aspect: Option<String>,
    native_kind: Option<u64>,
    present: bool,
) -> Result<WorthQueryApplicationObservedFact, WorthQueryApplicationAttemptDenial> {
    let aspect = aspect
        .and_then(AspectKey::new)
        .ok_or_else(|| denial("workflow evidence dependency aspect presence key is invalid"))?;
    let kind = native_kind
        .and_then(|kind| u32::try_from(kind).ok())
        .map(KindId::new)
        .ok_or_else(|| denial("workflow evidence dependency aspect presence kind is invalid"))?;
    Ok(WorthQueryApplicationObservedFact::AspectPresence {
        entity_id,
        kind,
        aspect,
        present,
    })
}
