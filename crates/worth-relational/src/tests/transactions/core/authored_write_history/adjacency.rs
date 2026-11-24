//! Exact endpoint occurrences, including empty-list entrants and authored no-ops.
use super::*;
use crate::facade::history::RelationalTouchAdjacencyDirection as Direction;
use crate::facade::identity::KindId;

mod audit;
mod paired;

fn key(anchor: EntityId, direction: Direction) -> RelationalAuthoredWriteSelector {
    RelationalAuthoredWriteSelector::Adjacency {
        anchor,
        kind: KindId(2),
        direction,
    }
}
