//! Real endpoint deltas drive the same SDK component partitioner as physical sources.
use super::*;
use crate::aggregate_identity::cohorts::checked_components::CheckedComponents;
use crate::aggregate_identity::membership::key_lookup;
use worth_execution::{ComponentEdit, ExecutionRequest, MapKernelFailure, PartitionItemId};
use worth_foundational::PartitionIdentity;

pub(super) fn maintain(
    execution: &ExecutionRequest<'_, '_>,
    before: &Discovery,
    after: &Discovery,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<Vec<BTreeSet<Member>>> {
    let old = universe(before, work)?;
    let new = universe(after, work)?;
    let mut items = BTreeMap::new();
    for member in old.iter().chain(&new) {
        key_lookup(member, items.len(), work)?;
        if items.contains_key(member) {
            continue;
        }
        work.retain(extent::tree_entry::<Member, PartitionItemId>())?;
        key_lookup(member, items.len(), work)?;
        let id =
            u64::try_from(items.len()).map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
        items.insert(member.clone(), PartitionItemId(id));
    }
    let mut graph = CheckedComponents::new();
    for member in &old {
        key_lookup(member, items.len(), work)?;
        graph.edit(execution, ComponentEdit::UpsertItem(items[member]), work)?;
    }
    for pair in &before.pairs {
        edit_pair(execution, &mut graph, &items, pair, true, work)?;
    }
    for pair in &before.pairs {
        pair_lookup(pair, after.pairs.len(), work)?;
        if !after.pairs.contains(pair) {
            edit_pair(execution, &mut graph, &items, pair, false, work)?;
        }
    }
    for member in &old {
        key_lookup(member, new.len(), work)?;
        if !new.contains(member) {
            key_lookup(member, items.len(), work)?;
            graph.edit(execution, ComponentEdit::RemoveItem(items[member]), work)?;
        }
    }
    for member in &new {
        key_lookup(member, items.len(), work)?;
        graph.edit(execution, ComponentEdit::UpsertItem(items[member]), work)?;
    }
    for pair in &after.pairs {
        pair_lookup(pair, before.pairs.len(), work)?;
        if !before.pairs.contains(pair) {
            edit_pair(execution, &mut graph, &items, pair, true, work)?;
        }
    }
    let mut groups: BTreeMap<PartitionIdentity, BTreeSet<Member>> = BTreeMap::new();
    for member in new {
        key_lookup(&member, items.len(), work)?;
        let Some(route) = graph.route(items[&member]) else {
            return work.invalid(PlanDenial::InvalidComponentGraph);
        };
        work.retain(
            extent::tree_entry::<PartitionIdentity, BTreeSet<Member>>()
                + extent::tree_entry::<Member, ()>(),
        )?;
        key_lookup(&member, groups.len(), work)?;
        groups.entry(route).or_default().insert(member);
    }
    work.retain(
        (groups.len() as u64)
            .checked_mul(extent::vector_entry::<BTreeSet<Member>>())
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?,
    )?;
    Ok(groups.into_values().collect())
}

fn universe(
    discovery: &Discovery,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<BTreeSet<Member>> {
    let mut result = BTreeSet::new();
    for group in &discovery.groups {
        for member in group {
            key_lookup(member, result.len(), work)?;
            work.retain(extent::tree_entry::<Member, ()>())?;
            if !result.insert(member.clone()) {
                return work.invalid(PlanDenial::InvalidComponentGraph);
            }
        }
    }
    Ok(result)
}
fn pair_lookup(
    pair: &(Member, Member),
    count: usize,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<()> {
    key_lookup(&pair.0, count, work)?;
    key_lookup(&pair.1, count, work)
}
fn edit_pair(
    execution: &ExecutionRequest<'_, '_>,
    graph: &mut CheckedComponents,
    items: &BTreeMap<Member, PartitionItemId>,
    pair: &(Member, Member),
    add: bool,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<()> {
    pair_lookup(pair, items.len(), work)?;
    let (Some(&a), Some(&b)) = (items.get(&pair.0), items.get(&pair.1)) else {
        return work.invalid(PlanDenial::InvalidComponentGraph);
    };
    graph.edit(
        execution,
        if add {
            ComponentEdit::AddEdge(a, b)
        } else {
            ComponentEdit::RemoveEdge(a, b)
        },
        work,
    )
}
