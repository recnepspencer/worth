//! Bounded key navigation for the exact admitted index read.

use super::{width, Stop};

pub(super) fn btree_navigation_work<E>(entries: usize, text_bytes: usize) -> Result<u64, Stop<E>> {
    let levels = usize::BITS
        .checked_sub(entries.leading_zeros())
        .ok_or(Stop::AccountingOverflow)?;
    let entry_count = width(entries)?;
    width(levels as usize)?
        .checked_mul(11)
        // One std BTreeMap get cannot compare more stored keys than exist.
        .map(|comparisons| comparisons.min(entry_count))
        .and_then(|n| n.checked_mul(u64::try_from(text_bytes).ok()?.checked_add(1)?))
        .and_then(|n| n.checked_add(1))
        .ok_or(Stop::AccountingOverflow)
}

pub(super) fn ordered_navigation_work<E>(entries: usize) -> Result<u64, Stop<E>> {
    let mut levels = 1_usize;
    let mut threshold = 63_usize;
    while entries >= threshold {
        levels = levels.checked_add(1).ok_or(Stop::AccountingOverflow)?;
        threshold = match threshold
            .checked_add(1)
            .and_then(|n| n.checked_mul(32))
            .and_then(|n| n.checked_sub(1))
        {
            Some(next) => next,
            None => break,
        };
    }
    let slots = entries.min(64);
    let comparisons = usize::BITS as usize - slots.leading_zeros() as usize + 1;
    // im::OrdMap can repeat one comparison within a node. Preserve the
    // two-operand factor and height while capping comparisons by map size.
    width(
        comparisons
            .checked_add(1)
            .ok_or(Stop::AccountingOverflow)?
            .min(entries),
    )?
    .checked_mul(2)
    .and_then(|n| n.checked_mul(u64::try_from(levels).ok()?))
    .ok_or(Stop::AccountingOverflow)
}
