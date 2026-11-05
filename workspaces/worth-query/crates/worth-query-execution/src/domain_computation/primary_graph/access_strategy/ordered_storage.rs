//! Every copied ordered cell retains its original SDK grant, never a payer chain.
use worth_execution::{ExecutionOrderedMap as Map, ExecutionRequest};
use {
    crate::domain_computation::vocabulary::primary_graph::control::WorthQueryManagedIndexControl as Control,
    crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop as Stop,
};

pub(in crate::domain_computation::primary_graph) fn lookup(
    control: &Control<'_>,
    count: usize,
    key_bytes: usize,
) -> Result<(), Stop> {
    let levels = usize::BITS
        - count
            .checked_add(1)
            .ok_or_else(|| control.refuse(Stop::CapacityOverflow))?
            .leading_zeros();
    let work = u64::from(levels)
        .checked_mul(2)
        .and_then(|height| height.checked_add(1))
        .and_then(|height| height.checked_mul(key_bytes as u64 + 1))
        .ok_or_else(|| control.refuse(Stop::CapacityOverflow))?;
    control.checkpoint(work)
}
pub(in crate::domain_computation::primary_graph) fn insert<K: Ord + Clone, V: Clone>(
    map: &mut Map<K, V>,
    key: &K,
    value: &V,
    execution: &ExecutionRequest<'_, '_>,
    control: &Control<'_>,
) -> Result<(), Stop> {
    for _ in 0..4 {
        lookup(
            control,
            map.len(),
            size_of::<K>() + size_of::<V>() + 16 * size_of::<usize>(),
        )?;
    }
    map.insert_cloned_native_for_request(key, value, *execution)
        .map_err(|cause| control.refuse(cause.into()))
}
pub(in crate::domain_computation::primary_graph) fn remove<K: Ord, V>(
    map: &mut Map<K, V>,
    key: &K,
    execution: &ExecutionRequest<'_, '_>,
    control: &Control<'_>,
) -> Result<(), Stop> {
    for _ in 0..4 {
        lookup(
            control,
            map.len(),
            size_of::<K>() + size_of::<V>() + 16 * size_of::<usize>(),
        )?;
    }
    map.remove_for_request(key, *execution)
        .map(|_| ())
        .map_err(|cause| control.refuse(cause.into()))
}
