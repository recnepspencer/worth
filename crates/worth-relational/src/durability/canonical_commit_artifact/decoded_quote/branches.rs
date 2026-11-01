//! Exact descriptive branch axes own names and parent vectors, never roots.
use super::*;

pub(super) fn descriptor(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "branch_id" => text(value, request)?,
                "reference" => observation(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
pub(super) fn cell(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "branch_id" => text(value, request)?,
                "observation" => observation(value, request)?,
                "fork_provenance" => optional(value, |value| observation(value, request))?,
                "fork_source_branch_id" => optional(value, |value| text(value, request))?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn observation(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "branch_id" {
            add(&mut bytes, text(value, request)?)?;
        }
        if field == "target" && value.is_map() {
            value.fields(request, |variant, value| {
                if variant == "Basis" {
                    value.fields(request, |field, value| {
                        if field == "parent_commit_ids" {
                            add(
                                &mut bytes,
                                vector::<u64>(value, request, |_| Ok(OwnerAllocation::default()))?,
                            )?;
                        }
                        Ok(())
                    })?;
                }
                Ok(())
            })?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
pub(super) fn merge_basis(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "source_head" | "target_head" => super::semantic::receipt(value, request)?,
                "merge_base" => merge_base(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn merge_base(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "commit" => super::semantic::receipt(value, request)?,
                "supporting_left_ancestors" | "supporting_right_ancestors" => {
                    ancestors(value, request)?
                }
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}

fn ancestors(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_vector::<crate::history::data::CommitId>(value, request, |_| {
        Ok(OwnerAllocation::default())
    })?;
    add(
        &mut bytes,
        crate::history::data::MergeBasisAncestors::control_bytes().ok_or_else(malformed)?,
    )?;
    Ok(bytes)
}
