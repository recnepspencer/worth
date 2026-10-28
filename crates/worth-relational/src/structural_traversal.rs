//! Native frontier mechanics shared by bounded scope and complete reads.
use std::collections::{BTreeSet, VecDeque};

use crate::identity::data::EntityId;

pub(crate) trait BreadthFirstRead<Member = EntityId> {
    type Error;

    fn seed(&mut self, entity: Member) -> Result<(), Self::Error>;

    /// Admit retained frontier/visited storage before either collection grows.
    fn retain(&mut self, entity: Member, distance: usize) -> Result<(), Self::Error>;

    /// Fixed-width physical policies need no additional comparison work.
    /// Semantic logical keys debit their actual search bytes here.
    fn membership_lookup(&mut self, _: &Member, _: usize) -> Result<(), Self::Error> {
        Ok(())
    }

    fn expand(
        &mut self,
        entity: Member,
        distance: usize,
        enqueue: impl FnMut(&mut Self, Member) -> Result<(), Self::Error>,
    ) -> Result<(), Self::Error>;
}

/// O((v + e) log v) frontier work, O(v) scratch. Policies own read/retention
/// admission; the driver never constructs another request or allowance.
pub(crate) fn walk<Member: Clone + Ord, Read: BreadthFirstRead<Member>>(
    seeds: &[Member],
    read: &mut Read,
) -> Result<BTreeSet<Member>, Read::Error> {
    let mut visited = BTreeSet::new();
    let mut frontier = VecDeque::new();
    for seed in seeds.iter().cloned() {
        read.seed(seed.clone())?;
        enqueue(seed, 0, &mut visited, &mut frontier, read)?;
    }
    while let Some((distance, entity)) = frontier.pop_front() {
        read.expand(entity, distance, |read, neighbor| {
            enqueue(neighbor, distance + 1, &mut visited, &mut frontier, read)
        })?;
    }
    Ok(visited)
}

fn enqueue<Member: Clone + Ord, Read: BreadthFirstRead<Member>>(
    entity: Member,
    distance: usize,
    visited: &mut BTreeSet<Member>,
    frontier: &mut VecDeque<(usize, Member)>,
    read: &mut Read,
) -> Result<(), Read::Error> {
    read.membership_lookup(&entity, visited.len())?;
    if !visited.contains(&entity) {
        read.retain(entity.clone(), distance)?;
        read.membership_lookup(&entity, visited.len())?;
        visited.insert(entity.clone());
        frontier.push_back((distance, entity));
    }
    Ok(())
}
