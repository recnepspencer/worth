use super::*;

pub(super) fn required_text(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<String, WorthQueryApplicationAttemptDenial> {
    let value = observe_field_value(runtime, execution, snapshot, entity, kind, locator)?
        .ok_or_else(|| denial("workflow operation input field is unavailable"))?;
    let AspectValue::String(InternedString::Raw(text)) = &value else {
        return Err(denial("workflow operation input field has the wrong type"));
    };
    let text = text.clone();
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value,
    });
    Ok(text)
}

pub(super) fn required_u64(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<u64, WorthQueryApplicationAttemptDenial> {
    let value = observe_field_value(runtime, execution, snapshot, entity, kind, locator)?
        .ok_or_else(|| denial("workflow proposal numeric field is unavailable"))?;
    let AspectValue::UInt64(number) = value else {
        return Err(denial("workflow proposal numeric field has the wrong type"));
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<[u8; 32]>, WorthQueryApplicationAttemptDenial> {
    let Some(value) = observe_field_value(runtime, execution, snapshot, entity, kind, locator)?
    else {
        facts.push(WorthQueryApplicationObservedFact::AbsentField {
            entity_id: entity,
            kind,
            locator: locator.clone(),
        });
        return Ok(None);
    };
    let AspectValue::String(InternedString::Raw(text)) = &value else {
        return Err(denial(
            "workflow operation source identity has the wrong type",
        ));
    };
    let identity = decode_identity(text)
        .ok_or_else(|| denial("workflow operation source identity is malformed"))?;
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value,
    });
    Ok(Some(identity))
}

pub(in crate::domain_computation::primary_graph) fn decode_identity(
    text: &str,
) -> Option<[u8; 32]> {
    if text.len() != 64 || !text.is_ascii() {
        return None;
    }
    let mut identity = [0_u8; 32];
    for (index, byte) in identity.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(identity)
}

#[cfg(test)]
mod identity_tests {
    #[test]
    fn proposal_identity_rejects_utf8_split_before_slicing() {
        let corrupt = format!("aé{}", "0".repeat(61));
        assert_eq!(corrupt.len(), 64);
        assert!(super::decode_identity(&corrupt).is_none());
        assert_eq!(super::decode_identity(&"0".repeat(64)), Some([0; 32]));
    }
}
