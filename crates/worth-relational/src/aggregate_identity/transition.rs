//! Naming policy over complete partitions, independent of publication timing.
use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};
pub(crate) use worth_foundational::facade::AggregateLineageKind;
use worth_foundational::facade::{AggregateDefinitionIdentity, AggregateIdentity};

use crate::history::data::CommitId;
use crate::identity::data::EntityId;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PriorAggregate {
    pub(crate) identity: AggregateIdentity,
    pub(crate) members: BTreeSet<EntityId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AggregateAssignment {
    pub(crate) identity: AggregateIdentity,
    pub(crate) members: BTreeSet<EntityId>,
    pub(crate) born: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AggregateLineageEvent {
    pub(crate) occurrence: CommitId,
    pub(crate) kind: AggregateLineageKind,
    pub(crate) from: AggregateIdentity,
    pub(crate) to: AggregateIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AggregateTransition {
    pub(crate) assignments: Vec<AggregateAssignment>,
    pub(crate) retired: BTreeSet<AggregateIdentity>,
    pub(crate) lineage: BTreeSet<AggregateLineageEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AggregateTransitionDenial {
    EmptyAggregate,
    OverlappingMembership,
    DuplicateIdentity,
    MintCollision,
}

pub(crate) fn transition<E: From<AggregateTransitionDenial>>(
    definition: AggregateDefinitionIdentity,
    occurrence: CommitId,
    prior: &[PriorAggregate],
    mut proposed: Vec<BTreeSet<EntityId>>,
    checkpoint: &mut impl FnMut(u64) -> Result<(), E>,
) -> Result<AggregateTransition, E> {
    let (old_members, old_identities) = index_prior(prior, checkpoint)?;
    if proposed.iter().any(BTreeSet::is_empty) {
        return Err(AggregateTransitionDenial::EmptyAggregate.into());
    }
    proposed.sort_by_key(|members| *members.first().expect("nonempty checked"));
    let new_members = index_groups(&proposed, checkpoint)?;
    let mut keepers = Vec::with_capacity(prior.len());
    for old in prior {
        let mut keeper = None;
        for member in &old.members {
            checkpoint(1)?;
            if let Some(&group) = new_members.get(member) {
                keeper = Some(group);
                break;
            }
        }
        keepers.push(keeper);
    }
    let mut result = AggregateTransition {
        assignments: Vec::with_capacity(proposed.len()),
        retired: BTreeSet::new(),
        lineage: BTreeSet::new(),
    };
    let mut assigned = BTreeSet::new();
    for (group, members) in proposed.into_iter().enumerate() {
        checkpoint(1)?;
        let fragments = incoming_fragments(&members, &old_members, checkpoint)?;
        let winner = fragments.iter().min_by_key(|(_, anchor)| *anchor);
        let retained = winner
            .filter(|(old, _)| keepers[**old] == Some(group))
            .map(|(old, _)| prior[*old].identity);
        let identity = retained.unwrap_or_else(|| {
            mint(
                definition,
                occurrence,
                *members.first().expect("nonempty checked"),
            )
        });
        if !assigned.insert(identity) || (retained.is_none() && old_identities.contains(&identity))
        {
            return Err(AggregateTransitionDenial::MintCollision.into());
        }
        for &old in fragments.keys() {
            checkpoint(1)?;
            if keepers[old] != Some(group) {
                result.lineage.insert(AggregateLineageEvent {
                    occurrence,
                    kind: AggregateLineageKind::SplitFrom,
                    from: identity,
                    to: prior[old].identity,
                });
            }
        }
        result.assignments.push(AggregateAssignment {
            identity,
            members,
            born: retained.is_none(),
        });
    }
    for (old, keeper) in prior.iter().zip(keepers) {
        checkpoint(1)?;
        let survivor = keeper.map(|group| result.assignments[group].identity);
        if survivor != Some(old.identity) {
            result.retired.insert(old.identity);
            if let Some(to) = survivor {
                result.lineage.insert(AggregateLineageEvent {
                    occurrence,
                    kind: AggregateLineageKind::MergedInto,
                    from: old.identity,
                    to,
                });
            }
        }
    }
    Ok(result)
}

fn index_prior<E: From<AggregateTransitionDenial>>(
    prior: &[PriorAggregate],
    checkpoint: &mut impl FnMut(u64) -> Result<(), E>,
) -> Result<(BTreeMap<EntityId, usize>, BTreeSet<AggregateIdentity>), E> {
    let mut identities = BTreeSet::new();
    let mut members = BTreeMap::new();
    for (group, old) in prior.iter().enumerate() {
        checkpoint(1)?;
        if !identities.insert(old.identity) {
            return Err(AggregateTransitionDenial::DuplicateIdentity.into());
        }
        if old.members.is_empty() {
            return Err(AggregateTransitionDenial::EmptyAggregate.into());
        }
        for &member in &old.members {
            checkpoint(1)?;
            if members.insert(member, group).is_some() {
                return Err(AggregateTransitionDenial::OverlappingMembership.into());
            }
        }
    }
    Ok((members, identities))
}

fn index_groups<E: From<AggregateTransitionDenial>>(
    groups: &[BTreeSet<EntityId>],
    checkpoint: &mut impl FnMut(u64) -> Result<(), E>,
) -> Result<BTreeMap<EntityId, usize>, E> {
    let mut members = BTreeMap::new();
    for (group, entries) in groups.iter().enumerate() {
        if entries.is_empty() {
            return Err(AggregateTransitionDenial::EmptyAggregate.into());
        }
        for &member in entries {
            checkpoint(1)?;
            if members.insert(member, group).is_some() {
                return Err(AggregateTransitionDenial::OverlappingMembership.into());
            }
        }
    }
    Ok(members)
}

fn incoming_fragments<E>(
    members: &BTreeSet<EntityId>,
    old_members: &BTreeMap<EntityId, usize>,
    checkpoint: &mut impl FnMut(u64) -> Result<(), E>,
) -> Result<BTreeMap<usize, EntityId>, E> {
    let mut fragments = BTreeMap::new();
    for member in members {
        checkpoint(1)?;
        if let Some(&old) = old_members.get(member) {
            fragments.entry(old).or_insert(*member);
        }
    }
    Ok(fragments)
}

pub(super) fn mint(
    definition: AggregateDefinitionIdentity,
    occurrence: CommitId,
    anchor: EntityId,
) -> AggregateIdentity {
    let mut hash = Sha256::new();
    hash.update(b"WORTH_GRAPH_AGGREGATE_ID_V1\0");
    hash.update(definition.bytes());
    hash.update(occurrence.0.to_be_bytes());
    hash.update(anchor.partition_value_u64().to_be_bytes());
    hash.update(anchor.local_slot_value().to_be_bytes());
    hash.update(anchor.generation_value().to_be_bytes());
    AggregateIdentity::from_bytes(hash.finalize().into())
}
