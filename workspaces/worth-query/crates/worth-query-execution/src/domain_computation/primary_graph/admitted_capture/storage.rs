//! Conservative physical backing for complete clones of standard ordered trees.
pub(in crate::domain_computation) fn tree_clone_bytes<T>(len: usize) -> Option<u64> {
    u64::try_from(len).ok()?.checked_mul(
        11_u64
            .checked_mul(size_of::<T>() as u64)?
            .checked_add(14 * size_of::<usize>() as u64)?,
    )
}
