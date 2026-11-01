//! The persisted definition delta and authored evidence have distinct owners.
use super::*;
use crate::history::data::{
    CanonicalAuthoredIndexWrite, CanonicalAuthoredIndexWrites, CanonicalIndexDefinitionDelta,
};
use crate::indexes::data::{DerivedIndexDefinition, RelatedEntityOrderingField};

pub(super) fn delta(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "definitions" {
            add(
                &mut bytes,
                vector::<DerivedIndexDefinition>(value, request, |entry| {
                    definition(entry, request)
                })?,
            )?;
            if value.elements(request, |_| Ok(()))? != 0 {
                add(
                    &mut bytes,
                    CanonicalIndexDefinitionDelta::root_bytes().ok_or_else(malformed)?,
                )?;
            }
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn definition(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "name" => text(value, request)?,
                "kind" => kind(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn kind(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        match variant {
            "EntityField" | "RelationField" => value.fields(request, |field, value| {
                if field == "field_locator" {
                    add(&mut bytes, super::aspects::locator(value, request)?)?;
                }
                Ok(())
            })?,
            "RelatedEntityOrdering" => value.fields(request, |field, value| {
                if field == "ordering" {
                    add(
                        &mut bytes,
                        vector::<RelatedEntityOrderingField>(value, request, |entry| {
                            let mut bytes = OwnerAllocation::default();
                            entry.fields(request, |field, value| {
                                if field == "locator" {
                                    add(&mut bytes, super::aspects::locator(value, request)?)?;
                                }
                                Ok(())
                            })?;
                            Ok(bytes)
                        })?,
                    )?;
                }
                Ok(())
            })?,
            // RelationJoinDefinition and its two legs are fixed scalar fields.
            "RelationJoin" => {}
            _ => {}
        }
        Ok(())
    })?;
    Ok(bytes)
}
pub(super) fn authored(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    if matches!(value.0.first(), Some(0xa0..=0xbf | 0xd9..=0xdb)) {
        return Ok(OwnerAllocation::default());
    }
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        if variant == "Complete" {
            add(
                &mut bytes,
                CanonicalAuthoredIndexWrites::root_bytes().ok_or_else(malformed)?,
            )?;
            add(
                &mut bytes,
                vector::<CanonicalAuthoredIndexWrite>(value, request, |entry| {
                    let mut bytes = OwnerAllocation::default();
                    entry.fields(request, |field, value| {
                        add(
                            &mut bytes,
                            match field {
                                "definition" => authored_definition(value, request)?,
                                "old_key" | "new_key" => {
                                    optional(value, |value| comparison_key(value, request))?
                                }
                                _ => OwnerAllocation::default(),
                            },
                        )
                    })?;
                    Ok(bytes)
                })?,
            )?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn authored_definition(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "name" => text(value, request)?,
                "locator" => super::aspects::locator(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
pub(super) fn comparison_key(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "canonical_value_bytes" {
            add(
                &mut bytes,
                vector::<u8>(value, request, |_| Ok(OwnerAllocation::default()))?,
            )?;
        }
        Ok(())
    })?;
    Ok(bytes)
}

#[cfg(test)]
mod tests;
