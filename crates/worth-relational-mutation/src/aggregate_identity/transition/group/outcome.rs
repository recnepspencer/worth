//! Collision guard and lineage emission for one chosen group naming plan.
use std::collections::{BTreeMap, BTreeSet};
use worth_relational_contracts::internal_port::aggregate_identity::transition::group::model::AggregateGroupMembersOwnerPort as _;

use worth_foundational::facade::AggregateIdentity;

use super::{
    AggregateGroupClaimant, AggregateGroupMembers, AggregateGroupOrigin,
    AggregateGroupPartitionAssignment, AggregateGroupPartitionTransition, AggregatePriorGroup,
    Denial, Event, GroupName, Name,
};

/// Every naming decision, before any effect exists.
pub(super) struct Plan<'a> {
    pub(super) prior: &'a [AggregatePriorGroup],
    pub(super) proposed: Vec<AggregateGroupMembers>,
    pub(super) names: Vec<GroupName>,
    pub(super) kept_by: Vec<Option<usize>>,
    pub(super) destination: Vec<Option<(usize, usize)>>,
    pub(super) old_owner: BTreeMap<Name, usize>,
}

impl Plan<'_> {
    fn keeper_of(&self) -> Vec<Option<usize>> {
        let mut keeper_of = vec![None; self.prior.len()];
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
            least_member: self.proposed[group].least_name().expect("nonempty group"),
            live_prior: self.kept_by[group].map(|old| self.prior[old].identity),
        }
    }

    fn retiring_claimant(&self, old: usize) -> AggregateGroupClaimant {
        let group = &self.prior[old];
        AggregateGroupClaimant {
            anchor: group.anchor.clone(),
            least_member: group.members.least_name().expect("nonempty prior"),
            live_prior: Some(group.identity),
        }
    }

    /// No two live groups may hold one name, and a minted name may never be
    /// the name of a prior group, live or retiring in this same commit.
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
        for (old, prior) in self.prior.iter().enumerate() {
            if keeper_of[old].is_none()
                && let Some(&group) = holders.get(&prior.identity)
            {
                return collision(
                    prior.identity,
                    self.retiring_claimant(old),
                    self.claimant(group),
                );
            }
        }
        Ok(())
    }

    /// Builds assignments, retirements, and canonical lineage records.
    pub(super) fn assemble(
        self,
        is_retired: &impl Fn(&AggregateIdentity) -> bool,
    ) -> AggregateGroupPartitionTransition {
        let keeper_of = self.keeper_of();
        let mut result = AggregateGroupPartitionTransition {
            assignments: Vec::with_capacity(self.proposed.len()),
            retired: BTreeSet::new(),
            lineage: BTreeSet::new(),
        };
        let home = |old: usize| keeper_of[old].or(self.destination[old].map(|(group, _)| group));
        for (group, members) in self.proposed.into_iter().enumerate() {
            let GroupName {
                identity,
                anchor,
                sources,
            } = self.names[group].clone();
            let origin = match self.kept_by[group] {
                Some(_) => AggregateGroupOrigin::Kept,
                None if is_retired(&identity) => AggregateGroupOrigin::Revived,
                None => AggregateGroupOrigin::Born,
            };
            match origin {
                AggregateGroupOrigin::Born => {
                    result.lineage.insert(Event::Born { group: identity });
                }
                AggregateGroupOrigin::Revived => {
                    result.lineage.insert(Event::Revived { group: identity });
                }
                AggregateGroupOrigin::Kept => {}
            }
            for (name, _) in members.iter() {
                if let Some(&old) = self.old_owner.get(name)
                    && home(old) != Some(group)
                {
                    result.lineage.insert(Event::SplitFrom {
                        fragment: identity,
                        source: self.prior[old].identity,
                    });
                }
            }
            result.assignments.push(AggregateGroupPartitionAssignment {
                identity,
                anchor,
                sources,
                members,
                origin,
            });
        }
        for (old, group) in self.prior.iter().enumerate() {
            if keeper_of[old].is_some() {
                continue;
            }
            result.retired.insert(group.identity);
            result.lineage.insert(Event::Retired {
                group: group.identity,
            });
            if let Some((into, _)) = self.destination[old] {
                result.lineage.insert(Event::MergedInto {
                    absorbed: group.identity,
                    keeper: self.names[into].identity,
                });
            }
        }
        result
            .assignments
            .sort_by_key(|assignment| assignment.identity);
        result
    }
}
