//! Checked quotes of the concrete canonical owners, before their decode grows.
use super::borrowed_payload::{malformed, BorrowedPayload};
use super::RelationalCanonicalCommitArtifactRefusal as Refusal;
use worth_execution::ExecutionRequest;

mod admission;
mod checkpoint_effect;
pub(super) use admission::admit_canonical;
pub(crate) use checkpoint_effect::{decode_checkpoint_effect, CheckpointEffectData};
mod aspects;
mod branches;
mod cascades;
pub(super) mod codec_retirements;
mod content_schema;
mod contracts;
mod coupled_publication;
mod diagnostics;
mod envelope;
mod historical_selection;
mod identities;
mod indexes;
mod lineage;
pub(super) mod merge;
#[cfg(test)]
mod nonmerge_tests;
mod owned_graph_selection;
mod patch;
mod plan;
mod portable_aspects;
mod required_output;
pub(super) mod schema;
pub(crate) mod schema_image;
mod semantic;
mod strategy;
mod symbols;
mod touches;
pub(super) mod validation;
#[cfg(test)]
pub(super) use envelope::canonical as quote_canonical;
pub(super) use envelope::quote;

/// Concrete retained storage and additional simultaneous decode/check storage.
#[derive(Clone, Copy, Default)]
pub(super) struct OwnerAllocation {
    pub(super) retained: u64,
    pub(super) transient: u64,
}
impl From<crate::aspect_wire::CanonicalDecodeStorage> for OwnerAllocation {
    fn from(bytes: crate::aspect_wire::CanonicalDecodeStorage) -> Self {
        Self {
            retained: bytes.retained_bytes,
            transient: bytes.transient_bytes,
        }
    }
}
impl From<u64> for OwnerAllocation {
    fn from(retained: u64) -> Self {
        Self {
            retained,
            transient: 0,
        }
    }
}
impl OwnerAllocation {
    fn checked_add(self, other: impl Into<Self>) -> Option<Self> {
        let other = other.into();
        Some(Self {
            retained: self.retained.checked_add(other.retained)?,
            transient: self.transient.checked_add(other.transient)?,
        })
    }
    fn checked_mul(self, count: u64) -> Option<Self> {
        Some(Self {
            retained: self.retained.checked_mul(count)?,
            transient: self.transient.checked_mul(count)?,
        })
    }
}
#[derive(Default)]
pub(super) struct DecodedQuote {
    pub(super) control: u64,
    pub(super) envelope: OwnerAllocation,
    pub(super) semantic: OwnerAllocation,
    pub(super) association: OwnerAllocation,
    pub(super) index_delta: OwnerAllocation,
    pub(super) index_writes: OwnerAllocation,
    pub(super) origin: OwnerAllocation,
    pub(super) cascades: OwnerAllocation,
    pub(super) selection: OwnerAllocation,
}
fn add(total: &mut OwnerAllocation, bytes: impl Into<OwnerAllocation>) -> Result<(), Refusal> {
    *total = total.checked_add(bytes).ok_or_else(malformed)?;
    Ok(())
}
fn text(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    u64::try_from(value.text(request)?.len())
        .map(OwnerAllocation::from)
        .map_err(|_| malformed())
}
fn vector_capacity(count: usize) -> Result<usize, Refusal> {
    if count == 0 {
        return Ok(0);
    }
    // serde_core 1.0.229 VecVisitor initially reserves min(actual count,
    // 1 MiB / size_of::<T>()), then pushes. Pinned Vec geometric growth
    // needs at most twice the verified count (or its small minimum).
    count
        .checked_mul(2)
        .map(|count| count.max(8))
        .ok_or_else(malformed)
}
fn vector_storage<T>(count: usize) -> Result<u64, Refusal> {
    let layout =
        std::alloc::Layout::array::<T>(vector_capacity(count)?).map_err(|_| malformed())?;
    u64::try_from(layout.size()).map_err(|_| malformed())
}
fn vector<T>(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
    mut nested: impl FnMut(BorrowedPayload<'_>) -> Result<OwnerAllocation, Refusal>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    let count = value.elements(request, |element| add(&mut bytes, nested(element)?))?;
    let backing = vector_storage::<T>(count)?;
    add(&mut bytes, backing)?;
    // A relocating growth may simultaneously own old and new buffers. Keep
    // that additional concrete backing in the transient admission.
    bytes.transient = bytes.transient.checked_add(backing).ok_or_else(malformed)?;
    Ok(bytes)
}
fn vector_with_storage(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
    storage: impl FnOnce(usize) -> Option<u64>,
    mut nested: impl FnMut(BorrowedPayload<'_>) -> Result<OwnerAllocation, Refusal>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    let count = value.elements(request, |element| add(&mut bytes, nested(element)?))?;
    let backing = storage(vector_capacity(count)?).ok_or_else(malformed)?;
    add(&mut bytes, backing)?;
    bytes.transient = bytes.transient.checked_add(backing).ok_or_else(malformed)?;
    Ok(bytes)
}
fn optional(
    value: BorrowedPayload<'_>,
    quote: impl FnOnce(BorrowedPayload<'_>) -> Result<OwnerAllocation, Refusal>,
) -> Result<OwnerAllocation, Refusal> {
    if value.is_none() {
        Ok(OwnerAllocation::default())
    } else {
        quote(value)
    }
}
fn shared_root<T>() -> Result<OwnerAllocation, Refusal> {
    let layout = std::alloc::Layout::new::<(usize, usize)>()
        .extend(std::alloc::Layout::new::<T>())
        .map_err(|_| malformed())?
        .0
        .pad_to_align();
    u64::try_from(layout.size())
        .map(OwnerAllocation::from)
        .map_err(|_| malformed())
}
fn shared_vector<T>(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
    nested: impl FnMut(BorrowedPayload<'_>) -> Result<OwnerAllocation, Refusal>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = vector::<T>(value, request, nested)?;
    let count = value.elements(request, |_| Ok(()))?;
    let temporary = vector_storage::<T>(count)?;
    bytes.retained = bytes
        .retained
        .checked_sub(temporary)
        .ok_or_else(malformed)?;
    bytes.transient = bytes
        .transient
        .checked_add(temporary)
        .ok_or_else(malformed)?;
    let layout = std::alloc::Layout::array::<T>(count).map_err(|_| malformed())?;
    let layout = std::alloc::Layout::new::<(usize, usize)>()
        .extend(layout)
        .map_err(|_| malformed())?
        .0
        .pad_to_align();
    add(
        &mut bytes,
        u64::try_from(layout.size()).map_err(|_| malformed())?,
    )?;
    Ok(bytes)
}
fn shared_text(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = text(value, request)?;
    bytes.transient = bytes.retained;
    let tail =
        std::alloc::Layout::array::<u8>(usize::try_from(bytes.retained).map_err(|_| malformed())?)
            .map_err(|_| malformed())?;
    let layout = std::alloc::Layout::new::<(usize, usize)>()
        .extend(tail)
        .map_err(|_| malformed())?
        .0
        .pad_to_align();
    bytes.retained = u64::try_from(layout.size()).map_err(|_| malformed())?;
    Ok(bytes)
}
fn association(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    if value.is_none() {
        return Ok(OwnerAllocation::default());
    }
    let mut octets = value.octets(request)?;
    let count = octets.len();
    if count == 0 || count > crate::history::data::RelationalCallerAssociation::MAX_LEN {
        return Err(malformed());
    }
    octets.skip(count)?;
    octets.finish()?;
    let bytes = crate::history::data::RelationalCallerAssociation::retained_allocation_bytes(count)
        .ok_or_else(malformed)?;
    Ok(OwnerAllocation::from(bytes))
}

pub(super) fn field<'a>(
    value: BorrowedPayload<'a>,
    expected: &str,
    request: &ExecutionRequest<'_, '_>,
) -> Result<BorrowedPayload<'a>, Refusal> {
    let mut result = None;
    value.fields(request, |name, value| {
        if name == expected && result.replace(value).is_some() {
            return Err(malformed());
        }
        Ok(())
    })?;
    result.ok_or_else(malformed)
}
