//! Keepers, birth anchors, names and the live-name collision guard of one
//! incremental group transition, decided exactly as the full-partition
//! authority decides them, from touched members and role counts.
use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::AggregateIdentity;

use super::super::super::mint::mint_group_identity;
use super::super::keeper::{choose_keepers, unique_max};
use super::super::{
    AggregateContinuityRule as Rule, AggregateGroupClaimant, AggregateGroupScheme,
    AggregateGroupStateView, Denial, Name, Role,
};
use super::scope::{Proposal, Shape};

/// The name one next-state group will carry.
pub(super) struct GroupName {
    pub(super) identity: AggregateIdentity,
    pub(super) anchor: Name,
    pub(super) sources: Vec<AggregateIdentity>,
}

/// Every naming decision of one incremental transition, before any effect.
pub(super) struct Plan<'v, V> {
    pub(super) proposal: Proposal<'v, V>,
    pub(super) names: Vec<GroupName>,
    pub(super) kept_by: Vec<Option<usize>>,
    pub(super) destination: Vec<Option<(usize, usize)>>,
}

impl<'v, V: AggregateGroupStateView> Plan<'v, V> {
    /// Complexity: O((T + S + P) log M).
    pub(super) fn name(
        scheme: &AggregateGroupScheme,
        proposal: Proposal<'v, V>,
    ) -> Result<Self, Denial> {
        let rule = scheme.rule();
        let anchors = (0..proposal.groups.len())
            .map(|group| birth_anchor(rule, &proposal, group))
            .collect::<Result<Vec<_>, _>>()?;
        let destination: Vec<_> = (0..proposal.priors.len())
            .map(|old| destination(rule, &proposal, old))
            .collect();
        let prior_anchors: Vec<Name> = proposal
            .priors
            .iter()
            .map(|p| p.header.anchor.clone())
            .collect();
        let kept_by = choose_keepers(rule, &prior_anchors, &destination, proposal.groups.len());
        let names = (0..proposal.groups.len())
            .map(|group| match kept_by[group] {
                Some(old) => {
                    let header = &proposal.priors[old].header;
                    GroupName {
                        identity: header.identity,
                        anchor: header.anchor.clone(),
                        sources: header.sources.clone(),
                    }
                }
                None => {
                    let sources: Vec<_> = sources_of(&proposal, group).into_iter().collect();
                    let anchor = anchors[group].clone();
                    GroupName {
                        identity: mint_group_identity(scheme.id(), &anchor, rule, &sources),
                        anchor,
                        sources,
                    }
                }
            })
            .collect();
        Ok(Self {
            proposal,
            names,
            kept_by,
            destination,
        })
    }

    pub(super) fn keeper_of(&self) -> Vec<Option<usize>> {
        let mut keeper_of = vec![None; self.proposal.priors.len()];
        for (group, old) in self.kept_by.iter().enumerate() {
            if let Some(old) = old {
                keeper_of[*old] = Some(group);
            }
        }
        keeper_of
    }

    fn claimant(&self, group: usize) -> AggregateGroupClaimant {
        AggregateGroupClaimant {
            anchor: self.names[group].anchor.clone(),
            least_member: self.proposal.least[group].clone(),
            live_prior: self.kept_by[group].map(|old| self.proposal.priors[old].header.identity),
        }
    }

    fn retiring_claimant(&self, old: usize) -> Result<AggregateGroupClaimant, Denial> {
        let header = &self.proposal.priors[old].header;
        let least_member = self
            .proposal
            .state
            .least_member(&header.identity, None, &BTreeSet::new())
            .ok_or(Denial::EmptyGroup)?;
        Ok(AggregateGroupClaimant {
            anchor: header.anchor.clone(),
            least_member,
            live_prior: Some(header.identity),
        })
    }

    /// No two live groups may hold one name, and a minted name may never be
    /// the name of a prior group, live or retiring in this same commit.
    ///
    /// Complexity: O(P log P + S log M).
    pub(super) fn guard_live_collisions(&self) -> Result<(), Denial> {
        let collision =
            |identity, first: AggregateGroupClaimant, second: AggregateGroupClaimant| {
                let (first, second) = if first <= second {
                    (first, second)
                } else {
                    (second, first)
                };
                Err(Denial::LiveNameCollision {
                    identity,
                    first,
                    second,
                })
            };
        let mut holders = BTreeMap::new();
        for (group, name) in self.names.iter().enumerate() {
            if let Some(first) = holders.insert(name.identity, group) {
                return collision(name.identity, self.claimant(first), self.claimant(group));
            }
        }
        let keeper_of = self.keeper_of();
        for (old, prior) in self.proposal.priors.iter().enumerate() {
            if keeper_of[old].is_none()
                && let Some(&group) = holders.get(&prior.header.identity)
            {
                let retiring = self.retiring_claimant(old)?;
                return collision(prior.header.identity, retiring, self.claimant(group));
            }
        }
        Ok(())
    }
}

/// The birth anchor of proposed `group`, as [`super::super::keeper::birth_anchor`]
/// names it over the group's complete membership.
fn birth_anchor<V: AggregateGroupStateView>(
    rule: Rule,
    proposal: &Proposal<'_, V>,
    group: usize,
) -> Result<Name, Denial> {
    let least_member = proposal.least[group].clone();
    let anchor = match rule {
        Rule::Anchor => proposal
            .least_with(group, Role::Anchor)
            .or(Some(least_member.clone())),
        Rule::ProducerRole => proposal.least_with(group, Role::Anchor),
        Rule::Boundary => proposal.least_with(group, Role::Boundary),
    };
    anchor.ok_or(Denial::MissingAnchorMember { rule, least_member })
}

/// The proposed group scoped prior `old`'s continuity rule follows, with
/// its score, as [`super::super::keeper::destination`] decides it.
///
/// Complexity: O(touched members of `old` * log M).
fn destination<V: AggregateGroupStateView>(
    rule: Rule,
    proposal: &Proposal<'_, V>,
    old: usize,
) -> Option<(usize, usize)> {
    let prior = &proposal.priors[old];
    let anchor = &prior.header.anchor;
    match rule {
        Rule::Anchor => proposal.new_owner(anchor).map(|(group, _)| (group, 0)),
        Rule::ProducerRole => proposal
            .new_owner(anchor)
            .filter(|&(_, role)| {
                role == Role::Anchor && proposal.prior_role(old, anchor) == Some(role)
            })
            .map(|(group, _)| (group, 0)),
        Rule::Boundary => {
            let mut counts = BTreeMap::<usize, usize>::new();
            let mut touched_boundary = 0u64;
            for (name, member) in &prior.touched {
                if member.role != Role::Boundary {
                    continue;
                }
                touched_boundary += 1;
                if let Some((group, Role::Boundary)) = proposal.new_owner(name) {
                    *counts.entry(group).or_default() += 1;
                }
            }
            // Untouched boundary members stay, with their role, in the residual.
            let untouched = prior
                .header
                .counts
                .of(Role::Boundary)
                .saturating_sub(touched_boundary);
            if untouched > 0
                && let Some(group) = proposal.residual_of[old]
            {
                *counts.entry(group).or_default() += untouched as usize;
            }
            unique_max(counts.into_iter())
        }
    }
}

/// The scoped priors proposed `group` received members from.
fn sources_of<V: AggregateGroupStateView>(
    proposal: &Proposal<'_, V>,
    group: usize,
) -> BTreeSet<AggregateIdentity> {
    match &proposal.groups[group] {
        Shape::Declared(members) => members
            .iter()
            .filter_map(|(name, _)| proposal.old_owner(name))
            .map(|old| proposal.priors[old].header.identity)
            .collect(),
        Shape::Residual(old) => BTreeSet::from([proposal.priors[*old].header.identity]),
    }
}
