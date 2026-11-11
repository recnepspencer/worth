//! Merge identity/policy descriptions own only their declared names and keys.
use super::*;
use crate::merge::data::{DeclaredIdentityKeys, IdentityBasisName, MergePolicyName};
use worth_foundational::facade::AspectKey;

pub(super) fn declaration(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "scope" => scope(value, request)?,
                "basis" => basis(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
pub(super) fn scope(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        if variant == "AspectKey" {
            add(&mut bytes, text(value, request)?)?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
pub(super) fn basis(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    if !value.is_map() {
        return Ok(OwnerAllocation::default());
    }
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        add(
            &mut bytes,
            match variant {
                "DeclaredKeySet" => {
                    let mut keys =
                        shared_vector::<AspectKey>(value, request, |value| text(value, request))?;
                    add(
                        &mut keys,
                        DeclaredIdentityKeys::control_bytes().ok_or_else(malformed)?,
                    )?;
                    keys
                }
                "Custom" => custom(
                    value,
                    request,
                    IdentityBasisName::control_bytes().ok_or_else(malformed)?,
                )?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
pub(super) fn policy_declaration(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "aspect_key" => text(value, request)?,
                "policy" => policy(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
pub(super) fn policy(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    if !value.is_map() {
        return Ok(OwnerAllocation::default());
    }
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        if variant == "Custom" {
            add(
                &mut bytes,
                custom(
                    value,
                    request,
                    MergePolicyName::control_bytes().ok_or_else(malformed)?,
                )?,
            )?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn custom(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
    control: u64,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::from(control);
    value.fields(request, |field, value| {
        if field == "name" {
            add(&mut bytes, shared_text(value, request)?)?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
