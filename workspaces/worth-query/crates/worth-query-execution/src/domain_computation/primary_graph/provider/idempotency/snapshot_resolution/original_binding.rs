//! One grammar for the original keyed creation row, published or unpublished.
use super::original_receipt::RecordedOriginalBinding;
use super::*;
use worth_relational::facade::{
    durability::ReadmittedRelationalCreatedEntity, transactions::AspectFieldLocator,
};
type Denial = WorthQueryProviderIdempotencyResolutionDenial;
pub(in crate::domain_computation::primary_graph::provider) fn read(
    row: &ReadmittedRelationalCreatedEntity<'_>,
    layout: &WorthQueryProviderIdempotencyLayout,
) -> Result<RecordedOriginalBinding, Denial> {
    if row.kind_id() != layout.entity_kind {
        return Err(Denial::Unavailable);
    }
    let value = |locator: &AspectFieldLocator| {
        row.field(
            locator.aspect().aspect_key(),
            locator.field_path().fields().first()?,
        )
    };
    let text = |locator: &AspectFieldLocator| {
        value(locator)
            .and_then(AspectValue::raw_text)
            .map(str::to_owned)
            .ok_or(Denial::Unavailable)
    };
    let number = |locator: &AspectFieldLocator| match value(locator) {
        Some(AspectValue::UInt64(value)) => Ok(*value),
        _ => Err(Denial::Unavailable),
    };
    if number(&layout.association_locator)?
        != super::super::recorded_origin::ENCLOSING_NATIVE_ASSOCIATION
    {
        return Err(Denial::Unavailable);
    }
    Ok(RecordedOriginalBinding {
        key: decode_key(&text(&layout.key_locator)?)?,
        intent: text(&layout.intent_locator)?,
        affinity: text(&layout.affinity_locator)?,
        outcome: WorthQueryApplicationCommitOutcomeIdentity::restore(number(
            &layout.outcome_identity_locator,
        )?)
        .ok_or(Denial::Unavailable)?,
        emitted: usize::try_from(number(&layout.emitted_effect_count_locator)?)
            .map_err(|_| Denial::Unavailable)?,
        settlement: worth_foundational::facade::DurableQueryOutputSettlement::from_recorded_code(
            number(&layout.output_settlement_locator)?,
        ),
    })
}
fn decode_key(text: &str) -> Result<[u8; 32], Denial> {
    if text.len() != 64 {
        return Err(Denial::Unavailable);
    }
    let mut key = [0; 32];
    for (target, pair) in key.iter_mut().zip(text.as_bytes().as_chunks::<2>().0) {
        let digit = |byte| match byte {
            b'0'..=b'9' => Ok(byte - b'0'),
            b'a'..=b'f' => Ok(byte - b'a' + 10),
            _ => Err(Denial::Unavailable),
        };
        *target = digit(pair[0])? * 16 + digit(pair[1])?;
    }
    Ok(key)
}
