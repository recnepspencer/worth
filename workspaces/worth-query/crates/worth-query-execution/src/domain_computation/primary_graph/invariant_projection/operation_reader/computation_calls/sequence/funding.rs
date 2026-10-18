//! Complete conservative bounds for each immutable call-sequence view.
use super::*;
fn im_node<T>() -> Option<u64> {
    u64::try_from(
        size_of::<T>()
            .checked_mul(64)?
            .checked_add(65 * size_of::<Option<Arc<()>>>())?
            .checked_add(6 * size_of::<usize>())?
            .checked_add(4 * align_of::<T>().max(align_of::<usize>()))?,
    )
    .ok()
}
pub(super) fn bound(calls: usize, entities: usize, pairs: usize, payload: u64) -> Option<u64> {
    let tree = (calls.checked_mul(2)?.checked_add(1)? as u64)
        .checked_mul((size_of::<Node>() + 2 * size_of::<usize>()) as u64)?;
    let reached = im_node::<(EntityId, im::OrdSet<usize>)>()?
        .checked_mul(entities.checked_add(1)? as u64)?
        .checked_add(
            im_node::<usize>()?.checked_mul(pairs.checked_add(entities)?.checked_add(1)? as u64)?,
        )?;
    (size_of::<ComputationCallSequence>() as u64)
        .checked_add(4 * size_of::<usize>() as u64)?
        .checked_add(tree)?
        .checked_add(reached)?
        .checked_add(payload)
}
pub(super) fn shape_bound(calls: usize) -> Option<u64> {
    (size_of::<CallShape>() as u64)
        .checked_add(2 * size_of::<usize>() as u64)?
        .checked_add((calls as u64).checked_mul(
            (11 * size_of::<(ComputationRead, usize)>() + 16 * size_of::<usize>()) as u64,
        )?)
}
