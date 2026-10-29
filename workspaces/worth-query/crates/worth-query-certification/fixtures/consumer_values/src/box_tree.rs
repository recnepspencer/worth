//! A balanced finite box tree; callers own storage, controls and source resolution.
use crate::{BoundsDenial, Box2};
mod edit;
pub use edit::{insert_box_source, remove_box_source, replace_box_source, BoxEditStorage};

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
    pub height: u8,
}

/// Every source becomes one leaf; adjacent source spans form exact enclosing nodes.
pub fn build_box_tree<Id: Copy, E>(
    count: usize,
    mut source: impl FnMut(usize) -> Result<Box2, E>,
    mut append: impl FnMut(BoxNode<Id>) -> Result<Id, E>,
    mut checkpoint: impl FnMut(u64) -> Result<(), E>,
) -> Result<Option<Id>, E> {
    if count == 0 {
        return Ok(None);
    }
    build_span(0, count, &mut source, &mut append, &mut checkpoint).map(|(id, _, _)| Some(id))
}

fn build_span<Id: Copy, E>(
    start: usize,
    end: usize,
    source: &mut impl FnMut(usize) -> Result<Box2, E>,
    append: &mut impl FnMut(BoxNode<Id>) -> Result<Id, E>,
    checkpoint: &mut impl FnMut(u64) -> Result<(), E>,
) -> Result<(Id, Box2, u8), E> {
    checkpoint(1)?;
    if end - start == 1 {
        let bounds = source(start)?;
        return append(BoxNode {
            bounds,
            links: BoxNodeLinks::Leaf { source: start },
            last_source: start,
            height: 1,
        })
        .map(|id| (id, bounds, 1));
    }
    let middle = start + (end - start) / 2;
    let (left, left_bounds, left_height) = build_span(start, middle, source, append, checkpoint)?;
    let (right, right_bounds, right_height) = build_span(middle, end, source, append, checkpoint)?;
    let bounds = left_bounds.union(right_bounds);
    // Balanced addressable spans have height at most usize::BITS + 1.
    let height = left_height.max(right_height) + 1;
    append(BoxNode {
        bounds,
        links: BoxNodeLinks::Branch { left, right },
        last_source: end - 1,
        height,
    })
    .map(|id| (id, bounds, height))
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
