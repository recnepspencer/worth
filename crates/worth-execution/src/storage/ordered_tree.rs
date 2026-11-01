//! Height-balanced finite tree over caller-owned storage and comparison.
//! The caller supplies isolated staged storage, validates its own coordinates,
//! and admits every comparison/read/write through its original control. Failed
//! edits may have changed staged storage; only the caller can publish or discard.
use std::cmp::Ordering;
mod sorted;
pub use sorted::build_sorted_ordered_tree;

/// Comparator-ordered extrema carried by one finite subtree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionOrderedTreeBounds<Source> {
    pub lower: Source,
    pub upper: Source,
}
impl<Source: Copy> ExecutionOrderedTreeBounds<Source> {
    pub const fn point(source: Source) -> Self {
        Self {
            lower: source,
            upper: source,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ExecutionOrderedTreeNode<Id, Source = usize> {
    pub source: Source,
    pub bounds: ExecutionOrderedTreeBounds<Source>,
    pub left: Option<Id>,
    pub right: Option<Id>,
    pub height: u8,
}

/// Borrowed finite node operations supplied by the ordered tree mechanism's caller.
pub trait ExecutionOrderedTreeStorage {
    type Id: Copy;
    type Source: Copy;
    type Stop;
    fn checkpoint(&mut self) -> Result<(), Self::Stop>;
    fn compare_sources(
        &mut self,
        left: Self::Source,
        right: Self::Source,
    ) -> Result<Ordering, Self::Stop>;
    fn append(
        &mut self,
        node: ExecutionOrderedTreeNode<Self::Id, Self::Source>,
    ) -> Result<Self::Id, Self::Stop>;
    fn read(
        &mut self,
        id: Self::Id,
    ) -> Result<ExecutionOrderedTreeNode<Self::Id, Self::Source>, Self::Stop>;
    fn replace(
        &mut self,
        id: Self::Id,
        node: ExecutionOrderedTreeNode<Self::Id, Self::Source>,
    ) -> Result<(), Self::Stop>;
    fn remove(&mut self, id: Self::Id) -> Result<(), Self::Stop>;
    fn invalid(&self) -> Self::Stop;
}

pub fn build_ordered_tree<S: ExecutionOrderedTreeStorage>(
    sources: impl IntoIterator<Item = S::Source>,
    storage: &mut S,
) -> Result<Option<S::Id>, S::Stop> {
    let mut root = None;
    for source in sources {
        root = Some(insert(root, source, 0, storage)?);
    }
    Ok(root)
}

/// Insert one stable source occurrence. The comparator orders the value and
/// breaks ties by occurrence identity, so distinct equal values remain distinct.
/// The caller owns isolated storage; an interrupted edit is never published.
pub fn insert_ordered_tree<S: ExecutionOrderedTreeStorage>(
    root: Option<S::Id>,
    source: S::Source,
    storage: &mut S,
) -> Result<S::Id, S::Stop> {
    insert(root, source, 0, storage)
}

/// Remove using the predecessor's values, before a renamed source is inserted
/// using the proposed values. No source roster is scanned or renumbered.
pub fn remove_ordered_tree<S: ExecutionOrderedTreeStorage>(
    root: Option<S::Id>,
    source: S::Source,
    storage: &mut S,
) -> Result<Option<S::Id>, S::Stop> {
    remove(root, source, 0, storage)
}

fn remove<S: ExecutionOrderedTreeStorage>(
    root: Option<S::Id>,
    source: S::Source,
    depth: u32,
    storage: &mut S,
) -> Result<Option<S::Id>, S::Stop> {
    storage.checkpoint()?;
    if depth > 2 * usize::BITS {
        return Err(storage.invalid());
    }
    let id = root.ok_or_else(|| storage.invalid())?;
    let mut node = storage.read(id)?;
    match storage.compare_sources(source, node.source)? {
        Ordering::Less => node.left = remove(node.left, source, depth + 1, storage)?,
        Ordering::Greater => node.right = remove(node.right, source, depth + 1, storage)?,
        Ordering::Equal => match (node.left, node.right) {
            (None, child) | (child, None) => {
                storage.remove(id)?;
                return Ok(child);
            }
            (Some(_), Some(right)) => {
                let (source, right) = remove_minimum(right, depth + 1, storage)?;
                node.source = source;
                node.right = right;
            }
        },
    }
    refresh_node(id, node, storage)?;
    balance(id, storage).map(Some)
}

fn remove_minimum<S: ExecutionOrderedTreeStorage>(
    id: S::Id,
    depth: u32,
    storage: &mut S,
) -> Result<(S::Source, Option<S::Id>), S::Stop> {
    storage.checkpoint()?;
    if depth > 2 * usize::BITS {
        return Err(storage.invalid());
    }
    let mut node = storage.read(id)?;
    let Some(left) = node.left else {
        storage.remove(id)?;
        return Ok((node.source, node.right));
    };
    let (source, left) = remove_minimum(left, depth + 1, storage)?;
    node.left = left;
    refresh_node(id, node, storage)?;
    Ok((source, Some(balance(id, storage)?)))
}

fn insert<S: ExecutionOrderedTreeStorage>(
    root: Option<S::Id>,
    source: S::Source,
    depth: u32,
    storage: &mut S,
) -> Result<S::Id, S::Stop> {
    storage.checkpoint()?;
    if depth > 2 * usize::BITS {
        return Err(storage.invalid());
    }
    let Some(id) = root else {
        return storage.append(ExecutionOrderedTreeNode {
            source,
            bounds: ExecutionOrderedTreeBounds::point(source),
            left: None,
            right: None,
            height: 1,
        });
    };
    let mut node = storage.read(id)?;
    match storage.compare_sources(source, node.source)? {
        Ordering::Less => node.left = Some(insert(node.left, source, depth + 1, storage)?),
        Ordering::Greater => node.right = Some(insert(node.right, source, depth + 1, storage)?),
        Ordering::Equal => return Err(storage.invalid()),
    }
    refresh_node(id, node, storage)?;
    balance(id, storage)
}

fn height<S: ExecutionOrderedTreeStorage>(
    id: Option<S::Id>,
    storage: &mut S,
) -> Result<u8, S::Stop> {
    match id {
        Some(id) => storage.read(id).map(|node| node.height),
        None => Ok(0),
    }
}

fn refresh_node<S: ExecutionOrderedTreeStorage>(
    id: S::Id,
    mut node: ExecutionOrderedTreeNode<S::Id, S::Source>,
    storage: &mut S,
) -> Result<(), S::Stop> {
    node.bounds = ExecutionOrderedTreeBounds {
        lower: match node.left {
            Some(id) => storage.read(id)?.bounds.lower,
            None => node.source,
        },
        upper: match node.right {
            Some(id) => storage.read(id)?.bounds.upper,
            None => node.source,
        },
    };
    node.height = height(node.left, storage)?
        .max(height(node.right, storage)?)
        .checked_add(1)
        .ok_or_else(|| storage.invalid())?;
    storage.replace(id, node)
}

fn balance<S: ExecutionOrderedTreeStorage>(id: S::Id, storage: &mut S) -> Result<S::Id, S::Stop> {
    let mut node = storage.read(id)?;
    let left = height(node.left, storage)?;
    let right = height(node.right, storage)?;
    if i16::from(left) - i16::from(right) > 1 {
        let child_id = node.left.ok_or_else(|| storage.invalid())?;
        let child = storage.read(child_id)?;
        if height(child.left, storage)? < height(child.right, storage)? {
            node.left = Some(rotate_left(child_id, storage)?);
            refresh_node(id, node, storage)?;
        }
        rotate_right(id, storage)
    } else if i16::from(right) - i16::from(left) > 1 {
        let child_id = node.right.ok_or_else(|| storage.invalid())?;
        let child = storage.read(child_id)?;
        if height(child.right, storage)? < height(child.left, storage)? {
            node.right = Some(rotate_right(child_id, storage)?);
            refresh_node(id, node, storage)?;
        }
        rotate_left(id, storage)
    } else {
        Ok(id)
    }
}

fn rotate_left<S: ExecutionOrderedTreeStorage>(
    id: S::Id,
    storage: &mut S,
) -> Result<S::Id, S::Stop> {
    let mut node = storage.read(id)?;
    let pivot_id = node.right.ok_or_else(|| storage.invalid())?;
    let mut pivot = storage.read(pivot_id)?;
    node.right = pivot.left;
    refresh_node(id, node, storage)?;
    pivot.left = Some(id);
    refresh_node(pivot_id, pivot, storage)?;
    Ok(pivot_id)
}

fn rotate_right<S: ExecutionOrderedTreeStorage>(
    id: S::Id,
    storage: &mut S,
) -> Result<S::Id, S::Stop> {
    let mut node = storage.read(id)?;
    let pivot_id = node.left.ok_or_else(|| storage.invalid())?;
    let mut pivot = storage.read(pivot_id)?;
    node.left = pivot.right;
    refresh_node(id, node, storage)?;
    pivot.right = Some(id);
    refresh_node(pivot_id, pivot, storage)?;
    Ok(pivot_id)
}

#[cfg(test)]
mod tests;
