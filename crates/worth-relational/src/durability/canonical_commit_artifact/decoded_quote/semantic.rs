//! The semantic Arc can escape independently; quote its actual nested owners.
use super::*;
use crate::transactions::data::{
    RecordRef, RelationalCreatedEntityCorrespondence, RelationalCreatedRelationCorrespondence,
    RelationalInvariantExecutionMeaning,
};

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_root::<crate::transactions::data::RelationalCommitSemanticResult>()?;
    value.fields(request, |field, value| {
        if matches!(field, "created_entities" | "created_relations") {
            let count = value.elements(request, |_| Ok(()))?;
            let scratch = std::alloc::Layout::array::<RecordRef>(count).map_err(|_| malformed())?;
            bytes.transient = bytes
                .transient
                .checked_add(u64::try_from(scratch.size()).map_err(|_| malformed())?)
                .ok_or_else(malformed)?;
        }
        let nested = match field {
            "receipt" => receipt(value, request)?,
            "created_entities" => {
                vector::<RelationalCreatedEntityCorrespondence>(value, request, |entry| {
                    entity_correspondence(entry, request)
                })?
            }
            "created_relations" => {
                vector::<RelationalCreatedRelationCorrespondence>(value, request, |entry| {
                    relation_correspondence(entry, request)
                })?
            }
            "changed_records" => {
                vector::<RecordRef>(value, request, |_| Ok(OwnerAllocation::default()))?
            }
            "structural" => structural(value, request)?,
            "validation" => {
                vector::<RelationalInvariantExecutionMeaning>(value, request, |entry| {
                    super::validation::execution(entry, request)
                })?
            }
            // Summary owners have their own concrete fields; their input is
            // visited below rather than measured by serializing decoded data.
            "history" => history(value, request)?,
            "aspects" => aspects(value, request)?,
            "schema" => schema(value, request)?,
            "change" | "publication" => OwnerAllocation::default(),
            _ => OwnerAllocation::default(),
        };
        add(&mut bytes, nested)
    })?;
    Ok(bytes)
}

pub(super) fn receipt(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "branch_id" => text(value, request)?,
                "parents" => vector::<crate::history::data::CommitId>(value, request, |_| {
                    Ok(OwnerAllocation::default())
                })?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}

fn entity_correspondence(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "key" {
            add(&mut bytes, key(value, request)?)?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn relation_correspondence(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    entity_correspondence(value, request)
}
fn key(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "client_key" => text(value, request)?,
                "source" | "target" => endpoint(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn endpoint(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    if value
        .0
        .first()
        .is_some_and(|marker| *marker >= 0xa0 && *marker <= 0xbf)
    {
        return Ok(OwnerAllocation::default());
    }
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        if variant == "Created" {
            add(&mut bytes, key(value, request)?)?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn structural(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "touched_partitions" {
            add(
                &mut bytes,
                vector::<crate::identity::data::PartitionId>(value, request, |_| {
                    Ok(OwnerAllocation::default())
                })?,
            )?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn history(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "target_branch" {
            add(&mut bytes, text(value, request)?)?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn aspects(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "touched_aspects" {
            add(
                &mut bytes,
                vector::<worth_foundational::facade::AspectKey>(value, request, |entry| {
                    text(entry, request)
                })?,
            )?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn schema(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "transition" {
            add(
                &mut bytes,
                optional(value, |value| {
                    let mut bytes = OwnerAllocation::default();
                    value.fields(request, |field, value| {
                        if field == "changed_strata" {
                            add(
                                &mut bytes,
                                vector::<crate::schema::data::SchemaStratum>(
                                    value,
                                    request,
                                    |_| Ok(OwnerAllocation::default()),
                                )?,
                            )?;
                        }
                        Ok(())
                    })?;
                    Ok(bytes)
                })?,
            )?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
