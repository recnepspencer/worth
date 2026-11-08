//! A complete native ordered input needs no persistent path replacements.
use super::{
    funding::PathFunding, node, request::funded, ExecutionOrderedMap, ExecutionOrderedMapDenial,
};
use crate::ExecutionRequest;
use std::sync::Arc;

enum Failure {
    Input,
    Storage(ExecutionOrderedMapDenial),
}
impl From<ExecutionOrderedMapDenial> for Failure {
    fn from(value: ExecutionOrderedMapDenial) -> Self {
        Self::Storage(value)
    }
}
impl<K: Ord, V> ExecutionOrderedMap<K, V> {
    /// Build one balanced cell per strictly increasing native pair.
    /// `None` means invalid length/order; valid empty input returns an empty map.
    /// Payloads move with their existing custody. The native caller owns admission
    /// of key comparisons and construction work, as for native insertion.
    pub fn from_sorted_native_for_request(
        count: usize,
        entries: impl IntoIterator<Item = (K, V)>,
        request: ExecutionRequest<'_, '_>,
    ) -> Result<Option<Self>, ExecutionOrderedMapDenial> {
        funded(request, |funding| {
            let bytes = if funding.uses_accounting() {
                u64::try_from(count)
                    .ok()
                    .and_then(|n| n.checked_mul(node::cell_bytes::<K, V>().ok()?))
                    .ok_or(ExecutionOrderedMapDenial::CapacityOverflow)?
            } else {
                0
            };
            let pool = PathFunding::admit(funding, bytes)?;
            let mut entries = entries.into_iter();
            let root = build(count, &mut entries, &mut None, &pool, request);
            match root {
                Ok(root) if entries.next().is_none() => Ok(Some(Self { root })),
                Ok(_) | Err(Failure::Input) => Ok(None),
                Err(Failure::Storage(cause)) => Err(cause),
            }
        })
    }
}

fn build<K: Ord, V>(
    count: usize,
    entries: &mut impl Iterator<Item = (K, V)>,
    previous: &mut Option<Arc<node::Entry<K, V>>>,
    funding: &PathFunding,
    request: ExecutionRequest<'_, '_>,
) -> Result<node::Link<K, V>, Failure> {
    request
        .consult()
        .map_err(ExecutionOrderedMapDenial::Request)?;
    if count == 0 {
        return Ok(None);
    }
    let left = build(count / 2, entries, previous, funding, request)?;
    let (key, value) = entries.next().ok_or(Failure::Input)?;
    if previous.as_ref().is_some_and(|old| old.key >= key) {
        return Err(Failure::Input);
    }
    let entry = node::Entry::owned_native(key, value, funding)?;
    *previous = Some(entry.clone());
    let right = build(count - count / 2 - 1, entries, previous, funding, request)?;
    Ok(Some(node::build(entry, left, right, funding)?))
}
