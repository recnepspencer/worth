use super::BridgeRetentionDenial as Denial;

pub(in crate::conditional_execution) fn sum(parts: &[u64]) -> Result<u64, Denial> {
    parts.iter().try_fold(0u64, |total, part| {
        total.checked_add(*part).ok_or(Denial::BytesExhausted)
    })
}

pub(in crate::conditional_execution) fn array_charge<T>(count: usize) -> Result<u64, Denial> {
    count
        .checked_mul(std::mem::size_of::<T>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Denial::BytesExhausted)
}

pub(in crate::conditional_execution) fn arc_charge<T>() -> Result<u64, Denial> {
    arc_layout(std::alloc::Layout::new::<T>())
}

pub(in crate::conditional_execution) fn arc_slice_charge<T>(count: usize) -> Result<u64, Denial> {
    arc_layout(std::alloc::Layout::array::<T>(count).map_err(|_| Denial::BytesExhausted)?)
}

pub(in crate::conditional_execution) fn arc_value_charge<T: ?Sized>(
    value: &T,
) -> Result<u64, Denial> {
    arc_layout(std::alloc::Layout::for_value(value))
}

fn arc_layout(value: std::alloc::Layout) -> Result<u64, Denial> {
    // ArcInner contains the two atomic reference counts followed by the value.
    let (layout, _) = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(value)
        .map_err(|_| Denial::BytesExhausted)?;
    u64::try_from(layout.pad_to_align().size()).map_err(|_| Denial::BytesExhausted)
}

/// Conservative Rust 1.94 BTreeMap representation bound, excluding allocator
/// bookkeeping/RSS: 11 entries, 12 child pointers, parent and two u16 fields.
/// One node per entry plus a retained empty root bounds every occupancy.
pub(in crate::conditional_execution) fn btree_charge<K, V>(entries: usize) -> Result<u64, Denial> {
    let alignment = std::mem::align_of::<K>()
        .max(std::mem::align_of::<V>())
        .max(std::mem::align_of::<usize>());
    sum(&[
        array_charge::<K>(11)?,
        array_charge::<V>(11)?,
        array_charge::<usize>(13)?,
        array_charge::<u16>(2)?,
        array_charge::<u8>(alignment.checked_mul(5).ok_or(Denial::BytesExhausted)?)?,
    ])?
    .checked_mul(
        u64::try_from(entries.checked_add(1).ok_or(Denial::BytesExhausted)?)
            .map_err(|_| Denial::BytesExhausted)?,
    )
    .ok_or(Denial::BytesExhausted)
}
