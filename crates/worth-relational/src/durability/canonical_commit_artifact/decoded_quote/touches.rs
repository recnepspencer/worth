//! Exact descriptive touches own field paths and index comparison bytes.
use super::*;
use crate::history::data::RelationalDescriptiveTouch;
use worth_foundational::facade::FieldKey;

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    if !value.is_map() {
        return Ok(OwnerAllocation::default());
    }
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        if variant == "Exact" {
            add(
                &mut bytes,
                vector::<RelationalDescriptiveTouch>(value, request, |value| {
                    touch(value, request)
                })?,
            )?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn touch(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |_, value| {
        value.fields(request, |field, value| {
            add(
                &mut bytes,
                match field {
                    "aspect" => text(value, request)?,
                    "path" => vector::<FieldKey>(value, request, |value| text(value, request))?,
                    "old_key" | "new_key" => optional(value, |value| {
                        super::indexes::comparison_key(value, request)
                    })?,
                    _ => OwnerAllocation::default(),
                },
            )
        })
    })?;
    Ok(bytes)
}
