//! Retained canonical shape carries its full-build dependency span.
use super::*;
impl<T, F> ReductionTree<T, F> {
    /// Cost of a full build over this tree's identity set: the same sequential
    /// shape work and dependent evaluation path as the shipped constructor.
    pub fn checked_full_build_cost(&self) -> Option<(u64, u64)> {
        let spine = |left: bool| {
            std::iter::successors(self.root.as_deref(), |node| {
                if left {
                    node.left.as_deref()
                } else {
                    node.right.as_deref()
                }
            })
            .count()
        };
        let ends = u64::try_from(spine(true))
            .ok()?
            .checked_add(u64::try_from(spine(false)).ok()?)?;
        checked_cost(
            self.len,
            ends,
            self.root.as_ref().map_or(0, |root| root.full_subtree_span),
        )
    }
}
pub(super) fn checked_cost(count: usize, ends: u64, subtree_span: u64) -> Option<(u64, u64)> {
    let n = u64::try_from(count).ok()?;
    let work = n.checked_mul(9)?.checked_sub(ends)?;
    let span = n
        .checked_mul(5)?
        .checked_sub(ends)?
        .checked_add(subtree_span)?;
    Some((work, span))
}
