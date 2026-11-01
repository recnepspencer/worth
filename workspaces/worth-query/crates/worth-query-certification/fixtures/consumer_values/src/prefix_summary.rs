//! Exact lexical enclosure; Source coordinates retain distinct equal-name occurrences.
use crate::{prefix_order, PrefixOrder, PrefixSourceBounds};
use std::cmp::Ordering;

pub fn join_prefix_bounds<Source: Copy, E>(
    left: PrefixSourceBounds<Source>,
    right: PrefixSourceBounds<Source>,
    mut compare: impl FnMut(Source, Source) -> Result<Ordering, E>,
) -> Result<PrefixSourceBounds<Source>, E> {
    Ok(PrefixSourceBounds {
        lower: if compare(left.lower, right.lower)? == Ordering::Greater {
            right.lower
        } else {
            left.lower
        },
        upper: if compare(left.upper, right.upper)? == Ordering::Less {
            right.upper
        } else {
            left.upper
        },
    })
}
pub fn prefix_bounds_may_match<E>(
    lower: &str,
    upper: &str,
    prefix: &str,
    mut checkpoint: impl FnMut(u64) -> Result<(), E>,
) -> Result<bool, E> {
    Ok(
        prefix_order(lower, prefix, &mut checkpoint)? != PrefixOrder::After
            && prefix_order(upper, prefix, checkpoint)? != PrefixOrder::Before,
    )
}
