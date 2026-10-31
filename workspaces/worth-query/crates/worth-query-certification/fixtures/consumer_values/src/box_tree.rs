//! A balanced finite box tree; callers own storage, controls and source resolution.
use crate::{BoundsDenial, Box2};
mod edit;
mod order;
pub use edit::{insert_box_source, remove_box_source, replace_box_source, BoxEditStorage};
pub use order::compare_box_bounds;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoxIndexDenial {
    Bounds(BoundsDenial),
    InvalidTopology,
}
impl std::fmt::Display for BoxIndexDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bounds(cause) => std::fmt::Display::fmt(cause, formatter),
            Self::InvalidTopology => formatter.write_str("invalid finite box tree"),
        }
    }
}
impl std::error::Error for BoxIndexDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Bounds(cause) => Some(cause),
            Self::InvalidTopology => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BoxNodeLinks<Id, Source = usize> {
    Leaf { source: Source },
    Branch { left: Id, right: Id },
}

#[derive(Clone, Copy, Debug)]
pub struct BoxNode<Id, Source = usize> {
    pub bounds: Box2,
    pub links: BoxNodeLinks<Id, Source>,
    pub last_source: Source,
    pub last_bounds: Box2,
    pub height: u8,
}

/// Cold construction uses the same spatial ordering and edits as maintained roots.
pub fn build_box_tree<S: BoxEditStorage>(
    count: usize,
    mut source: impl FnMut(usize) -> Result<(S::Source, Box2), S::Stop>,
    storage: &mut S,
) -> Result<Option<S::Id>, S::Stop> {
    storage.checkpoint()?;
    let mut root = None;
    for ordinal in 0..count {
        let (source, bounds) = source(ordinal)?;
        root = Some(insert_box_source(root, source, bounds, storage)?);
    }
    Ok(root)
}

/// Emit only overlapping leaf ordinals. The source owner resolves those ordinals.
pub fn query_box_tree<Id: Copy, Source: Copy, E>(
    root: Option<Id>,
    node_count: usize,
    probe: Box2,
    mut read: impl FnMut(Id) -> Result<BoxNode<Id, Source>, E>,
    mut emit: impl FnMut(Source) -> Result<(), E>,
    mut checkpoint: impl FnMut(u64) -> Result<(), E>,
    invalid: impl Fn() -> E,
) -> Result<(), E> {
    let mut remaining = node_count;
    match root {
        None if node_count == 0 => Ok(()),
        None => Err(invalid()),
        Some(root) => visit(
            root,
            0,
            &mut remaining,
            probe,
            &mut read,
            &mut emit,
            &mut checkpoint,
            &invalid,
        ),
    }
}

fn visit<Id: Copy, Source: Copy, E>(
    id: Id,
    depth: u32,
    remaining: &mut usize,
    probe: Box2,
    read: &mut impl FnMut(Id) -> Result<BoxNode<Id, Source>, E>,
    emit: &mut impl FnMut(Source) -> Result<(), E>,
    checkpoint: &mut impl FnMut(u64) -> Result<(), E>,
    invalid: &impl Fn() -> E,
) -> Result<(), E> {
    checkpoint(1)?;
    // Cold splits and subsequent AVL edits both have logarithmic height.
    // This also prevents malformed cyclic backing from recursing without bound.
    if depth > 2 * usize::BITS || *remaining == 0 {
        return Err(invalid());
    }
    *remaining -= 1;
    let node = read(id)?;
    if !node.bounds.overlaps(probe) {
        return Ok(());
    }
    match node.links {
        BoxNodeLinks::Leaf { source } => emit(source),
        BoxNodeLinks::Branch { left, right } => {
            visit(
                left,
                depth + 1,
                remaining,
                probe,
                read,
                emit,
                checkpoint,
                invalid,
            )?;
            visit(
                right,
                depth + 1,
                remaining,
                probe,
                read,
                emit,
                checkpoint,
                invalid,
            )
        }
    }
}
