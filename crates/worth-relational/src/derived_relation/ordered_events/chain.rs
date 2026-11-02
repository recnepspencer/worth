//! Actual chain proof and complete unpruned successor candidates.
use super::*;
use crate::structural_traversal::{walk, BreadthFirstRead};

pub(super) struct ValidatedEventCandidates {
    seeds: Vec<EntityId>,
}
impl ValidatedEventCandidates {
    pub(super) fn seeds(&self) -> &[EntityId] {
        &self.seeds
    }
}

pub(super) fn validate_candidates<Port: SourceRead>(
    source: &mut EventSource<'_, Port>,
    seeds: Vec<EntityId>,
) -> Result<ValidatedEventCandidates, Denial> {
    for seed in &seeds {
        prefix(source, *seed)?;
    }
    // Validation precedes every retirement-pruned membership traversal.
    successors(source, &seeds, None, true)?;
    Ok(ValidatedEventCandidates { seeds })
}

pub(super) fn prefix<Port: SourceRead>(
    source: &mut EventSource<'_, Port>,
    seed: EntityId,
) -> Result<Vec<EntityId>, Denial> {
    let mut policy = Prefix {
        source,
        ordered: Vec::new(),
        terminated: false,
    };
    let visited = walk(&[seed], &mut policy)?;
    if !policy.terminated {
        return Err(Definition::InvalidEventChain.into());
    }
    for member in visited {
        policy.source.remember_prefix(member)?;
    }
    Ok(policy.ordered)
}
struct Prefix<'a, 'b, Port> {
    source: &'a mut EventSource<'b, Port>,
    ordered: Vec<EntityId>,
    terminated: bool,
}
impl<Port: SourceRead> BreadthFirstRead for Prefix<'_, '_, Port> {
    type Error = Denial;
    fn seed(&mut self, _: EntityId) -> Result<(), Denial> {
        self.source.read.checkpoint(1)
    }
    fn retain(&mut self, member: EntityId, _: usize) -> Result<(), Denial> {
        self.source
            .read
            .retain(3 * size_of::<EntityId>() + 5 * size_of::<usize>())?;
        super::super::source_buffer::reserve(self.source.read, &mut self.ordered, 1)?;
        self.ordered.push(member);
        Ok(())
    }
    fn membership_lookup(&mut self, _: &EntityId, count: usize) -> Result<(), Denial> {
        self.source
            .read
            .checkpoint((usize::BITS - count.max(1).leading_zeros()) as usize)
    }
    fn expand(
        &mut self,
        event: EntityId,
        _: usize,
        mut enqueue: impl FnMut(&mut Self, EntityId) -> Result<(), Denial>,
    ) -> Result<(), Denial> {
        if self.source.known_prefix(event)? {
            self.terminated = true;
        } else if let Some((previous, _)) = self.source.next(event)? {
            enqueue(self, previous)?;
        } else {
            self.terminated = true;
        }
        Ok(())
    }
}

pub(super) fn successors<Port: SourceRead>(
    source: &mut EventSource<'_, Port>,
    seeds: &[EntityId],
    row: Option<EntityId>,
    validate: bool,
) -> Result<BTreeSet<EntityId>, Denial> {
    let mut policy = Successors {
        source,
        row,
        validate,
    };
    walk(seeds, &mut policy)
}
struct Successors<'a, 'b, Port> {
    source: &'a mut EventSource<'b, Port>,
    row: Option<EntityId>,
    validate: bool,
}
impl<Port: SourceRead> BreadthFirstRead for Successors<'_, '_, Port> {
    type Error = Denial;
    fn seed(&mut self, _: EntityId) -> Result<(), Denial> {
        self.source.read.checkpoint(1)
    }
    fn retain(&mut self, _: EntityId, _: usize) -> Result<(), Denial> {
        self.source
            .read
            .retain(3 * size_of::<EntityId>() + 5 * size_of::<usize>())
    }
    fn membership_lookup(&mut self, _: &EntityId, count: usize) -> Result<(), Denial> {
        self.source
            .read
            .checkpoint((usize::BITS - count.max(1).leading_zeros()) as usize)
    }
    fn expand(
        &mut self,
        event: EntityId,
        _: usize,
        mut enqueue: impl FnMut(&mut Self, EntityId) -> Result<(), Denial>,
    ) -> Result<(), Denial> {
        let index = self
            .source
            .leaf(self.source.recipe.predecessor, event, true)?;
        // Keep the genuine raw record roster borrowed only across checkpoints;
        // records are Copy and the support vector may grow on the next read.
        let count = self.source.support[index].physical_relations().len();
        for i in 0..count {
            self.source.read.checkpoint(1)?;
            let record = self.source.support[index].physical_relations()[i];
            let successor = record.source;
            if self.validate {
                if self.source.next(successor)? != Some((event, record.relation_id)) {
                    return Err(Definition::InvalidEventChain.into());
                }
                self.source.remember_prefix(successor)?;
            }
            if let Some(row) = self.row
                && self.source.retired_at(successor, row)?
            {
                continue;
            }
            enqueue(self, successor)?;
        }
        Ok(())
    }
}
