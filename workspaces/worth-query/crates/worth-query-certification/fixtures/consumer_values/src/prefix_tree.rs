//! A sorted, height-balanced prefix source; storage and source values stay borrowed.
use crate::PrefixOrder;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrefixIndexDenial {
    InvalidTopology,
}
impl std::fmt::Display for PrefixIndexDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("invalid finite prefix tree")
    }
}
impl std::error::Error for PrefixIndexDenial {}

#[derive(Clone, Copy, Debug)]
pub struct PrefixNode<Id> {
    pub source: usize,
    pub left: Option<Id>,
    pub right: Option<Id>,
    pub height: u8,
}

/// Borrowed finite node operations supplied by the prefix mechanism's caller.
pub trait PrefixBuildStorage {
    type Id: Copy;
    type Stop;
    fn checkpoint(&mut self) -> Result<(), Self::Stop>;
    fn compare_sources(&mut self, left: usize, right: usize) -> Result<Ordering, Self::Stop>;
    fn append(&mut self, node: PrefixNode<Self::Id>) -> Result<Self::Id, Self::Stop>;
    fn read(&mut self, id: Self::Id) -> Result<PrefixNode<Self::Id>, Self::Stop>;
    fn replace(&mut self, id: Self::Id, node: PrefixNode<Self::Id>) -> Result<(), Self::Stop>;
    fn invalid(&self) -> Self::Stop;
}

pub fn build_prefix_tree<S: PrefixBuildStorage>(
    source_count: usize,
    storage: &mut S,
) -> Result<Option<S::Id>, S::Stop> {
    let mut root = None;
    for source in 0..source_count {
        root = Some(insert(root, source, 0, storage)?);
    }
    Ok(root)
}

fn insert<S: PrefixBuildStorage>(
    root: Option<S::Id>,
    source: usize,
    depth: u32,
    storage: &mut S,
) -> Result<S::Id, S::Stop> {
    storage.checkpoint()?;
    if depth > 2 * usize::BITS {
        return Err(storage.invalid());
    }
    let Some(id) = root else {
        return storage.append(PrefixNode {
            source,
            left: None,
            right: None,
            height: 1,
        });
    };
    let mut node = storage.read(id)?;
    if storage.compare_sources(source, node.source)? == Ordering::Less {
        node.left = Some(insert(node.left, source, depth + 1, storage)?);
    } else {
        node.right = Some(insert(node.right, source, depth + 1, storage)?);
    }
    write_height(id, node, storage)?;
    balance(id, storage)
}

fn height<S: PrefixBuildStorage>(id: Option<S::Id>, storage: &mut S) -> Result<u8, S::Stop> {
    match id {
        Some(id) => storage.read(id).map(|node| node.height),
        None => Ok(0),
    }
}

fn write_height<S: PrefixBuildStorage>(
    id: S::Id,
    mut node: PrefixNode<S::Id>,
    storage: &mut S,
) -> Result<(), S::Stop> {
    node.height = height(node.left, storage)?
        .max(height(node.right, storage)?)
        .checked_add(1)
        .ok_or_else(|| storage.invalid())?;
    storage.replace(id, node)
}

fn balance<S: PrefixBuildStorage>(id: S::Id, storage: &mut S) -> Result<S::Id, S::Stop> {
    let mut node = storage.read(id)?;
    let left = height(node.left, storage)?;
    let right = height(node.right, storage)?;
    if i16::from(left) - i16::from(right) > 1 {
        let child_id = node.left.ok_or_else(|| storage.invalid())?;
        let child = storage.read(child_id)?;
        if height(child.left, storage)? < height(child.right, storage)? {
            node.left = Some(rotate_left(child_id, storage)?);
            write_height(id, node, storage)?;
        }
        rotate_right(id, storage)
    } else if i16::from(right) - i16::from(left) > 1 {
        let child_id = node.right.ok_or_else(|| storage.invalid())?;
        let child = storage.read(child_id)?;
        if height(child.right, storage)? < height(child.left, storage)? {
            node.right = Some(rotate_right(child_id, storage)?);
            write_height(id, node, storage)?;
        }
        rotate_left(id, storage)
    } else {
        Ok(id)
    }
}

fn rotate_left<S: PrefixBuildStorage>(id: S::Id, storage: &mut S) -> Result<S::Id, S::Stop> {
    let mut node = storage.read(id)?;
    let pivot_id = node.right.ok_or_else(|| storage.invalid())?;
    let mut pivot = storage.read(pivot_id)?;
    node.right = pivot.left;
    write_height(id, node, storage)?;
    pivot.left = Some(id);
    write_height(pivot_id, pivot, storage)?;
    Ok(pivot_id)
}

fn rotate_right<S: PrefixBuildStorage>(id: S::Id, storage: &mut S) -> Result<S::Id, S::Stop> {
    let mut node = storage.read(id)?;
    let pivot_id = node.left.ok_or_else(|| storage.invalid())?;
    let mut pivot = storage.read(pivot_id)?;
    node.left = pivot.right;
    write_height(id, node, storage)?;
    pivot.right = Some(id);
    write_height(pivot_id, pivot, storage)?;
    Ok(pivot_id)
}

/// Query operations resolve only original source ordinals, never copied values.
pub trait PrefixQueryStorage {
    type Id: Copy;
    type Stop;
    fn checkpoint(&mut self) -> Result<(), Self::Stop>;
    fn read(&mut self, id: Self::Id) -> Result<PrefixNode<Self::Id>, Self::Stop>;
    fn classify_source(&mut self, source: usize) -> Result<PrefixOrder, Self::Stop>;
    fn emit(&mut self, source: usize) -> Result<(), Self::Stop>;
    fn invalid(&self) -> Self::Stop;
}

pub fn query_prefix_tree<S: PrefixQueryStorage>(
    root: Option<S::Id>,
    node_count: usize,
    storage: &mut S,
) -> Result<(), S::Stop> {
    let mut remaining = node_count;
    if let Some(root) = root {
        visit(root, 0, &mut remaining, storage)
    } else if node_count == 0 {
        Ok(())
    } else {
        Err(storage.invalid())
    }
}

fn visit<S: PrefixQueryStorage>(
    id: S::Id,
    depth: u32,
    remaining: &mut usize,
    storage: &mut S,
) -> Result<(), S::Stop> {
    storage.checkpoint()?;
    if depth > 2 * usize::BITS || *remaining == 0 {
        return Err(storage.invalid());
    }
    *remaining -= 1;
    let node = storage.read(id)?;
    match storage.classify_source(node.source)? {
        PrefixOrder::Before => {
            if let Some(right) = node.right {
                visit(right, depth + 1, remaining, storage)?;
            }
        }
        PrefixOrder::After => {
            if let Some(left) = node.left {
                visit(left, depth + 1, remaining, storage)?;
            }
        }
        PrefixOrder::Matches => {
            if let Some(left) = node.left {
                visit(left, depth + 1, remaining, storage)?;
            }
            storage.emit(node.source)?;
            if let Some(right) = node.right {
                visit(right, depth + 1, remaining, storage)?;
            }
        }
    }
    Ok(())
}
