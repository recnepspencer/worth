//! Logical field text resolved against the SAME native retained symbol dictionary.
use super::admission::WorthQueryWorkflowReadDenial as Denial;
use crate::domain_computation::primary_graph::application_attempt::observe_field_value_borrowed;
use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::RelationalPinnedRead,
};

pub(super) fn inspect<R>(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    pinned: &RelationalPinnedRead,
    entity: EntityId,
    kind: KindId,
    field: &AspectFieldLocator,
    inspect: impl FnOnce(Option<&str>) -> R,
) -> Result<R, Denial> {
    observe_field_value_borrowed(
        runtime,
        execution,
        pinned.snapshot(),
        entity,
        kind,
        field,
        |value| match value {
            Some(AspectValue::String(InternedString::Raw(text))) => {
                Ok(inspect(Some(text.as_str())))
            }
            Some(AspectValue::String(InternedString::Symbol(symbol))) => pinned.with_symbol_text(
                execution,
                worth_relational::facade::symbols::Symbol(symbol.0),
                |text| inspect(text.map(|text| text.as_str())),
            ),
            _ => Ok(inspect(None)),
        },
    )
    .map_err(Denial::Relational)?
    .ok_or(Denial::Corrupt)?
    .map_err(Denial::Relational)
}
