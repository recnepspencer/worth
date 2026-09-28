use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::RelationalRuntime,
    snapshots::SnapshotHandle,
};

use super::{denial, WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact};

pub(super) fn observed_text(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<String, WorthQueryApplicationAttemptDenial> {
    let value = super::super::observe_field_value(runtime, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow assessment evidence field is unavailable"))?;
    let retained = value.clone();
    let AspectValue::String(value) = &value else {
        return Err(denial(
            "workflow assessment evidence field has the wrong type",
        ));
    };
    let text = match value {
        InternedString::Raw(value) => value.clone(),
        InternedString::Symbol(_) => {
            return Err(denial(
                "workflow assessment evidence text is not materialized",
            ))
        }
    };
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value: retained,
    });
    Ok(text)
}

pub(super) fn observed_optional_text(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<String>, WorthQueryApplicationAttemptDenial> {
    if super::super::observe_field_value(runtime, snapshot, entity, kind, locator).is_none() {
        facts.push(WorthQueryApplicationObservedFact::AbsentField {
            entity_id: entity,
            kind,
            locator: locator.clone(),
        });
        return Ok(None);
    }
    observed_text(runtime, snapshot, entity, kind, locator, facts).map(Some)
}

pub(super) fn observed_bool(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<bool, WorthQueryApplicationAttemptDenial> {
    let value = super::super::observe_field_value(runtime, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow assessment evidence field is unavailable"))?;
    let AspectValue::Bool(posture) = value else {
        return Err(denial(
            "workflow assessment evidence field has the wrong type",
        ));
    };
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value: AspectValue::Bool(posture),
    });
    Ok(posture)
}

pub(super) fn observed_u64(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<u64, WorthQueryApplicationAttemptDenial> {
    let value = super::super::observe_field_value(runtime, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow assessment evidence field is unavailable"))?;
    let AspectValue::UInt64(number) = value else {
        return Err(denial(
            "workflow assessment evidence field has the wrong type",
        ));
    };
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value: AspectValue::UInt64(number),
    });
    Ok(number)
}

pub(super) fn optional_identity(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<[u8; 32]>, WorthQueryApplicationAttemptDenial> {
    let Some(value) = super::super::observe_field_value(runtime, snapshot, entity, kind, locator)
    else {
        return Ok(None);
    };
    let AspectValue::String(InternedString::Raw(text)) = &value else {
        return Err(denial(
            "workflow operation receipt identity has the wrong type",
        ));
    };
    let identity = decode_identity(text)
        .ok_or_else(|| denial("workflow operation receipt identity is malformed"))?;
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value,
    });
    Ok(Some(identity))
}

fn decode_identity(text: &str) -> Option<[u8; 32]> {
    if text.len() != 64 {
        return None;
    }
    let mut identity = [0_u8; 32];
    for (index, byte) in identity.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(identity)
}
