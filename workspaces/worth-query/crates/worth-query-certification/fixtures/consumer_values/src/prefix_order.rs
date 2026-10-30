//! Unnormalized UTF-8 prefix semantics, shared by the sorted-tree mechanism.
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrefixOrder {
    Before,
    Matches,
    After,
}

/// Exact UTF-8 byte order, then the source's deterministic key for text ties.
pub fn compare_prefix_keys<E>(
    left: &str,
    left_key: u64,
    right: &str,
    right_key: u64,
    mut checkpoint: impl FnMut(u64) -> Result<(), E>,
) -> Result<Ordering, E> {
    for (left, right) in left.bytes().zip(right.bytes()) {
        checkpoint(1)?;
        let order = left.cmp(&right);
        if order != Ordering::Equal {
            return Ok(order);
        }
    }
    checkpoint(1)?;
    Ok(left
        .len()
        .cmp(&right.len())
        .then_with(|| left_key.cmp(&right_key)))
}

/// This is `starts_with` without normalization, case folding or allocation.
pub fn prefix_order<E>(
    key: &str,
    prefix: &str,
    mut checkpoint: impl FnMut(u64) -> Result<(), E>,
) -> Result<PrefixOrder, E> {
    for (index, prefix_byte) in prefix.bytes().enumerate() {
        checkpoint(1)?;
        let Some(key_byte) = key.as_bytes().get(index) else {
            return Ok(PrefixOrder::Before);
        };
        match key_byte.cmp(&prefix_byte) {
            Ordering::Less => return Ok(PrefixOrder::Before),
            Ordering::Greater => return Ok(PrefixOrder::After),
            Ordering::Equal => {}
        }
    }
    Ok(PrefixOrder::Matches)
}
