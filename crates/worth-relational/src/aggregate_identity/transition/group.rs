//! Allocation-independent group naming over complete before/after partitions.
//!
//! A group with no live prior source is named
//! `mint_group_name(scheme, birth anchor, continuity rule)`, so a retired group
//! re-formed around its anchor revives its exact name. An unkept split or merge
//! successor is named by `mint_group_transition`, which also hashes the sorted
//! identities of the prior groups it received members from. The birth anchor is
//! the least name bearing the rule's role (or, for
//! [`AggregateContinuityRule::Anchor`] without a role member, the least member
//! name). Keepers follow the declared continuity rule. Native carriers, commit
//! identities, and input order never decide anything.
use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::AggregateIdentity;

use super::mint::{mint_group_identity, mint_group_name};
use crate::aggregate_identity::group_lineage::AggregateGroupLineageEvent as Event;
use crate::lineage::data::PersistentNameIdentity as Name;
mod denial;
mod keeper;
mod model;
mod outcome;
#[cfg(test)]
mod tests;

pub use denial::AggregateGroupNamingDenial;
pub use model::{
    AggregateContinuityRule, AggregateGroupAssignment, AggregateGroupClaimant,
    AggregateGroupMember, AggregateGroupMembers, AggregateGroupOrigin, AggregateGroupScheme,
    AggregateGroupSchemeId, AggregateMemberRole, AggregatePriorGroup,
};
use AggregateGroupNamingDenial as Denial;
use AggregateMemberRole as Role;

/// The v2 group-name doors onto [`AggregateIdentity`].
pub trait AggregateGroupNaming {
    /// Mints the allocation-independent name of a group anchored at `anchor`.
    fn from_group_name(
        scheme: &AggregateGroupSchemeId,
        anchor: Name,
        rule: AggregateContinuityRule,
    ) -> AggregateIdentity;

    /// Mints the name of an unkept split/merge successor of `sources`, which
    /// are canonicalized (sorted, deduplicated) before hashing.
    fn from_group_transition(
        scheme: &AggregateGroupSchemeId,
        anchor: Name,
        rule: AggregateContinuityRule,
        sources: &[AggregateIdentity],
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

    fn from_group_transition(
        scheme: &AggregateGroupSchemeId,
        anchor: Name,
        rule: AggregateContinuityRule,
        sources: &[AggregateIdentity],
    ) -> AggregateIdentity {
        let sources: Vec<_> = sources
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        super::mint::mint_group_transition(scheme, anchor, rule, &sources)
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

/// The name one next-state group will carry.
#[derive(Clone, Debug)]
struct GroupName {
    identity: AggregateIdentity,
    anchor: Name,
    sources: Vec<AggregateIdentity>,
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
        let rule = scheme.rule();
        let old_owner = index_prior(scheme, prior, retired_history)?;
        proposed.sort_by_key(|members| members.least_name());
        let new_owner = index_proposed(&proposed)?;
        let anchors = proposed
            .iter()
            .map(|members| keeper::birth_anchor(rule, members))
            .collect::<Result<Vec<_>, _>>()?;
        let destination: Vec<_> = prior
            .iter()
            .map(|old| keeper::destination(rule, old, &new_owner))
            .collect();
        let kept_by = keeper::choose_keepers(rule, prior, &destination, proposed.len());
        let names: Vec<GroupName> = proposed
            .iter()
            .enumerate()
            .map(|(group, members)| match kept_by[group] {
                Some(old) => GroupName {
                    identity: prior[old].identity,
                    anchor: prior[old].anchor,
                    sources: prior[old].sources.clone(),
                },
                None => {
                    let sources: BTreeSet<_> = members
                        .iter()
                        .filter_map(|(name, _)| old_owner.get(name))
                        .map(|&old| prior[old].identity)
                        .collect();
                    let sources: Vec<_> = sources.into_iter().collect();
                    let anchor = anchors[group];
                    GroupName {
                        identity: mint_group_identity(scheme.id(), anchor, rule, &sources),
                        anchor,
                        sources,
                    }
                }
            })
            .collect();
        let plan = outcome::Plan {
            prior,
            proposed,
            names,
            kept_by,
            destination,
            old_owner,
        };
        plan.guard_live_collisions()?;
        Ok(plan.assemble(retired_history))
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
        let canonical = group.sources.windows(2).all(|pair| pair[0] < pair[1]);
        // Non-canonical sources refuse before any hashing sees them.
        if !canonical
            || identity
                != mint_group_identity(scheme.id(), group.anchor, scheme.rule(), &group.sources)
        {
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

/// Indexes the canonically sorted proposal: member name to its group position
/// and its role there.
fn index_proposed(
    proposed: &[AggregateGroupMembers],
) -> Result<BTreeMap<Name, (usize, Role)>, Denial> {
    let mut names = BTreeMap::new();
    let mut carriers = BTreeSet::new();
    for (group, members) in proposed.iter().enumerate() {
        if members.is_empty() {
            return Err(Denial::EmptyGroup);
        }
        for (name, member) in members.iter() {
            if names.insert(*name, (group, member.role)).is_some() {
                return Err(Denial::OverlappingMember { name: *name });
            }
            if !carriers.insert(&member.carrier) {
                return Err(Denial::DuplicateCarrier { name: *name });
            }
        }
    }
    Ok(names)
}
