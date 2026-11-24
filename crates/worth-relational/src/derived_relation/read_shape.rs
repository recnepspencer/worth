//! Structural graph reads declare the traversal they require.

/// The complete source interpretation admitted by a genuine structural view.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DerivedRelationReadShape {
    Adjacency { reverse: bool },
    Reachable { reverse: bool },
    Component,
}
