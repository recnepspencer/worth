//! Reuse the identity/policy leaf owners on the actual durable declarations.
use super::*;
use crate::durability::data::DurableSchemaDeclarationMemory;
use crate::history::data::CodecDeclarationsReadmissionMemory;
use crate::merge::data::{MergeIdentityMemory, MergeLeafMemory};

pub(in super::super::super) fn prepare_canonical_readmission(
    canonical: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(CodecDeclarationsReadmissionMemory, u64), Refusal> {
    let events = field(canonical, "required_output_codec_retirements", request)?;
    let mut count = 0usize;
    let mut retained = 0u64;
    visit(events, request, &mut |declarations| {
        count = count.checked_add(1).ok_or_else(malformed)?;
        visit_leaves(
            declarations,
            request,
            &mut |raw| sum(&mut retained, identities::basis(raw, request)?.retained),
            &mut |_| Ok(()),
        )?;
        visit_leaves(declarations, request, &mut |_| Ok(()), &mut |raw| {
            sum(&mut retained, identities::policy(raw, request)?.retained)
        })
    })?;
    let tree_memory = request
        .reserve_retained_memory(layout::<DurableSchemaDeclarationMemory>(count)?)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let mut declarations = Vec::with_capacity(count);
    visit(events, request, &mut |raw| {
        declarations.push(prepare_declaration(raw, request)?);
        Ok(())
    })?;
    Ok((
        CodecDeclarationsReadmissionMemory {
            declarations,
            _tree_memory: tree_memory,
        },
        retained,
    ))
}
fn prepare_declaration(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<DurableSchemaDeclarationMemory, Refusal> {
    let (mut identity_count, mut policy_count) = (0usize, 0usize);
    visit_leaves(
        value,
        request,
        &mut |_| {
            identity_count = identity_count.checked_add(1).ok_or_else(malformed)?;
            Ok(())
        },
        &mut |_| {
            policy_count = policy_count.checked_add(1).ok_or_else(malformed)?;
            Ok(())
        },
    )?;
    let bytes = layout::<MergeIdentityMemory>(identity_count)?
        .checked_add(layout::<Option<MergeLeafMemory>>(policy_count)?)
        .ok_or_else(malformed)?;
    let memory = request
        .reserve_retained_memory(bytes)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let mut identities = Vec::with_capacity(identity_count);
    let mut policies = Vec::with_capacity(policy_count);
    visit_leaves(
        value,
        request,
        &mut |raw| {
            identities.push(identity(raw, request)?);
            Ok(())
        },
        &mut |raw| {
            policies.push(policy(raw, request)?);
            Ok(())
        },
    )?;
    Ok(DurableSchemaDeclarationMemory {
        identities,
        policies,
        _tree_memory: memory,
    })
}
fn visit<'a>(
    value: BorrowedPayload<'a>,
    request: &ExecutionRequest<'_, '_>,
    visit: &mut impl FnMut(BorrowedPayload<'a>) -> Result<(), Refusal>,
) -> Result<(), Refusal> {
    value.elements(request, |event| {
        field(event, "kinds", request)?.elements(request, |kind| {
            for name in ["source", "target"] {
                visit(field(field(kind, name, request)?, "declarations", request)?)?;
            }
            Ok(())
        })?;
        Ok(())
    })?;
    Ok(())
}
fn visit_leaves<'a>(
    value: BorrowedPayload<'a>,
    request: &ExecutionRequest<'_, '_>,
    identity: &mut impl FnMut(BorrowedPayload<'a>) -> Result<(), Refusal>,
    policy: &mut impl FnMut(BorrowedPayload<'a>) -> Result<(), Refusal>,
) -> Result<(), Refusal> {
    field(value, "identity_declarations", request)?.elements(request, |declaration| {
        identity(field(declaration, "basis", request)?)
    })?;
    field(value, "merge_policy_declarations", request)?.elements(request, |declaration| {
        policy(field(declaration, "policy", request)?)
    })?;
    Ok(())
}
fn identity(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<MergeIdentityMemory, Refusal> {
    let mut memory = MergeIdentityMemory::default();
    if value.is_map() {
        let bytes = identities::basis(value, request)?.retained;
        value.fields(request, |variant, raw| {
            match variant {
                "DeclaredKeySet" => {
                    memory.keys = Some(leaf(raw.elements(request, |_| Ok(()))?, bytes, request)?)
                }
                "Custom" => {
                    memory.custom_name = Some(leaf(
                        field(raw, "name", request)?.text(request)?.len(),
                        bytes,
                        request,
                    )?)
                }
                _ => {}
            }
            Ok(())
        })?;
    }
    Ok(memory)
}
fn policy(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Option<MergeLeafMemory>, Refusal> {
    let mut memory = None;
    if value.is_map() {
        value.fields(request, |variant, raw| {
            if variant == "Custom" {
                memory = Some(leaf(
                    field(raw, "name", request)?.text(request)?.len(),
                    identities::policy(value, request)?.retained,
                    request,
                )?);
            }
            Ok(())
        })?;
    }
    Ok(memory)
}
fn leaf(
    count: usize,
    bytes: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<MergeLeafMemory, Refusal> {
    let memory = request
        .reserve_retained_memory(bytes)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    Ok(MergeLeafMemory::new(count, bytes, memory))
}
fn layout<T>(count: usize) -> Result<u64, Refusal> {
    u64::try_from(
        std::alloc::Layout::array::<T>(count)
            .map_err(|_| malformed())?
            .size(),
    )
    .map_err(|_| malformed())
}
fn sum(total: &mut u64, bytes: u64) -> Result<(), Refusal> {
    *total = total.checked_add(bytes).ok_or_else(malformed)?;
    Ok(())
}
