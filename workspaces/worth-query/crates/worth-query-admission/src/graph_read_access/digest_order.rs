use std::mem::size_of;

use crate::canonical_identity_derivation::WorthQueryCanonicalIdentityStop;

/// Sort owned values by their already rendered canonical keys. The keys are
/// prepared by their semantic owner; this routine only admits the comparison,
/// index-buffer and move work that the stable ordering performs.
pub(crate) fn sort_digest_keys_admitted<T, Stop>(
    mut keyed: Vec<(String, T)>,
    admit: &mut dyn FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<Vec<(String, T)>, WorthQueryCanonicalIdentityStop<Stop>> {
    admit(1, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
    let count = keyed.len();
    if count < 2 {
        return Ok(keyed);
    }
    let index_bytes = count
        .checked_mul(size_of::<usize>())
        .and_then(|bytes| bytes.checked_mul(2))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
    admit(index_bytes, index_bytes).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
    let mut order = Vec::new();
    order
        .try_reserve_exact(count)
        .map_err(|_| WorthQueryCanonicalIdentityStop::AllocationUnavailable)?;
    order.extend(0..count);
    let mut scratch = Vec::new();
    scratch
        .try_reserve_exact(count)
        .map_err(|_| WorthQueryCanonicalIdentityStop::AllocationUnavailable)?;
    scratch.resize(count, 0);

    let mut width = 1_usize;
    while width < count {
        let mut start = 0_usize;
        while start < count {
            let middle = start.saturating_add(width).min(count);
            let end = middle.saturating_add(width).min(count);
            let (mut left, mut right) = (start, middle);
            for slot in &mut scratch[start..end] {
                let take_left = if left == middle {
                    false
                } else if right == end {
                    true
                } else {
                    admit(2, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
                    let left_key = &keyed[order[left]].0;
                    let right_key = &keyed[order[right]].0;
                    let bytes = left_key
                        .len()
                        .checked_add(right_key.len())
                        .ok_or(WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
                    let work = bytes
                        .checked_add(1)
                        .and_then(|work| u64::try_from(work).ok())
                        .ok_or(WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
                    admit(work, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
                    left_key <= right_key
                };
                admit(size_of::<usize>() as u64, 0)
                    .map_err(WorthQueryCanonicalIdentityStop::Admission)?;
                *slot = if take_left {
                    let index = order[left];
                    left += 1;
                    index
                } else {
                    let index = order[right];
                    right += 1;
                    index
                };
            }
            start = end;
        }
        std::mem::swap(&mut order, &mut scratch);
        width = width.saturating_mul(2);
    }

    // `order[new] = old`; reuse the scratch buffer for the inverse mapping.
    // A cycle of swaps places each owned pair without constructing another
    // pair buffer or cloning any canonical authority.
    let inverse_writes = count
        .checked_mul(size_of::<usize>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
    admit(inverse_writes, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
    for (new, old) in order.into_iter().enumerate() {
        scratch[old] = new;
    }
    let swap_bytes = size_of::<(String, T)>()
        .checked_mul(3)
        .and_then(|bytes| size_of::<usize>().checked_mul(3)?.checked_add(bytes))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
    for index in 0..count {
        while scratch[index] != index {
            let other = scratch[index];
            admit(swap_bytes, 0).map_err(WorthQueryCanonicalIdentityStop::Admission)?;
            keyed.swap(index, other);
            scratch.swap(index, other);
        }
    }
    Ok(keyed)
}
