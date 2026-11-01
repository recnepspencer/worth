//! Spatial AVL edits. Exact leaf bounds route edits and bound query traversal.
use super::{order::key, Box2, BoxNode, BoxNodeLinks};

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
    insert(
        root,
        Incoming {
            source,
            bounds,
            reused: None,
        },
        0,
        storage,
    )
}

pub fn remove_box_source<S: BoxEditStorage>(
    root: S::Id,
    source: S::Source,
    storage: &mut S,
) -> Result<Option<S::Id>, S::Stop> {
    let bounds = storage.source_bounds(source)?;
    let detached = extract(root, source, bounds, 0, storage)?;
    storage.remove(detached.leaf)?;
    if let Some(parent) = detached.parent {
        storage.remove(parent)?;
    }
    Ok(detached.root)
}

/// A changed spatial key relocates actual live cells on bounded AVL paths.
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
    let detached = extract(root, source, before, 0, storage)?;
    storage.replace(detached.leaf, leaf(source, bounds))?;
    match (detached.root, detached.parent) {
        (None, None) => Ok(detached.leaf),
        (Some(root), Some(parent)) => insert(
            Some(root),
            Incoming {
                source,
                bounds,
                reused: Some((detached.leaf, parent)),
            },
            0,
            storage,
        ),
        _ => Err(storage.invalid()),
    }
}
// Consumed along one insertion path; reused cells were detached, never retired.
struct Incoming<Id, Source> {
    source: Source,
    bounds: Box2,
    reused: Option<(Id, Id)>,
}
struct Detached<Id> {
    root: Option<Id>,
    leaf: Id,
    parent: Option<Id>,
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
    incoming: Incoming<S::Id, S::Source>,
    depth: u32,
    storage: &mut S,
) -> Result<S::Id, S::Stop> {
    checkpoint(depth, storage)?;
    let (source, bounds) = (incoming.source, incoming.bounds);
    let Some(id) = root else {
        if incoming.reused.is_some() {
            return Err(storage.invalid());
        }
        return storage.append(leaf(source, bounds));
    };
    let node = storage.read(id)?;
    let (left, right) = match node.links {
        BoxNodeLinks::Leaf { source: previous } => {
            if source == previous {
                return Err(storage.invalid());
            }
            let (added, parent) = match incoming.reused {
                Some((leaf, parent)) => (leaf, Some(parent)),
                None => (storage.append(leaf(source, bounds))?, None),
            };
            let (left, right) = if key(bounds, source) < key(node.bounds, previous) {
                (added, id)
            } else {
                (id, added)
            };
            return match parent {
                Some(parent) => {
                    write_branch(parent, left, right, storage)?;
                    Ok(parent)
                }
                None => append_branch(left, right, storage),
            };
        }
        BoxNodeLinks::Branch { left, right } => {
            let left_node = storage.read(left)?;
            if key(bounds, source) <= key(left_node.last_bounds, left_node.last_source) {
                (insert(Some(left), incoming, depth + 1, storage)?, right)
            } else {
                (left, insert(Some(right), incoming, depth + 1, storage)?)
            }
        }
    };
    let children = write_branch(id, left, right, storage)?;
    balance(id, children, storage)
}

fn extract<S: BoxEditStorage>(
    id: S::Id,
    source: S::Source,
    bounds: Box2,
    depth: u32,
    storage: &mut S,
) -> Result<Detached<S::Id>, S::Stop> {
    checkpoint(depth, storage)?;
    let node = storage.read(id)?;
    let (left, right) = match node.links {
        BoxNodeLinks::Leaf { source: previous } => {
            if source != previous {
                return Err(storage.invalid());
            }
            return Ok(Detached {
                root: None,
                leaf: id,
                parent: None,
            });
        }
        BoxNodeLinks::Branch { left, right } => (left, right),
    };
    let left_node = storage.read(left)?;
    let (mut detached, left, right) =
        if key(bounds, source) <= key(left_node.last_bounds, left_node.last_source) {
            let detached = extract(left, source, bounds, depth + 1, storage)?;
            let remaining = detached.root;
            (detached, remaining, Some(right))
        } else {
            let detached = extract(right, source, bounds, depth + 1, storage)?;
            let remaining = detached.root;
            (detached, Some(left), remaining)
        };
    detached.root = match (left, right) {
        (Some(left), Some(right)) => {
            let children = write_branch(id, left, right, storage)?;
            Some(balance(id, children, storage)?)
        }
        (None, sibling) | (sibling, None) => {
            debug_assert!(detached.parent.is_none());
            detached.parent = Some(id);
            sibling
        }
    };
    Ok(detached)
}

fn checkpoint<S: BoxEditStorage>(depth: u32, storage: &mut S) -> Result<(), S::Stop> {
    storage.checkpoint()?;
    if depth > 2 * usize::BITS {
        Err(storage.invalid())
    } else {
        Ok(())
    }
}

struct Branch<Id, Source> {
    node: BoxNode<Id, Source>,
    children: [BoxNode<Id, Source>; 2],
}

fn branch<S: BoxEditStorage>(
    left: S::Id,
    right: S::Id,
    storage: &mut S,
) -> Result<Branch<S::Id, S::Source>, S::Stop> {
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
    Ok(Branch {
        node: BoxNode {
            bounds: a.bounds.union(b.bounds),
            links: BoxNodeLinks::Branch { left, right },
            last_source: b.last_source,
            last_bounds: b.last_bounds,
            height,
        },
        children: [a, b],
    })
}
fn append_branch<S: BoxEditStorage>(
    left: S::Id,
    right: S::Id,
    storage: &mut S,
) -> Result<S::Id, S::Stop> {
    let branch = branch(left, right, storage)?;
    storage.append(branch.node)
}
fn write_branch<S: BoxEditStorage>(
    id: S::Id,
    left: S::Id,
    right: S::Id,
    storage: &mut S,
) -> Result<Branch<S::Id, S::Source>, S::Stop> {
    let branch = branch(left, right, storage)?;
    storage.replace(id, branch.node)?;
    Ok(branch)
}

fn children<S: BoxEditStorage>(id: S::Id, storage: &mut S) -> Result<(S::Id, S::Id), S::Stop> {
    match storage.read(id)?.links {
        BoxNodeLinks::Branch { left, right } => Ok((left, right)),
        BoxNodeLinks::Leaf { .. } => Err(storage.invalid()),
    }
}

/// Consume the actual child values used by the immediately preceding branch write.
/// The exclusive storage borrow has not changed either child in between.
fn balance<S: BoxEditStorage>(
    id: S::Id,
    branch: Branch<S::Id, S::Source>,
    storage: &mut S,
) -> Result<S::Id, S::Stop> {
    let BoxNodeLinks::Branch { left, right } = branch.node.links else {
        return Err(storage.invalid());
    };
    let [a, b] = branch.children;
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
