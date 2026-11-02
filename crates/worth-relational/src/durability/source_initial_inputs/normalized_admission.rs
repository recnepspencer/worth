//! Quote the concrete normalized record owners before MessagePack allocates them.
use super::normalized::*;
use super::RelationalInitialInputsRefusal as Refusal;
use crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub(crate) fn readmit(
    bytes: &[u8],
    covered: u64,
    through: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Arc<NormalizedOriginalInputs>, Refusal> {
    let raw = BorrowedPayload(bytes);
    let canonical = raw.field("canonical", request).map_err(Refusal::Payload)?;
    let transitions = raw
        .field("transitions", request)
        .map_err(Refusal::Payload)?;
    let c = canonical
        .elements(request, |_| Ok(()))
        .map_err(Refusal::Payload)?;
    let mut dynamic = 0u64;
    let t = transitions
        .elements(request, |record| {
            dynamic = dynamic
                .checked_add(transition_dynamic_bytes(
                    record.field("transition", request)?,
                    request,
                )?)
                .ok_or_else(overflow)?;
            Ok(())
        })
        .map_err(Refusal::Payload)?;
    if c as u64 != covered || t as u64 != through {
        return Err(Refusal::Malformed);
    }
    let control = arc_layout::<NormalizedOriginalInputs>()?;
    let bytes = layout::<CanonicalOriginalRecord>(c)?
        .checked_add(layout::<TransitionOriginalRecord>(t)?)
        .and_then(|n| n.checked_add(dynamic))
        .and_then(|n| n.checked_add(control))
        .ok_or(Refusal::Malformed)?;
    let memory = request
        .reserve_retained_memory(bytes)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let mut original: NormalizedOriginalInputs =
        rmp_serde::from_slice(bytes_raw(raw)).map_err(|_| Refusal::Malformed)?;
    if original.canonical.len() != c || original.transitions.len() != t {
        return Err(Refusal::Malformed);
    }
    for (offset, record) in original.canonical.iter().enumerate() {
        if record.position.0 != offset as u64 + 1 {
            return Err(Refusal::Malformed);
        }
    }
    let mut follows = 0;
    for (offset, record) in original.transitions.iter().enumerate() {
        if record.sequence.0 != offset as u64 + 1
            || record.follows.0 < follows
            || record.follows.0 > covered
        {
            return Err(Refusal::Malformed);
        }
        follows = record.follows.0;
    }
    original.memory = memory;
    Ok(Arc::new(original))
}

pub(crate) fn transition_dynamic_bytes(
    raw: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    u64,
    crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal,
> {
    let mut bytes = 0u64;
    raw.fields(request, |variant, record| {
        let names: &[&str] = match variant {
            "Fork" => &["target", "source"],
            "Archive" | "DeletionRequested" | "Delete" => &["branch"],
            _ => return Err(overflow()),
        };
        for name in names {
            bytes = bytes
                .checked_add(record.field(name, request)?.text_length(request)? as u64)
                .ok_or_else(overflow)?;
        }
        if variant == "Fork" {
            let count = record
                .field("source_head", request)?
                .field("parent_commit_ids", request)?
                .elements(request, |_| Ok(()))?;
            bytes = bytes
                .checked_add(
                    count
                        .checked_mul(std::mem::size_of::<u64>())
                        .ok_or_else(overflow)? as u64,
                )
                .ok_or_else(overflow)?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn bytes_raw(raw: BorrowedPayload<'_>) -> &[u8] {
    raw.0
}
fn layout<T>(count: usize) -> Result<u64, Refusal> {
    std::alloc::Layout::array::<T>(count)
        .map(|layout| layout.size() as u64)
        .map_err(|_| Refusal::Malformed)
}
fn arc_layout<T>() -> Result<u64, Refusal> {
    std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map(|(layout, _)| layout.pad_to_align().size() as u64)
        .map_err(|_| Refusal::Malformed)
}
fn overflow(
) -> crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal {
    crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::Request(
        crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
    )
}
