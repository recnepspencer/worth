//! The concrete intent grammar owns these vectors, keys and field payloads.
use super::*;
use crate::symbols::data::ClientKey;
use crate::transactions::data::{AspectFieldPatch, EntityReference, MutationIntent};

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "merged_intents" {
            add(
                &mut bytes,
                vector::<MutationIntent>(value, request, |value| intent(value, request))?,
            )?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn intent(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |family, value| {
        if !matches!(family, "Create" | "Entity" | "Relation" | "Materialization") {
            return Err(malformed());
        }
        value.fields(request, |variant, value| {
            let valid = match family {
                "Create" => matches!(
                    variant,
                    "Entity"
                        | "EntityAspects"
                        | "BulkEntities"
                        | "Relation"
                        | "RelationAspects"
                        | "BulkRelations"
                ),
                "Entity" => matches!(
                    variant,
                    "UpdateFields" | "ApplyAspectPatch" | "Replace" | "Delete" | "Revalidate"
                ),
                "Relation" => matches!(
                    variant,
                    "UpdateEndpoints" | "ApplyAspectPatch" | "Delete" | "Revalidate"
                ),
                "Materialization" => matches!(
                    variant,
                    "SuspendEntity"
                        | "SuspendRelation"
                        | "RematerializeEntity"
                        | "RematerializeRelation"
                ),
                _ => false,
            };
            if !valid {
                return Err(malformed());
            }
            add(&mut bytes, fields(value, request)?)
        })
    })?;
    Ok(bytes)
}
fn fields(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        let nested = match field {
            "client_key" => client_key(value, request)?,
            "client_keys" => {
                vector::<ClientKey>(value, request, |value| client_key(value, request))?
            }
            "source" | "target" => endpoint(value, request)?,
            "endpoints" => vector::<(EntityReference, EntityReference)>(value, request, |value| {
                let mut bytes = OwnerAllocation::default();
                let count =
                    value.elements(request, |value| add(&mut bytes, endpoint(value, request)?))?;
                if count != 2 {
                    return Err(malformed());
                }
                Ok(bytes)
            })?,
            "fields" => super::aspects::field_patch(value, request)?,
            "field_patches" => vector::<AspectFieldPatch>(value, request, |value| {
                super::aspects::field_patch(value, request)
            })?,
            "aspect_patch" => super::portable_aspects::patch(value, request)?,
            "replacement" => fields(value, request)?,
            _ => OwnerAllocation::default(),
        };
        add(&mut bytes, nested)
    })?;
    Ok(bytes)
}
pub(super) fn client_key(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| match variant {
        "Raw" => add(&mut bytes, text(value, request)?),
        "Symbol" => Ok(()),
        _ => Err(malformed()),
    })?;
    Ok(bytes)
}
fn endpoint(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| match variant {
        "Created" => add(&mut bytes, fields(value, request)?),
        "Existing" => Ok(()),
        _ => Err(malformed()),
    })?;
    Ok(bytes)
}
