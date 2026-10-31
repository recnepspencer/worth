//! Spatial AVL edits. Exact leaf bounds route edits and bound query traversal.
use super::{Box2, BoxNode, BoxNodeLinks};

/// The caller supplies stable occurrence keys, isolated backing and original stops.
/// Keys are not positions in a newly sorted or filtered Source image.
pub trait BoxEditStorage {
    type Id: Copy;
    type Source: Copy + Ord;
    type Stop;
    fn checkpoint(&mut self) -> Result<(), Self::Stop>;
    fn read(&mut self, id: Self::Id) -> Result<BoxNode<Self::Id, Self::Source>, Self::Stop>;
    fn append(&mut self, node: BoxNode<Self::Id, Self::Source>) -> Result<Self::Id, Self::Stop>;
    fn replace(
        &mut self,
        id: Self::Id,
        node: BoxNode<Self::Id, Self::Source>,
    ) -> Result<(), Self::Stop>;
    fn remove(&mut self, id: Self::Id) -> Result<(), Self::Stop>;
    /// Bounds at the start of this occurrence edit; no Source projection is needed.
    fn source_bounds(&mut self, source: Self::Source) -> Result<Box2, Self::Stop>;
    fn invalid(&self) -> Self::Stop;
}

pub fn insert_box_source<S: BoxEditStorage>(
    root: Option<S::Id>,
    source: S::Source,
    bounds: Box2,
    storage: &mut S,
) -> Result<S::Id, S::Stop> {
    insert(root, source, bounds, 0, storage)
}

pub fn remove_box_source<S: BoxEditStorage>(
    root: S::Id,
    source: S::Source,
    storage: &mut S,
) -> Result<Option<S::Id>, S::Stop> {
    let bounds = storage.source_bounds(source)?;
    remove(root, source, bounds, 0, storage)
}

/// A changed spatial key retires/reinserts the leaf on bounded AVL paths.
pub fn replace_box_source<S: BoxEditStorage>(
    root: S::Id,
    source: S::Source,
    bounds: Box2,
    storage: &mut S,
) -> Result<S::Id, S::Stop> {
    let before = storage.source_bounds(source)?;
    if before == bounds {
        return Ok(root);
    }
    let root = remove(root, source, before, 0, storage)?;
    insert(root, source, bounds, 0, storage)
}

fn leaf<Id, Source: Copy>(source: Source, bounds: Box2) -> BoxNode<Id, Source> {
    BoxNode {
        bounds,
        links: BoxNodeLinks::Leaf { source },
        last_source: source,
        last_bounds: bounds,
        height: 1,
    }
}

fn insert<S: BoxEditStorage>(
    root: Option<S::Id>,
    source: S::Source,
    bounds: Box2,
    depth: u32,
    storage: &mut S,
) -> Result<S::Id, S::Stop> {
    checkpoint(depth, storage)?;
    let Some(id) = root else {
        return storage.append(leaf(source, bounds));
    };
    let node = storage.read(id)?;
    let (left, right) = match node.links {
        BoxNodeLinks::Leaf { source: previous } => {
            if source == previous {
                return Err(storage.invalid());
            }
            let added = storage.append(leaf(source, bounds))?;
            let (left, right) = if key(bounds, source) < key(node.bounds, previous) {
                (added, id)
            } else {
                (id, added)
            };
            return append_branch(left, right, storage);
        }
        BoxNodeLinks::Branch { left, right } => {
            let left_node = storage.read(left)?;
            if key(bounds, source) <= key(left_node.last_bounds, left_node.last_source) {
                (
                    insert(Some(left), source, bounds, depth + 1, storage)?,
                    right,
                )
            } else {
                (
                    left,
                    insert(Some(right), source, bounds, depth + 1, storage)?,
                )
            }
        }
    };
    write_branch(id, left, right, storage)?;
    balance(id, storage)
}

fn remove<S: BoxEditStorage>(
    id: S::Id,
    source: S::Source,
    bounds: Box2,
    depth: u32,
    storage: &mut S,
) -> Result<Option<S::Id>, S::Stop> {
    checkpoint(depth, storage)?;
    let node = storage.read(id)?;
    let (left, right) = match node.links {
        BoxNodeLinks::Leaf { source: previous } => {
            if source != previous {
                return Err(storage.invalid());
            }
            storage.remove(id)?;
            return Ok(None);
        }
        BoxNodeLinks::Branch { left, right } => (left, right),
    };
    let left_node = storage.read(left)?;
    let (left, right) = if key(bounds, source) <= key(left_node.last_bounds, left_node.last_source)
    {
        (
            remove(left, source, bounds, depth + 1, storage)?,
            Some(right),
        )
    } else {
        (
            Some(left),
            remove(right, source, bounds, depth + 1, storage)?,
        )
    };
    match (left, right) {
        (Some(left), Some(right)) => {
            write_branch(id, left, right, storage)?;
            balance(id, storage).map(Some)
        }
        (None, sibling) | (sibling, None) => {
            storage.remove(id)?;
            Ok(sibling)
        }
    }
}

fn key<Source: Copy>(bounds: Box2, source: Source) -> ([i64; 4], Source) {
    let raw = bounds.coordinates();
    (
        [raw.lower[0], raw.lower[1], raw.upper[0], raw.upper[1]],
        source,
    )
}

fn checkpoint<S: BoxEditStorage>(depth: u32, storage: &mut S) -> Result<(), S::Stop> {
    storage.checkpoint()?;
    if depth > 2 * usize::BITS {
        Err(storage.invalid())
    } else {
        Ok(())
    }
}

fn branch<S: BoxEditStorage>(
    left: S::Id,
    right: S::Id,
    storage: &mut S,
) -> Result<BoxNode<S::Id, S::Source>, S::Stop> {
    let a = storage.read(left)?;
    let b = storage.read(right)?;
    if key(a.last_bounds, a.last_source) >= key(b.last_bounds, b.last_source) {
        return Err(storage.invalid());
    }
    let height = a
        .height
        .max(b.height)
        .checked_add(1)
        .ok_or_else(|| storage.invalid())?;
    Ok(BoxNode {
        bounds: a.bounds.union(b.bounds),
        links: BoxNodeLinks::Branch { left, right },
        last_source: b.last_source,
        last_bounds: b.last_bounds,
        height,
    })
}
fn append_branch<S: BoxEditStorage>(
    left: S::Id,
    right: S::Id,
    storage: &mut S,
) -> Result<S::Id, S::Stop> {
    let node = branch(left, right, storage)?;
    storage.append(node)
}
fn write_branch<S: BoxEditStorage>(
    id: S::Id,
    left: S::Id,
    right: S::Id,
    storage: &mut S,
) -> Result<(), S::Stop> {
    let node = branch(left, right, storage)?;
    storage.replace(id, node)
}

fn children<S: BoxEditStorage>(id: S::Id, storage: &mut S) -> Result<(S::Id, S::Id), S::Stop> {
    match storage.read(id)?.links {
        BoxNodeLinks::Branch { left, right } => Ok((left, right)),
        BoxNodeLinks::Leaf { .. } => Err(storage.invalid()),
    }
}

fn balance<S: BoxEditStorage>(id: S::Id, storage: &mut S) -> Result<S::Id, S::Stop> {
    let (left, right) = children(id, storage)?;
    let a = storage.read(left)?;
    let b = storage.read(right)?;
    if i16::from(a.height) - i16::from(b.height) > 1 {
        let (a_left, a_right) = children(left, storage)?;
        if storage.read(a_left)?.height < storage.read(a_right)?.height {
            let left = rotate_left(left, storage)?;
            write_branch(id, left, right, storage)?;
        }
        rotate_right(id, storage)
    } else if i16::from(b.height) - i16::from(a.height) > 1 {
        let (b_left, b_right) = children(right, storage)?;
        if storage.read(b_left)?.height > storage.read(b_right)?.height {
            let right = rotate_right(right, storage)?;
            write_branch(id, left, right, storage)?;
        }
        rotate_left(id, storage)
    } else {
        Ok(id)
    }
}
fn rotate_left<S: BoxEditStorage>(id: S::Id, storage: &mut S) -> Result<S::Id, S::Stop> {
    let (left, right) = children(id, storage)?;
    let (middle, far) = children(right, storage)?;
    write_branch(id, left, middle, storage)?;
    write_branch(right, id, far, storage)?;
    Ok(right)
}
fn rotate_right<S: BoxEditStorage>(id: S::Id, storage: &mut S) -> Result<S::Id, S::Stop> {
    let (left, right) = children(id, storage)?;
    let (far, middle) = children(left, storage)?;
    write_branch(id, middle, right, storage)?;
    write_branch(left, far, id, storage)?;
    Ok(left)
}
