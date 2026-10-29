//! Temporary backing newly copied by the managed Source scope resolver.
use crate::domain_computation::primary_graph::{
    application_query::{
        one_shot::outcome::material::MaterialCompletion, WorthQueryObservedScopeSelector,
    },
    source_facts::SourceFactsAllocation,
};
use std::sync::Arc;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(super) fn scope(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity: &str,
    aspect: &str,
    field: &str,
    selector: &WorthQueryObservedScopeSelector,
) -> Result<Arc<SourceFactsAllocation>, Cause> {
    let allocation = SourceFactsAllocation::open(execution)?;
    // equality_field's three temporary keys, the resulting entity name,
    // cloned installed locator/value, and one possible terminal subject.
    let names = entity
        .len()
        .checked_mul(2)
        .and_then(|bytes| bytes.checked_add(aspect.len()))
        .and_then(|bytes| bytes.checked_add(field.len()))
        .and_then(|bytes| bytes.checked_add(entity.len().max(field.len())))
        .ok_or(Cause::ChargedBytesOverflow)?;
    let bytes = names
        .checked_add(selector.locator().owned_allocation_capacity_bytes())
        .and_then(|bytes| bytes.checked_add(selector.value().owned_allocation_capacity_bytes()))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Cause::ChargedBytesOverflow)?;
    allocation.claim(execution, bytes)?;
    MaterialCompletion::new(execution, &allocation).work(bytes)?;
    Ok(allocation)
}
