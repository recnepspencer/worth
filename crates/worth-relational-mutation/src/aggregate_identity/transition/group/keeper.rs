//! Continuity-rule keeper selection and birth anchors, by name and role only.
use std::collections::BTreeMap;
use worth_relational_contracts::internal_port::aggregate_identity::transition::group::model::AggregateGroupMembersOwnerPort as _;

use super::{AggregateContinuityRule as Rule, AggregateGroupMembers, AggregatePriorGroup};
use super::{AggregateGroupNamingDenial as Denial, Name, Role};

/// The next-state group a prior group's continuity rule follows, with the
/// rule's strength score. `None` means the rule keeps nothing. A continuity
/// role counts only when the member holds it both before and after.
pub(super) fn destination(
    rule: Rule,
    old: &AggregatePriorGroup,
    new_owner: &BTreeMap<Name, (usize, Role)>,
) -> Option<(usize, usize)> {
    let old_role = |name: &Name| old.members.get(name).map(|member| member.role);
    match rule {
        Rule::Anchor => new_owner.get(&old.anchor).map(|&(group, _)| (group, 0)),
        Rule::ProducerRole => new_owner
            .get(&old.anchor)
            .filter(|&&(_, role)| role == Role::Anchor && old_role(&old.anchor) == Some(role))
            .map(|&(group, _)| (group, 0)),
        Rule::Boundary => {
            let mut counts = BTreeMap::<usize, usize>::new();
            for (name, member) in old.members.iter() {
                if member.role == Role::Boundary
                    && let Some(&(group, Role::Boundary)) = new_owner.get(name)
                {
                    *counts.entry(group).or_default() += 1;
                }
            }
            unique_max(counts.into_iter())
        }
    }
}

/// For each next-state group, the prior group it keeps, if any.
/// `anchors[old]` is prior group `old`'s birth anchor.
pub(super) fn choose_keepers(
    rule: Rule,
    anchors: &[Name],
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
                .min_by_key(|(old, _)| &anchors[*old])
                .map(|(old, _)| old),
            Rule::Boundary => unique_max(claim.into_iter()).map(|(old, _)| old),
        })
        .collect()
}

/// The sole entry with the strictly greatest nonzero score; ties keep nothing.
pub(super) fn unique_max(entries: impl Iterator<Item = (usize, usize)>) -> Option<(usize, usize)> {
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

/// The birth anchor every proposed group must be able to name, kept or not:
/// a rule's required role member can never be absent from a live group.
pub(super) fn birth_anchor(rule: Rule, members: &AggregateGroupMembers) -> Result<Name, Denial> {
    let least_member = members.least_name().ok_or(Denial::EmptyGroup)?;
    let anchor = match rule {
        Rule::Anchor => members
            .least_with(Role::Anchor)
            .or(Some(least_member.clone())),
        Rule::ProducerRole => members.least_with(Role::Anchor),
        Rule::Boundary => members.least_with(Role::Boundary),
    };
    anchor.ok_or(Denial::MissingAnchorMember { rule, least_member })
}
