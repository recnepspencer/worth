//! Canonical assignments, retirements and lineage events of one incremental
//! group transition: a kept group records only its member delta.
use std::collections::BTreeSet;

use worth_relational_contracts::internal_port::aggregate_identity::transition::group::transition_result::member_delta;

use super::super::{
    AggregateGroupAssignment, AggregateGroupChange, AggregateGroupMemberDelta,
    AggregateGroupStateView, AggregateGroupTransition, Denial, Event,
};
use super::naming::{GroupName, Plan};
use super::scope::Shape;

impl<V: AggregateGroupStateView> Plan<'_, V> {
    /// Complexity: O((T + S + P) log M) plus each born or revived group's
    /// members.
    pub(super) fn assemble(self) -> Result<AggregateGroupTransition, Denial> {
        let keeper_of = self.keeper_of();
        let home = |old: usize| keeper_of[old].or(self.destination[old].map(|(group, _)| group));
        let proposal = &self.proposal;
        let mut result = AggregateGroupTransition {
            assignments: Vec::with_capacity(proposal.groups.len()),
            retired: BTreeSet::new(),
            lineage: BTreeSet::new(),
        };
        for (group, shape) in proposal.groups.iter().enumerate() {
            let GroupName {
                identity,
                anchor,
                sources,
            } = &self.names[group];
            let identity = *identity;
            let change = match self.kept_by[group] {
                Some(old) => AggregateGroupChange::Kept(self.kept_delta(old, group)?),
                None if proposal.state.is_retired(&identity) => {
                    result.lineage.insert(Event::Revived { group: identity });
                    AggregateGroupChange::Revived(proposal.materialize(group)?)
                }
                None => {
                    result.lineage.insert(Event::Born { group: identity });
                    AggregateGroupChange::Born(proposal.materialize(group)?)
                }
            };
            let fragment_sources: BTreeSet<usize> = match shape {
                Shape::Declared(members) => members
                    .iter()
                    .filter_map(|(name, _)| proposal.old_owner(name))
                    .collect(),
                Shape::Residual(old) => BTreeSet::from([*old]),
            };
            for old in fragment_sources {
                if home(old) != Some(group) {
                    result.lineage.insert(Event::SplitFrom {
                        fragment: identity,
                        source: proposal.priors[old].header.identity,
                    });
                }
            }
            result.assignments.push(AggregateGroupAssignment {
                identity,
                anchor: anchor.clone(),
                sources: sources.clone(),
                change,
            });
        }
        for (old, prior) in proposal.priors.iter().enumerate() {
            if keeper_of[old].is_some() {
                continue;
            }
            let identity = prior.header.identity;
            result.retired.insert(identity);
            result.lineage.insert(Event::Retired { group: identity });
            if let Some((into, _)) = self.destination[old] {
                result.lineage.insert(Event::MergedInto {
                    absorbed: identity,
                    keeper: self.names[into].identity,
                });
            }
        }
        result
            .assignments
            .sort_by_key(|assignment| assignment.identity);
        Ok(result)
    }

    /// The member delta of proposed `group`, which keeps scoped prior `old`.
    ///
    /// Complexity: O(touched members of `old`) when `group` is `old`'s own
    /// residual (the `k`-member edit), else a diff of both memberships.
    fn kept_delta(&self, old: usize, group: usize) -> Result<AggregateGroupMemberDelta, Denial> {
        let proposal = &self.proposal;
        if let Shape::Residual(residual) = proposal.groups[group]
            && residual == old
        {
            let prior = &proposal.priors[old];
            let mut delta = AggregateGroupMemberDelta::new();
            for name in &prior.leaving {
                delta.remove(name.clone())?;
            }
            for (name, carrier) in &prior.recarried {
                let role = prior.touched[name].role;
                delta.upsert(name.clone(), carrier.clone(), role)?;
            }
            return Ok(delta);
        }
        // A declared group keeping `old` while `old`'s untouched members stay
        // behind in a residual renames that residual: its members leave
        // `old` either way, so this diff is bounded by the born residual.
        member_delta(&proposal.prior_members(old)?, &proposal.materialize(group)?)
    }
}
