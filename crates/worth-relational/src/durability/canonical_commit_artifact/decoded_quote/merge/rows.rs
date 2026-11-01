//! Correspondence and schema witnesses own typed rows and validation scratch.
use super::*;
use crate::merge::data::{
    RelationalMergeCorrespondenceWitnessRow, RelationalSchemaReconciliationWitnessRow,
};
use crate::schema::data::{SchemaDiffAtom, SchemaStratum};
use crate::transactions::data::RecordRef;

pub(super) fn request(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if matches!(field, "target_branch" | "source_branch" | "request_digest") {
            add(&mut bytes, text(value, request)?)?;
        }
        Ok(())
    })?;
    bytes.transient = 64; // Normalized request digest validation.
    Ok(bytes)
}

pub(super) fn correspondence(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        match field {
            "rows" => {
                add(
                    &mut bytes,
                    shared_rows::<RelationalMergeCorrespondenceWitnessRow>(
                        value,
                        request,
                        |value| correspondence_row(value, request),
                    )?,
                )?;
                let count = value.elements(request, |_| Ok(()))?;
                // The owner builds two exact-capacity frequency rosters, then
                // sorts in place. RecordRef contains no nested allocation.
                let backing = std::alloc::Layout::array::<RecordRef>(count)
                    .map_err(|_| malformed())?
                    .size();
                let scratch = u64::try_from(backing)
                    .map_err(|_| malformed())?
                    .checked_mul(2)
                    .ok_or_else(malformed)?;
                bytes.transient = bytes.transient.checked_add(scratch).ok_or_else(malformed)?;
            }
            "request_digest" | "branch_basis_digest" | "witness_digest" => {
                add(&mut bytes, text(value, request)?)?;
            }
            _ => {}
        }
        Ok(())
    })?;
    bytes.transient = bytes.transient.checked_add(64).ok_or_else(malformed)?;
    Ok(bytes)
}

pub(super) fn correspondence_row(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "scope" => optional(value, |value| identities::scope(value, request))?,
                "authority_basis" => identities::basis(value, request)?,
                "candidate_digest" => text(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    // candidate() clones an optional AspectKey, but shares declared-key/custom
    // Arc owners. Only that actual scalar key clone needs additional backing.
    value.fields(request, |field, value| {
        if field == "scope" {
            let clone = optional(value, |value| identities::scope(value, request))?;
            bytes.transient = bytes
                .transient
                .checked_add(clone.retained)
                .ok_or_else(malformed)?;
        }
        Ok(())
    })?;
    bytes.transient = bytes.transient.checked_add(64).ok_or_else(malformed)?;
    Ok(bytes)
}

pub(super) fn schema(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "rows" => shared_rows::<RelationalSchemaReconciliationWitnessRow>(
                    value,
                    request,
                    |value| schema_row(value, request),
                )?,
                "request_digest" | "branch_basis_digest" | "witness_digest" => {
                    text(value, request)?
                }
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    bytes.transient = bytes.transient.checked_add(64).ok_or_else(malformed)?;
    Ok(bytes)
}

pub(super) fn schema_row(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    let mut input_clone = 0u64;
    let mut schema_names = 0u64;
    value.fields(request, |field, value| {
        let nested = match field {
            "basis" => {
                let basis = schema_basis(value, request)?;
                schema_names = basis.retained;
                input_clone = input_clone
                    .checked_add(basis.retained)
                    .ok_or_else(malformed)?;
                basis
            }
            "correspondence_linkage" => optional(value, |value| {
                let mut linkage = OwnerAllocation::default();
                value.fields(request, |field, value| {
                    match field {
                        "scope" => {
                            let scope = identities::scope(value, request)?;
                            input_clone = input_clone
                                .checked_add(scope.retained)
                                .ok_or_else(malformed)?;
                            add(&mut linkage, scope)?;
                        }
                        "basis" => add(&mut linkage, identities::basis(value, request)?)?,
                        _ => {}
                    }
                    Ok(())
                })?;
                Ok(linkage)
            })?,
            "row_digest" => text(value, request)?,
            _ => OwnerAllocation::default(),
        };
        add(&mut bytes, nested)
    })?;
    let transition = schema_transition_scratch(schema_names)?;
    bytes.transient = bytes
        .transient
        .checked_add(input_clone)
        .and_then(|n| n.checked_add(transition))
        .and_then(|n| n.checked_add(64))
        .ok_or_else(malformed)?;
    Ok(bytes)
}

fn schema_basis(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if matches!(field, "source_schema_id" | "target_schema_id") {
            add(&mut bytes, optional(value, |value| text(value, request))?)?;
        } else if field == "registry_digest" {
            add(&mut bytes, text(value, request)?)?;
        }
        Ok(())
    })?;
    Ok(bytes)
}

fn schema_transition_scratch(names: u64) -> Result<u64, Refusal> {
    // One synthetic atom, two strata, source/target/element schema names.
    // Sum the possible detail branches, so malformed semantic inputs are funded
    // before owner validation too; none use Std tree nodes or serialized buffers.
    let mut bytes = names
        .checked_add("merge-schema-target-".len() as u64)
        .and_then(|n| n.checked_mul(3))
        .ok_or_else(malformed)?;
    for backing in [
        std::alloc::Layout::array::<SchemaDiffAtom>(1)
            .map_err(|_| malformed())?
            .size(),
        std::alloc::Layout::array::<SchemaStratum>(2)
            .map_err(|_| malformed())?
            .size(),
        "merge_schema".len(),
    ] {
        bytes = bytes
            .checked_add(u64::try_from(backing).map_err(|_| malformed())?)
            .ok_or_else(malformed)?;
    }
    for literal in [
        "merge_schema_reconciliation",
        "merge-source-shape",
        "merge-target-shape",
        "merge schema reconciliation detected structural continuity denial",
    ] {
        let layout = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::array::<u8>(literal.len()).map_err(|_| malformed())?)
            .map_err(|_| malformed())?
            .0
            .pad_to_align();
        bytes = bytes
            .checked_add(u64::try_from(layout.size()).map_err(|_| malformed())?)
            .ok_or_else(malformed)?;
    }
    let controls = crate::schema::data::SchemaChangeText::control_bytes()
        .and_then(|bytes| bytes.checked_mul(4))
        .ok_or_else(malformed)?;
    bytes = bytes.checked_add(controls).ok_or_else(malformed)?;
    Ok(bytes)
}
