//! Checked single-pass construction from an already comparator-ordered sequence.
use super::{
    ExecutionOrderedTreeBounds as Bounds, ExecutionOrderedTreeNode as Node,
    ExecutionOrderedTreeStorage as Storage,
};
use std::cmp::Ordering;

struct Subtree<Id, Source> {
    id: Id,
    height: u8,
    bounds: Bounds<Source>,
}

/// Build a balanced finite tree with one append per source and no storage reads.
/// Input must contain exactly `count` strictly increasing sources. Both input
/// and storage failures propagate; failed staged construction is not publishable.
pub fn build_sorted_ordered_tree<S: Storage>(
    count: usize,
    sources: impl IntoIterator<Item = Result<S::Source, S::Stop>>,
    storage: &mut S,
) -> Result<Option<S::Id>, S::Stop> {
    let mut sources = sources.into_iter();
    let root = build(count, &mut sources, &mut None, storage)?;
    if let Some(extra) = sources.next() {
        extra?;
        return Err(storage.invalid());
    }
    storage.checkpoint()?;
    Ok(root.map(|root| root.id))
}

fn build<S: Storage>(
    count: usize,
    sources: &mut impl Iterator<Item = Result<S::Source, S::Stop>>,
    previous: &mut Option<S::Source>,
    storage: &mut S,
) -> Result<Option<Subtree<S::Id, S::Source>>, S::Stop> {
    storage.checkpoint()?;
    if count == 0 {
        return Ok(None);
    }
    let left = build(count / 2, sources, previous, storage)?;
    let source = sources.next().ok_or_else(|| storage.invalid())??;
    if let Some(previous) = *previous
        && storage.compare_sources(previous, source)? != Ordering::Less
    {
        return Err(storage.invalid());
    }
    *previous = Some(source);
    let right = build(count - count / 2 - 1, sources, previous, storage)?;
    let height = 1 + left
        .as_ref()
        .map_or(0, |tree| tree.height)
        .max(right.as_ref().map_or(0, |tree| tree.height));
    let bounds = Bounds {
        lower: left.as_ref().map_or(source, |tree| tree.bounds.lower),
        upper: right.as_ref().map_or(source, |tree| tree.bounds.upper),
    };
    let id = storage.append(Node {
        source,
        left: left.map(|tree| tree.id),
        right: right.map(|tree| tree.id),
        height,
        bounds,
    })?;
    Ok(Some(Subtree { id, height, bounds }))
}
