use super::entry_measurement::EntryComparisonWidth;
use crate::canonicalization::basis::CanonicalBasisEntry;

/// Bounds the unchanged Rust 1.94.0 (4a4ef493e) stable sort. The recorded
/// canonical comparison count remains the count from the actual `sort_by`.
/// Source: rust-lang/rust, library/core/src/slice/sort/{stable,shared}.
pub(super) fn sorting_claim(length: usize, width: EntryComparisonWidth) -> Option<(usize, usize)> {
    let (comparisons, moves, scratch) = if length <= 20 {
        // stable/mod.rs selects insertion sort through 20 entries. Each
        // inserted tail compares/shifts at most its preceding prefix.
        let insertion_comparisons = length.checked_mul(length.saturating_sub(1))? / 2;
        let insertion_moves = insertion_comparisons.checked_add(length.checked_mul(2)?)?;
        // The same compiler's size-optimized std build instead selects tiny
        // mergesort. Cover both implementations without assuming std flags.
        let levels = if length < 2 {
            0
        } else {
            usize::BITS as usize - (length - 1).leading_zeros() as usize
        };
        let tiny_comparisons = length.checked_mul(levels)?;
        let tiny_moves = length
            .checked_mul(4)?
            .checked_mul(levels)?
            .checked_add(length.checked_mul(3)?)?;
        let scratch = if length < 2 {
            0
        } else {
            (length / 2)
                .checked_mul(size_of::<CanonicalBasisEntry>())?
                .checked_add(4096)?
        };
        (
            insertion_comparisons.max(tiny_comparisons),
            insertion_moves.max(tiny_moves),
            scratch,
        )
    } else {
        let logarithm = usize::BITS as usize - (length - 1).leading_zeros() as usize;
        let partition_levels = logarithm.checked_mul(2)?;
        let merge_levels = logarithm.checked_add(2)?;
        // drift.rs scans runs, then each entry participates in at most one
        // quicksort tree and one physical merge tree. quicksort.rs decrements
        // its 2*floor(log2(n)) depth limit on EVERY partition (not only bad
        // pivots). At one level: pivot sampling <= n, ancestor checks <= n,
        // and at most two partitions, each <= n comparisons.
        let partition_comparisons = length.checked_mul(4)?.checked_mul(partition_levels)?;
        // smallsort.rs leaves have <=32 entries; even insertion sorting each
        // leaf uses <=16 comparisons per entry. Eager fallback and outer
        // powersort each have <=ceil(log2(n))+2 merge levels, <=n comparisons
        // per level. Run discovery contributes at most 2n comparisons.
        let comparisons = partition_comparisons
            .checked_add(length.checked_mul(16)?)?
            .checked_add(length.checked_mul(2)?.checked_mul(merge_levels)?)?
            .checked_add(length.checked_mul(2)?)?;
        // Two partition passes copy to scratch and back; include pivot copies.
        // Small leaves include insertion shifts, scratch copies and merging.
        // Physical merge copies <=4n entries per level; two merge trees.
        let moves = length
            .checked_mul(6)?
            .checked_mul(partition_levels)?
            .checked_add(length.checked_mul(34)?)?
            .checked_add(length.checked_mul(8)?.checked_mul(merge_levels)?)?
            .checked_add(length.checked_mul(3)?)?;
        // stable/mod.rs uses max(ceil(n/2), min(n, 8MB/entry), 48)
        // entries, backed by 4096 stack bytes or one heap allocation. Reserve
        // both storage bounds conservatively before std owns their lifetime.
        let scratch = length
            .max(48)
            .checked_mul(size_of::<CanonicalBasisEntry>())?
            .checked_add(4096)?
            // Each drift frame owns 66 usize runs and 66 u8 depths. A
            // quicksort fallback's eager drift frame can coexist with the
            // outer frame; eager leaves never recurse into another fallback.
            .checked_add(
                66_usize
                    .checked_mul(size_of::<usize>().checked_add(1)?)?
                    .checked_mul(2)?,
            )?;
        (comparisons, moves, scratch)
    };
    let duplicate_checks = length.saturating_sub(1);
    let work = comparisons
        .checked_mul(width.entry.checked_mul(2)?)?
        .checked_add(moves.checked_mul(size_of::<CanonicalBasisEntry>())?)?
        .checked_add(
            duplicate_checks.checked_mul(
                width
                    .domain
                    .checked_add(width.locus)?
                    .checked_mul(2)?
                    .checked_add(3)?,
            )?,
        )?
        .checked_add(width.duplicate_clone_work)?;
    Some((work, scratch.checked_add(width.duplicate_clone_bytes)?))
}
