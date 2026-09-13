pub(super) fn exact_contiguous_blocks(blocks: &mut [u64], first: u64, next: u64) -> bool {
    let Some(expected) = next.checked_sub(first) else {
        return false;
    };
    if blocks.len() as u64 != expected {
        return false;
    }
    blocks.sort_unstable();
    blocks
        .iter()
        .copied()
        .zip(first..next)
        .all(|(found, expected)| found == expected)
}
