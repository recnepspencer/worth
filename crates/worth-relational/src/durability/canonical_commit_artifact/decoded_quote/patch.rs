//! Canonical patch operation owners, including full struct-field meaning.
use super::*;
use crate::publication::patch::data::{
    PublishedAuthoritativeAspectChange, PublishedAuthoritativeFieldSet,
    PublishedAuthoritativePatchOperation, PublishedAuthoritativeRecordPatch,
};
use worth_foundational::facade::FieldKey;

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "authoritative_record_patches" {
            let count = value.elements(request, |_| Ok(()))?;
            let scratch = std::alloc::Layout::array::<usize>(count).map_err(|_| malformed())?;
            bytes.transient = bytes
                .transient
                .checked_add(u64::try_from(scratch.size()).map_err(|_| malformed())?)
                .ok_or_else(malformed)?;
            add(
                &mut bytes,
                vector::<PublishedAuthoritativeRecordPatch>(value, request, |value| {
                    record(value, request)
                })?,
            )?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn record(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "authoritative_patch" => operations(value, request)?,
                "semantic_changes" => {
                    vector::<PublishedAuthoritativeAspectChange>(value, request, |value| {
                        change(value, request)
                    })?
                }
                "detail" => detail(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn operations(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "operations" {
            add(
                &mut bytes,
                vector::<PublishedAuthoritativePatchOperation>(value, request, |value| {
                    operation(value, request)
                })?,
            )?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn operation(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        if !matches!(
            variant,
            "WholeAspectSet" | "WholeAspectClear" | "FieldLevelPatch"
        ) {
            return Err(malformed());
        }
        value.fields(request, |field, value| {
            add(
                &mut bytes,
                match field {
                    "aspect_key" => text(value, request)?,
                    "binding" => binding(value, request)?,
                    "value" => super::portable_aspects::validation_input(value, request)?,
                    "field_sets" => {
                        vector::<PublishedAuthoritativeFieldSet>(value, request, |value| {
                            field_set(value, request)
                        })?
                    }
                    "field_clears" => {
                        vector::<FieldKey>(value, request, |value| text(value, request))?
                    }
                    _ => OwnerAllocation::default(),
                },
            )
        })
    })?;
    Ok(bytes)
}
fn field_set(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "field" => text(value, request)?,
                "value" => super::aspects::scalar(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
pub(super) fn binding(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    if !value.is_map() {
        return Ok(OwnerAllocation::default());
    }
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        if matches!(variant, "EntityField" | "RelationField") {
            value.fields(request, |field, value| {
                if field == "field" {
                    add(&mut bytes, text(value, request)?)?;
                }
                Ok(())
            })?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn change(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "aspect_key" => text(value, request)?,
                "binding" => binding(value, request)?,
                "field_path" => optional(value, |value| {
                    vector::<FieldKey>(value, request, |value| text(value, request))
                })?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn detail(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        if variant != "DenseBitset" {
            return Err(malformed());
        }
        add(
            &mut bytes,
            vector::<u64>(value, request, |_| Ok(OwnerAllocation::default()))?,
        )
    })?;
    Ok(bytes)
}
