//! Concrete retained and simultaneous decode storage for index generations.
mod entries;
use crate::durability::canonical_commit_artifact::{
    borrowed_payload::{malformed, BorrowedPayload},
    RelationalCanonicalCommitArtifactRefusal as Refusal,
};
use crate::indexes::data::DerivedIndexGeneration;
pub(super) use entries::quote as entries_quote;
use std::alloc::Layout;
use worth_execution::ExecutionRequest;

pub(super) fn add(total: &mut u64, bytes: u64) -> Result<(), Refusal> {
    *total = total.checked_add(bytes).ok_or_else(malformed)?;
    Ok(())
}
pub(super) fn vector<T>(count: usize) -> Result<u64, Refusal> {
    if count == 0 {
        return Ok(0);
    }
    // Serde sequence growth and a relocating old/new buffer both precede use.
    let capacity = count.checked_mul(2).ok_or_else(malformed)?.max(8);
    let bytes = Layout::array::<T>(capacity)
        .map_err(|_| malformed())?
        .size();
    u64::try_from(bytes)
        .ok()
        .and_then(|n| n.checked_mul(2))
        .ok_or_else(malformed)
}
pub(super) fn shared<T>() -> Result<u64, Refusal> {
    let layout = Layout::new::<[usize; 2]>()
        .extend(Layout::new::<T>())
        .map_err(|_| malformed())?
        .0
        .pad_to_align();
    u64::try_from(layout.size()).map_err(|_| malformed())
}
pub(super) fn roster(count: usize) -> Result<u64, Refusal> {
    let mut bytes = vector::<DerivedIndexGeneration>(count)?;
    add(
        &mut bytes,
        shared::<DerivedIndexGeneration>()?
            .checked_mul(count as u64)
            .ok_or_else(malformed)?,
    )?;
    Ok(bytes)
}
pub(super) fn full(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<u64, Refusal> {
    let mut bytes = 0;
    let count = value.elements(request, |generation| {
        generation.fields(request, |name, value| {
            match name {
                "source_branch_id" => add(&mut bytes, value.text_length(request)? as u64)?,
                "applicability" => {
                    add(
                        &mut bytes,
                        value.field("branch_id", request)?.text_length(request)? as u64,
                    )?;
                }
                "entries" => add(&mut bytes, entries::quote(value, request)?)?,
                _ => {}
            }
            Ok(())
        })
    })?;
    add(&mut bytes, roster(count)?)?;
    Ok(bytes)
}
// Complete storage bound of all reachable std B-tree nodes, rounded upward for
// five-key occupancy plus root and a possible in-flight split at every level.
pub(super) fn tree<K, V>(count: usize) -> Result<u64, Refusal> {
    if count == 0 {
        return Ok(0);
    }
    let leaf = Layout::new::<(usize, u16, u16)>()
        .extend(Layout::new::<[K; 11]>())
        .map_err(|_| malformed())?
        .0
        .extend(Layout::new::<[V; 11]>())
        .map_err(|_| malformed())?
        .0;
    let node = leaf
        .pad_to_align()
        .extend(Layout::new::<[usize; 12]>())
        .map_err(|_| malformed())?
        .0
        .pad_to_align()
        .size();
    let nodes = count.div_ceil(5).checked_add(2).ok_or_else(malformed)?;
    u64::try_from(node.checked_mul(nodes).ok_or_else(malformed)?).map_err(|_| malformed())
}
