//! Prefix traversal over the single production caller-storage tree mechanic.
use crate::{PrefixNode, PrefixOrder};

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

/// Query operations resolve only owner-issued source occurrences, never copied values.
pub trait PrefixQueryStorage {
    type Id: Copy;
    type Source: Copy;
    type Stop;
    fn checkpoint(&mut self) -> Result<(), Self::Stop>;
    fn read(&mut self, id: Self::Id) -> Result<PrefixNode<Self::Id, Self::Source>, Self::Stop>;
    fn classify_source(&mut self, source: Self::Source) -> Result<PrefixOrder, Self::Stop>;
    fn emit(&mut self, source: Self::Source) -> Result<(), Self::Stop>;
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
