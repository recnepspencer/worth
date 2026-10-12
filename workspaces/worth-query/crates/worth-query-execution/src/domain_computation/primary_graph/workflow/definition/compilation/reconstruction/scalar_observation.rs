use super::*;

pub(super) fn observed_u64(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<u64, WorthQueryApplicationAttemptDenial> {
    let value = observe_field_value(runtime, execution, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow definition field is unavailable"))?;
    let AspectValue::UInt64(value) = value else {
        return Err(denial("workflow definition field has invalid type"));
    };
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value: AspectValue::UInt64(value),
    });
    Ok(value)
}

pub(super) fn observed_optional_u64(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<u64>, WorthQueryApplicationAttemptDenial> {
    let Some(value) = observe_field_value(runtime, execution, snapshot, entity, kind, locator)
    else {
        facts.push(WorthQueryApplicationObservedFact::AbsentField {
            entity_id: entity,
            kind,
            locator: locator.clone(),
        });
        return Ok(None);
    };
    let AspectValue::UInt64(number) = value else {
        return Err(denial("workflow definition field has invalid type"));
    };
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value: AspectValue::UInt64(number),
    });
    Ok(Some(number))
}

pub(super) fn observed_bool(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<bool, WorthQueryApplicationAttemptDenial> {
    let value = observe_field_value(runtime, execution, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow definition field is unavailable"))?;
    let AspectValue::Bool(value) = value else {
        return Err(denial("workflow definition field has invalid type"));
    };
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value: AspectValue::Bool(value),
    });
    Ok(value)
}

pub(super) fn observed_optional_text(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<String>, WorthQueryApplicationAttemptDenial> {
    let Some(value) = observe_field_value(runtime, execution, snapshot, entity, kind, locator)
    else {
        facts.push(WorthQueryApplicationObservedFact::AbsentField {
            entity_id: entity,
            kind,
            locator: locator.clone(),
        });
        return Ok(None);
    };
    let AspectValue::String(InternedString::Raw(text)) = &value else {
        return Err(denial("workflow definition field has invalid type"));
    };
    let text = text.clone();
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value,
    });
    Ok(Some(text))
}

pub(super) fn observed_text(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<String, WorthQueryApplicationAttemptDenial> {
    let value = observe_field_value(runtime, execution, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow definition field is unavailable"))?;
    let AspectValue::String(InternedString::Raw(text)) = &value else {
        return Err(denial("workflow definition field has invalid type"));
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
