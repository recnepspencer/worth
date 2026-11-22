//! Allocation-independent group naming over complete before/after partitions.
//!
//! A group's name is `mint_group_name(scheme, birth anchor, continuity rule)`.
//! The birth anchor is the least name bearing the rule's role (or, for
//! [`AggregateContinuityRule::Anchor`] without a role member, the least member
//! name). Split and merge keepers follow the declared continuity rule. Native
//! carriers, commit identities, and input order never decide anything.
use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::AggregateIdentity;

use super::mint::mint_group_name;
use crate::aggregate_identity::group_lineage::AggregateGroupLineageEvent as Event;
use crate::lineage::data::PersistentNameIdentity as Name;
mod denial;
mod model;
#[cfg(test)]
mod tests;

pub use denial::AggregateGroupNamingDenial;
pub use model::{
    AggregateContinuityRule, AggregateGroupAssignment, AggregateGroupClaimant,
    AggregateGroupMember, AggregateGroupMembers, AggregateGroupOrigin, AggregateGroupScheme,
    AggregateGroupSchemeId, AggregateMemberRole, AggregatePriorGroup,
};
use AggregateContinuityRule as Rule;
use AggregateGroupNamingDenial as Denial;
use AggregateMemberRole as Role;

/// The v2 group-name door onto [`AggregateIdentity`].
pub trait AggregateGroupNaming {
    /// Mints the allocation-independent name of a group anchored at `anchor`.
    fn from_group_name(
        scheme: &AggregateGroupSchemeId,
        anchor: Name,
        rule: AggregateContinuityRule,
    ) -> AggregateIdentity;
}

impl AggregateGroupNaming for AggregateIdentity {
    fn from_group_name(
        scheme: &AggregateGroupSchemeId,
        anchor: Name,
        rule: AggregateContinuityRule,
    ) -> AggregateIdentity {
        mint_group_name(scheme, anchor, rule)
    }
}

/// The complete naming outcome of one group transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregateGroupTransition {
    /// Next-state groups in identity order.
    pub assignments: Vec<AggregateGroupAssignment>,
    pub retired: BTreeSet<AggregateIdentity>,
    pub lineage: BTreeSet<Event>,
}

/// Relational authority for allocation-independent group names.
pub struct AggregateGroupNamingAuthority;

impl AggregateGroupNamingAuthority {
    /// Names the `proposed` partition against the `prior` live groups.
    ///
    /// Complexity: O(M log M) in total member count. Fail-closed: any refusal
    /// returns before a single assignment, retirement, or event exists.
    pub fn transition(
        scheme: &AggregateGroupScheme,
        prior: &[AggregatePriorGroup],
        retired_history: &BTreeSet<AggregateIdentity>,
        mut proposed: Vec<AggregateGroupMembers>,
    ) -> Result<AggregateGroupTransition, Denial> {
        let old_owner = index_prior(scheme, prior, retired_history)?;
        proposed.sort_by_key(|members| members.least_name());
        let new_owner = index_proposed(&proposed)?;
        let destination: Vec<Option<(usize, usize)>> = prior
            .iter()
            .map(|old| destination(scheme.rule(), old, &new_owner))
            .collect();
        let mut kept_by = choose_keepers(scheme.rule(), prior, &destination, proposed.len());
        let mut names = Vec::with_capacity(proposed.len());
        for (group, members) in proposed.iter().enumerate() {
            names.push(match kept_by[group] {
                Some(old) => (prior[old].identity, prior[old].anchor),
                None => {
                    let anchor = birth_anchor(scheme.rule(), members)?;
                    (mint_group_name(scheme.id(), anchor, scheme.rule()), anchor)
                }
            });
        }
        let by_identity: BTreeMap<AggregateIdentity, usize> = prior
            .iter()
            .enumerate()
            .map(|(old, group)| (group.identity, old))
            .collect();
        let mut keeper_of = vec![None; prior.len()];
        for (group, old) in kept_by.iter().enumerate() {
            if let Some(old) = old {
                keeper_of[*old] = Some(group);
            }
        }
        // A born name equal to an unkept prior name re-forms that same group.
        for (group, (identity, _)) in names.iter().enumerate() {
            if let Some(&old) = by_identity.get(identity)
                && kept_by[group].is_none()
                && keeper_of[old].is_none()
            {
                kept_by[group] = Some(old);
                keeper_of[old] = Some(group);
            }
        }
        guard_live_collisions(&proposed, &names, &kept_by, prior)?;
        let mut result = AggregateGroupTransition {
            assignments: Vec::with_capacity(proposed.len()),
            retired: BTreeSet::new(),
            lineage: BTreeSet::new(),
        };
        for (group, members) in proposed.into_iter().enumerate() {
            let (identity, anchor) = names[group];
            let origin = match kept_by[group] {
                Some(_) => AggregateGroupOrigin::Kept,
                None if retired_history.contains(&identity) => AggregateGroupOrigin::Revived,
                None => AggregateGroupOrigin::Born,
            };
            match origin {
                AggregateGroupOrigin::Born => {
                    result.lineage.insert(Event::Born { group: identity })
                }
                AggregateGroupOrigin::Revived => {
                    result.lineage.insert(Event::Revived { group: identity })
                }
                AggregateGroupOrigin::Kept => false,
            };
            for (name, _) in members.iter() {
                let Some(&old) = old_owner.get(name) else {
                    continue;
                };
                let home = keeper_of[old].or(destination[old].map(|(group, _)| group));
                if home != Some(group) {
                    result.lineage.insert(Event::SplitFrom {
                        fragment: identity,
                        source: prior[old].identity,
                    });
                }
            }
            result.assignments.push(AggregateGroupAssignment {
                identity,
                anchor,
                members,
                origin,
            });
        }
        for (old, group) in prior.iter().enumerate() {
            if keeper_of[old].is_some() {
                continue;
            }
            result.retired.insert(group.identity);
            result.lineage.insert(Event::Retired {
                group: group.identity,
            });
            if let Some((into, _)) = destination[old] {
                result.lineage.insert(Event::MergedInto {
                    absorbed: group.identity,
                    keeper: names[into].0,
                });
            }
        }
        result
            .assignments
            .sort_by_key(|assignment| assignment.identity);
        Ok(result)
    }
}

fn index_prior(
    scheme: &AggregateGroupScheme,
    prior: &[AggregatePriorGroup],
    retired_history: &BTreeSet<AggregateIdentity>,
) -> Result<BTreeMap<Name, usize>, Denial> {
    let mut identities = BTreeSet::new();
    let mut owner = BTreeMap::new();
    for (old, group) in prior.iter().enumerate() {
        let identity = group.identity;
        if !identities.insert(identity) {
            return Err(Denial::DuplicatePriorIdentity { identity });
        }
        if identity != mint_group_name(scheme.id(), group.anchor, scheme.rule()) {
            return Err(Denial::PriorIdentityMismatch { identity });
        }
        if retired_history.contains(&identity) {
            return Err(Denial::RetiredIdentityLive { identity });
        }
        if group.members.is_empty() {
            return Err(Denial::EmptyGroup);
        }
        for (name, _) in group.members.iter() {
            if owner.insert(*name, old).is_some() {
                return Err(Denial::OverlappingMember { name: *name });
            }
        }
    }
    Ok(owner)
}

/// Indexes the canonically sorted proposal: member name to group position.
fn index_proposed(proposed: &[AggregateGroupMembers]) -> Result<BTreeMap<Name, usize>, Denial> {
    let mut names = BTreeMap::new();
    let mut carriers = BTreeSet::new();
    for (group, members) in proposed.iter().enumerate() {
        if members.is_empty() {
            return Err(Denial::EmptyGroup);
        }
        for (name, member) in members.iter() {
            if names.insert(*name, group).is_some() {
                return Err(Denial::OverlappingMember { name: *name });
            }
            if !carriers.insert(&member.carrier) {
                return Err(Denial::DuplicateCarrier { name: *name });
            }
        }
    }
    Ok(names)
}

/// The next-state group a prior group's continuity rule follows, with the
/// rule's strength score. `None` means the rule keeps nothing.
fn destination(
    rule: Rule,
    old: &AggregatePriorGroup,
    new_owner: &BTreeMap<Name, usize>,
) -> Option<(usize, usize)> {
    let anchored = || new_owner.get(&old.anchor).map(|&group| (group, 0));
    match rule {
        Rule::Anchor => anchored(),
        Rule::ProducerRole => anchored().filter(|_| {
            old.members.get(&old.anchor).map(|member| member.role) == Some(Role::Anchor)
        }),
        Rule::Boundary => {
            let mut counts = BTreeMap::<usize, usize>::new();
            for (name, member) in old.members.iter() {
                if member.role == Role::Boundary
                    && let Some(&group) = new_owner.get(name)
                {
                    *counts.entry(group).or_default() += 1;
                }
            }
            unique_max(counts.into_iter())
        }
    }
}

/// For each next-state group, the prior group it keeps, if any.
fn choose_keepers(
    rule: Rule,
    prior: &[AggregatePriorGroup],
    destination: &[Option<(usize, usize)>],
    groups: usize,
) -> Vec<Option<usize>> {
    let mut claims = vec![Vec::new(); groups];
    for (old, target) in destination.iter().enumerate() {
        if let Some((group, score)) = target {
            claims[*group].push((old, *score));
        }
    }
    claims
        .into_iter()
        .map(|claim| match rule {
            Rule::Anchor | Rule::ProducerRole => claim
                .into_iter()
                .min_by_key(|(old, _)| prior[*old].anchor)
                .map(|(old, _)| old),
            Rule::Boundary => unique_max(claim.into_iter()).map(|(old, _)| old),
        })
        .collect()
}

/// The sole entry with the strictly greatest nonzero score; ties keep nothing.
fn unique_max(entries: impl Iterator<Item = (usize, usize)>) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize)> = None;
    let mut tied = false;
    for (key, score) in entries.filter(|(_, score)| *score > 0) {
        match best {
            Some((_, top)) if score == top => tied = true,
            Some((_, top)) if score < top => {}
            _ => {
                best = Some((key, score));
                tied = false;
            }
        }
    }
    best.filter(|_| !tied)
}

fn birth_anchor(rule: Rule, members: &AggregateGroupMembers) -> Result<Name, Denial> {
    let least_member = members.least_name().ok_or(Denial::EmptyGroup)?;
    let anchor = match rule {
        Rule::Anchor => members.least_with(Role::Anchor).or(Some(least_member)),
        Rule::ProducerRole => members.least_with(Role::Anchor),
        Rule::Boundary => members.least_with(Role::Boundary),
    };
    anchor.ok_or(Denial::MissingAnchorMember { rule, least_member })
}

/// Two simultaneous live groups may never hold one name.
fn guard_live_collisions(
    proposed: &[AggregateGroupMembers],
    names: &[(AggregateIdentity, Name)],
    kept_by: &[Option<usize>],
    prior: &[AggregatePriorGroup],
) -> Result<(), Denial> {
    let claimant = |group: usize| AggregateGroupClaimant {
        anchor: names[group].1,
        least_member: proposed[group].least_name().expect("nonempty group"),
        kept_from: kept_by[group].map(|old| prior[old].identity),
    };
    let mut holders = BTreeMap::new();
    for (group, (identity, _)) in names.iter().enumerate() {
        if let Some(first) = holders.insert(*identity, group) {
            let (first, second) = (claimant(first), claimant(group));
            return Err(Denial::LiveNameCollision {
                identity: *identity,
                first: first.min(second),
                second: first.max(second),
            });
        }
    }
    Ok(())
}
