//! Ordered physical-event membership, retaining one original-side acquisition.
use super::evaluation::{evaluate_position, insert_member, SourceRead};
use super::{DerivedRelationData as Data, DerivedRelationDenial as Definition};
use super::{DerivedRelationHandle as Handle, DerivedRelationReadDenial as Denial};
use crate::graph_member::NativeMemberKey as Member;
use crate::identity::data::{EntityId, RelationId};
use crate::schema::data::RelationalSchemaRegistry;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) mod affected;
mod chain;
mod membership;

#[derive(Clone, Copy)]
pub(in crate::derived_relation) struct Recipe {
    pub predecessor: usize,
    pub introduced: usize,
    pub retired: usize,
}

pub(in crate::derived_relation) struct Acquisition {
    pub members: BTreeSet<Member>,
    pub support: Vec<Data>,
}

pub(in crate::derived_relation) fn acquire(
    registry: &RelationalSchemaRegistry,
    definition: &Handle,
    recipe: Recipe,
    anchor: EntityId,
    reverse: bool,
    read: &mut impl SourceRead,
) -> Result<Acquisition, Denial> {
    let mut source = EventSource::new(registry, definition, recipe, read);
    let members = if reverse {
        membership::inverse(&mut source, anchor)?
    } else {
        membership::forward(&mut source, anchor)?
    };
    Ok(Acquisition {
        members,
        support: source.support,
    })
}

/// Only one exact immutable-side read owns this memo and its original funding.
struct EventSource<'a, Port> {
    registry: &'a RelationalSchemaRegistry,
    definition: &'a Handle,
    recipe: Recipe,
    read: &'a mut Port,
    support: Vec<Data>,
    leaves: BTreeMap<(usize, bool, EntityId), usize>,
    valid_prefix: BTreeSet<EntityId>,
}
impl<'a, Port: SourceRead> EventSource<'a, Port> {
    fn new(
        registry: &'a RelationalSchemaRegistry,
        definition: &'a Handle,
        recipe: Recipe,
        read: &'a mut Port,
    ) -> Self {
        Self {
            registry,
            definition,
            recipe,
            read,
            support: Vec::new(),
            leaves: BTreeMap::new(),
            valid_prefix: BTreeSet::new(),
        }
    }
    fn leaf(&mut self, node: usize, anchor: EntityId, reverse: bool) -> Result<usize, Denial> {
        let key = (node, reverse, anchor);
        let levels = (usize::BITS - self.leaves.len().max(1).leading_zeros()) as usize;
        self.read.checkpoint(
            size_of_val(&key)
                .checked_mul(11)
                .and_then(|n| n.checked_mul(levels))
                .ok_or(Definition::WorkBudgetExceeded)?,
        )?;
        if let Some(index) = self.leaves.get(&key) {
            return Ok(*index);
        }
        // Registration permits only literal Stored nodes here, so this cannot
        // recurse into another event fold or create a temporary recipe issuer.
        let data = evaluate_position(
            self.registry,
            self.definition,
            node,
            reverse,
            anchor.into(),
            self.read,
        )?;
        let index = self.support.len();
        super::support::push(self.read, &mut self.support, data)?;
        self.read
            .retain(size_of_val(&key) + size_of::<usize>() + 4 * size_of::<usize>())?;
        self.leaves.insert(key, index);
        Ok(index)
    }
    fn next(&mut self, event: EntityId) -> Result<Option<(EntityId, RelationId)>, Denial> {
        let index = self.leaf(self.recipe.predecessor, event, false)?;
        // Raw records, not endpoint deduplication, prove the chain premise.
        match self.support[index].physical_relations() {
            [] => Ok(None),
            [record] => Ok(Some((record.target, record.relation_id))),
            _ => Err(Definition::InvalidEventChain.into()),
        }
    }
    fn targets(
        &mut self,
        node: usize,
        event: EntityId,
        reverse: bool,
    ) -> Result<Vec<EntityId>, Denial> {
        let index = self.leaf(node, event, reverse)?;
        let count = self.support[index].endpoints().len();
        self.read.retain(
            count
                .checked_mul(size_of::<EntityId>())
                .ok_or(Definition::WorkBudgetExceeded)?,
        )?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(count)
            .map_err(|_| allocation_failed())?;
        for member in self.support[index].endpoints() {
            self.read.checkpoint(1)?;
            output.push(member.entity().ok_or(Definition::InvalidMemberDomain)?);
        }
        Ok(output)
    }
    fn retired_at(&mut self, event: EntityId, row: EntityId) -> Result<bool, Denial> {
        let index = self.leaf(self.recipe.retired, event, false)?;
        let endpoints = self.support[index].endpoints();
        let levels = (usize::BITS - endpoints.len().max(1).leading_zeros()) as usize;
        self.read.checkpoint(levels)?;
        Ok(endpoints.binary_search(&Member::Entity(row)).is_ok())
    }
    fn known_prefix(&mut self, event: EntityId) -> Result<bool, Denial> {
        self.prefix_work()?;
        Ok(self.valid_prefix.contains(&event))
    }
    fn remember_prefix(&mut self, event: EntityId) -> Result<(), Denial> {
        if !self.known_prefix(event)? {
            self.read
                .retain(size_of::<EntityId>() + 4 * size_of::<usize>())?;
            self.valid_prefix.insert(event);
        }
        Ok(())
    }
    fn prefix_work(&mut self) -> Result<(), Denial> {
        let levels = (usize::BITS - self.valid_prefix.len().max(1).leading_zeros()) as usize;
        self.read.checkpoint(
            levels
                .checked_mul(11)
                .ok_or(Definition::WorkBudgetExceeded)?,
        )
    }
}
fn allocation_failed() -> Denial {
    Denial::Request(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
}
