use std::{collections::BTreeMap, mem::size_of, sync::Arc};

use worth_execution::MapKernelFailure;

use crate::execution::{PacketBudgetDenial, PacketKernelContext};
use crate::indexes::data::{DerivedIndexEntryMap, DerivedIndexRows};

pub(super) type IndexKernelStop = MapKernelFailure<PacketBudgetDenial>;

pub(super) fn checked_push<K: Ord, R>(
    entries: &mut BTreeMap<K, Vec<R>>,
    key: K,
    row: R,
    key_owned_bytes: u64,
    row_owned_bytes: u64,
    context: &mut PacketKernelContext<'_, '_, '_>,
) -> Result<(), IndexKernelStop> {
    context.checkpoint(1 + usize::BITS as u64 - entries.len().max(1).leading_zeros() as u64)?;
    let new_key = !entries.contains_key(&key);
    let raw_key = if new_key {
        (size_of::<(K, Vec<R>)>() + 3 * size_of::<usize>()) as u64 + key_owned_bytes
    } else {
        0
    };
    let final_key = if new_key {
        (size_of::<(Arc<K>, DerivedIndexRows<R>)>() + size_of::<K>() + 2 * size_of::<usize>())
            as u64
            + key_owned_bytes
    } else {
        0
    };
    context.claim_scratch(
        raw_key
            .saturating_add(size_of::<R>() as u64)
            .saturating_add(row_owned_bytes),
    )?;
    context.claim_result(
        final_key
            .saturating_add((size_of::<R>() + size_of::<Arc<R>>() + 2 * size_of::<usize>()) as u64)
            .saturating_add(row_owned_bytes),
    )?;
    let rows = entries.entry(key).or_default();
    rows.try_reserve_exact(1)
        .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    rows.push(row);
    Ok(())
}

pub(in crate::indexes) fn checked_finish<K: Ord + Clone, R: Clone>(
    entries: BTreeMap<K, Vec<R>>,
    context: &mut PacketKernelContext<'_, '_, '_>,
) -> Result<DerivedIndexEntryMap<K, R>, IndexKernelStop> {
    let mut output = DerivedIndexEntryMap::default();
    for (key, rows) in entries {
        context.checkpoint(1)?;
        let mut converted = DerivedIndexRows::default();
        for row in rows {
            context.checkpoint(1)?;
            converted.insert(converted.len(), row);
        }
        output.replace(key, converted);
    }
    Ok(output)
}
