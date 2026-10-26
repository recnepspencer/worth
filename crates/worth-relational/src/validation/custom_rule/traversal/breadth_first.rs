//! Native frontier mechanics shared by bounded scope and complete reads.
use std::collections::{BTreeSet, VecDeque};

use crate::identity::data::EntityId;

pub(super) trait BreadthFirstRead {
    type Error;

    fn seed(&mut self, entity: EntityId) -> Result<(), Self::Error>;

    /// Admit retained frontier/visited storage before either collection grows.
    fn retain(&mut self, entity: EntityId, distance: usize) -> Result<(), Self::Error>;

    fn expand(
        &mut self,
        entity: EntityId,
        distance: usize,
        enqueue: impl FnMut(&mut Self, EntityId) -> Result<(), Self::Error>,
    ) -> Result<(), Self::Error>;
}

/// O((v + e) log v) frontier work, O(v) scratch. Policies own read/retention
/// admission; the driver never constructs another request or allowance.
pub(super) fn walk<Read: BreadthFirstRead>(
    seeds: &[EntityId],
    read: &mut Read,
) -> Result<BTreeSet<EntityId>, Read::Error> {
    let mut visited = BTreeSet::new();
    let mut frontier = VecDeque::new();
    for seed in seeds.iter().copied() {
        read.seed(seed)?;
        enqueue(seed, 0, &mut visited, &mut frontier, read)?;
    }
    while let Some((distance, entity)) = frontier.pop_front() {
        read.expand(entity, distance, |read, neighbor| {
            enqueue(neighbor, distance + 1, &mut visited, &mut frontier, read)
        })?;
    }
    Ok(visited)
}

fn enqueue<Read: BreadthFirstRead>(
    entity: EntityId,
    distance: usize,
    visited: &mut BTreeSet<EntityId>,
    frontier: &mut VecDeque<(usize, EntityId)>,
    read: &mut Read,
) -> Result<(), Read::Error> {
    if !visited.contains(&entity) {
        read.retain(entity, distance)?;
        visited.insert(entity);
        frontier.push_back((distance, entity));
    }
    Ok(())
}
