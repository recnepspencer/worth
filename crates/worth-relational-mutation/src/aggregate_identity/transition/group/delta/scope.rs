//! The scoped prior groups and the proposed next partition of one scheme,
//! resolved from touched names alone.
use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::AggregateIdentity;
use worth_relational_contracts::internal_port::aggregate_identity::transition::group::model::AggregateGroupMembersOwnerPort as _;

use super::super::super::mint::mint_group_identity;
use super::super::{
    AggregateGroupHeader, AggregateGroupMember, AggregateGroupMembers, AggregateGroupScheme,
    AggregateGroupStateView, Denial, Name, Role,
};
use super::AggregateGroupScope;
use crate::contracts::graph_member::NativeMemberKey;

/// One scoped prior group and what the commit does to its members.
pub(super) struct Prior {
    pub(super) header: AggregateGroupHeader,
    pub(super) dissolved: bool,
    /// The prior members the commit touches, with their prior place.
    pub(super) touched: BTreeMap<Name, AggregateGroupMember>,
    /// Touched members leaving the residual: declared elsewhere or departed.
    pub(super) leaving: BTreeSet<Name>,
    /// Residual members whose carrier changes.
    pub(super) recarried: BTreeMap<Name, NativeMemberKey>,
}

impl Prior {
    /// Members the residual keeps: every prior member that is not leaving.
    fn residual_len(&self) -> u64 {
        self.header
            .counts
            .total()
            .saturating_sub(self.leaving.len() as u64)
    }
}

/// One proposed next-state group.
pub(super) enum Shape {
    /// Declared with its complete membership.
    Declared(AggregateGroupMembers),
    /// The untouched and re-carried members of scoped prior `old`.
    Residual(usize),
}

/// The proposed partition, sorted by least member name.
pub(super) struct Proposal<'v, V> {
    pub(super) state: &'v V,
    pub(super) priors: Vec<Prior>,
    pub(super) prior_index: BTreeMap<AggregateIdentity, usize>,
    /// Touched name to the scoped prior it belonged to.
    touched_owner: BTreeMap<Name, usize>,
    pub(super) departed: BTreeSet<Name>,
    pub(super) groups: Vec<Shape>,
    pub(super) least: Vec<Name>,
    /// Declared member name to its proposed group and role there.
    pub(super) declared_owner: BTreeMap<Name, (usize, Role)>,
    /// Scoped prior to the proposed position of its residual, if nonempty.
    pub(super) residual_of: Vec<Option<usize>>,
}

impl<'v, V: AggregateGroupStateView> Proposal<'v, V> {
    /// Complexity: O((T + S + P) log M), with the full authority's refusal
    /// order: prior checks, then emptiness, overlaps and carriers in least
    /// name order.
    pub(super) fn resolve(
        scheme: &AggregateGroupScheme,
        state: &'v V,
        scope: AggregateGroupScope,
    ) -> Result<Self, Denial> {
        let priors = scoped_priors(scheme, state, &scope)?;
        let prior_index = priors
            .iter()
            .enumerate()
            .map(|(old, prior)| (prior.header.identity, old))
            .collect();
        let touched_owner = priors
            .iter()
            .enumerate()
            .flat_map(|(old, prior)| prior.touched.keys().map(move |name| (name.clone(), old)))
            .collect();
        let mut proposed: Vec<(Option<Name>, Shape)> = scope
            .declared
            .into_iter()
            .map(|members| (members.least_name(), Shape::Declared(members)))
            .collect();
        for (old, prior) in priors.iter().enumerate() {
            if prior.dissolved || prior.residual_len() == 0 {
                continue;
            }
            let least = state.least_member(&prior.header.identity, None, &prior.leaving);
            proposed.push((least, Shape::Residual(old)));
        }
        proposed.sort_by(|(left, _), (right, _)| left.cmp(right));
        let mut proposal = Self {
            state,
            residual_of: vec![None; priors.len()],
            priors,
            prior_index,
            touched_owner,
            departed: scope.departed,
            groups: Vec::with_capacity(proposed.len()),
            least: Vec::with_capacity(proposed.len()),
            declared_owner: BTreeMap::new(),
        };
        proposal.admit(proposed)?;
        Ok(proposal)
    }

    /// Admits the sorted proposed groups, refusing an empty group, a member
    /// declared twice, or one carrier declared for two members.
    fn admit(&mut self, proposed: Vec<(Option<Name>, Shape)>) -> Result<(), Denial> {
        let mut carriers_seen = BTreeSet::new();
        for (group, (least, shape)) in proposed.into_iter().enumerate() {
            let least = least.ok_or(Denial::EmptyGroup)?;
            match &shape {
                Shape::Declared(members) => {
                    for (name, member) in members.iter() {
                        if self
                            .declared_owner
                            .insert(name.clone(), (group, member.role))
                            .is_some()
                        {
                            return Err(Denial::OverlappingMember { name: name.clone() });
                        }
                        if !carriers_seen.insert(member.carrier.clone()) {
                            return Err(Denial::DuplicateCarrier { name: name.clone() });
                        }
                    }
                }
                Shape::Residual(old) => self.residual_of[*old] = Some(group),
            }
            self.least.push(least);
            self.groups.push(shape);
        }
        Ok(())
    }

    /// The scoped prior `name` belonged to, if any.
    ///
    /// Complexity: O(log T), or O(log M) for an untouched name.
    pub(super) fn old_owner(&self, name: &Name) -> Option<usize> {
        if let Some(old) = self.touched_owner.get(name) {
            return Some(*old);
        }
        let membership = self.state.membership(name)?;
        self.prior_index.get(&membership.group).copied()
    }

    /// The proposed group `name` belongs to and its role there.
    ///
    /// Complexity: O(log M).
    pub(super) fn new_owner(&self, name: &Name) -> Option<(usize, Role)> {
        if let Some(owner) = self.declared_owner.get(name) {
            return Some(*owner);
        }
        if self.departed.contains(name) {
            return None;
        }
        let membership = self.state.membership(name)?;
        let old = *self.prior_index.get(&membership.group)?;
        let group = self.residual_of[old]?;
        Some((group, membership.member.role))
    }

    /// `name`'s role in scoped prior `old`, if it was a member there.
    pub(super) fn prior_role(&self, old: usize, name: &Name) -> Option<Role> {
        let prior = &self.priors[old];
        if let Some(member) = prior.touched.get(name) {
            return Some(member.role);
        }
        let membership = self.state.membership(name)?;
        (membership.group == prior.header.identity).then_some(membership.member.role)
    }

    /// The least member of proposed `group` holding `role`.
    pub(super) fn least_with(&self, group: usize, role: Role) -> Option<Name> {
        match &self.groups[group] {
            Shape::Declared(members) => members.least_with(role),
            Shape::Residual(old) => {
                let prior = &self.priors[*old];
                let identity = &prior.header.identity;
                self.state
                    .least_member(identity, Some(role), &prior.leaving)
            }
        }
    }

    /// Every member of proposed `group`.
    ///
    /// Complexity: O(|group|): only for a born or revived group, or a kept
    /// group diffed against a prior other than its own residual's.
    pub(super) fn materialize(&self, group: usize) -> Result<AggregateGroupMembers, Denial> {
        match &self.groups[group] {
            Shape::Declared(members) => Ok(members.clone()),
            Shape::Residual(old) => {
                let prior = &self.priors[*old];
                let mut members = AggregateGroupMembers::new();
                for (name, member) in self.state.members(&prior.header.identity).iter() {
                    if prior.leaving.contains(name) {
                        continue;
                    }
                    let carrier = prior.recarried.get(name).unwrap_or(&member.carrier);
                    members.insert(name.clone(), carrier.clone(), member.role)?;
                }
                Ok(members)
            }
        }
    }

    /// Every prior member of scoped prior `old`: its touched members when
    /// the commit touches all of them, else the state's walk.
    pub(super) fn prior_members(&self, old: usize) -> Result<AggregateGroupMembers, Denial> {
        let prior = &self.priors[old];
        if prior.touched.len() as u64 != prior.header.counts.total() {
            return Ok(self.state.members(&prior.header.identity));
        }
        let mut members = AggregateGroupMembers::new();
        for (name, member) in &prior.touched {
            members.insert(name.clone(), member.carrier.clone(), member.role)?;
        }
        Ok(members)
    }
}

/// Every scoped prior group, in identity order, with what the commit does
/// to its members: dissolved groups and the owners of touched names.
///
/// Complexity: O((T + S) log M).
fn scoped_priors(
    scheme: &AggregateGroupScheme,
    state: &impl AggregateGroupStateView,
    scope: &AggregateGroupScope,
) -> Result<Vec<Prior>, Denial> {
    let declared: BTreeSet<Name> = scope
        .declared
        .iter()
        .flat_map(|members| members.iter().map(|(name, _)| name.clone()))
        .collect();
    let mut touched: BTreeMap<AggregateIdentity, BTreeMap<Name, AggregateGroupMember>> = scope
        .dissolved
        .iter()
        .map(|id| (*id, BTreeMap::new()))
        .collect();
    let names = declared
        .iter()
        .chain(scope.departed.iter())
        .chain(scope.carriers.keys());
    for name in names {
        if let Some(membership) = state.membership(name) {
            let group = touched.entry(membership.group).or_default();
            group.insert(name.clone(), membership.member);
        }
    }
    let mut priors = Vec::with_capacity(touched.len());
    for (identity, touched) in touched {
        let header = state
            .group(&identity)
            .ok_or(Denial::UnknownGroup { identity })?;
        validate_prior(scheme, state, &header)?;
        let leaving: BTreeSet<Name> = touched
            .keys()
            .filter(|name| declared.contains(*name) || scope.departed.contains(*name))
            .cloned()
            .collect();
        let recarried = touched
            .iter()
            .filter(|(name, _)| !leaving.contains(*name))
            .filter_map(|(name, prior)| {
                let carrier = scope.carriers.get(name)?;
                (*carrier != prior.carrier).then(|| (name.clone(), carrier.clone()))
            })
            .collect();
        priors.push(Prior {
            dissolved: scope.dissolved.contains(&identity),
            header,
            touched,
            leaving,
            recarried,
        });
    }
    Ok(priors)
}

/// The full authority's prior checks, on one scoped group's header.
fn validate_prior(
    scheme: &AggregateGroupScheme,
    state: &impl AggregateGroupStateView,
    header: &AggregateGroupHeader,
) -> Result<(), Denial> {
    let identity = header.identity;
    let canonical = header.sources.windows(2).all(|pair| pair[0] < pair[1]);
    if !canonical
        || identity
            != mint_group_identity(scheme.id(), &header.anchor, scheme.rule(), &header.sources)
    {
        return Err(Denial::PriorIdentityMismatch { identity });
    }
    if state.is_retired(&identity) {
        return Err(Denial::RetiredIdentityLive { identity });
    }
    if header.counts.total() == 0 {
        return Err(Denial::EmptyGroup);
    }
    Ok(())
}
