//! An in-memory scheme state for the differential court: live groups with
//! complete member sets, the member-owner function and retired names, read
//! through the same view the branch group state implements.
use super::*;

#[derive(Default)]
pub(super) struct MemoryState {
    pub(super) groups: BTreeMap<AggregateIdentity, AggregatePriorGroup>,
    pub(super) owner: BTreeMap<Name, AggregateIdentity>,
    pub(super) retired: BTreeSet<AggregateIdentity>,
}

impl MemoryState {
    /// The full authority's `prior` argument for `scoped` groups, in
    /// identity order.
    pub(super) fn prior(&self, scoped: &BTreeSet<AggregateIdentity>) -> Vec<AggregatePriorGroup> {
        scoped
            .iter()
            .map(|identity| {
                let group = &self.groups[identity];
                AggregatePriorGroup {
                    identity: group.identity,
                    anchor: group.anchor.clone(),
                    sources: group.sources.clone(),
                    members: group.members.clone(),
                }
            })
            .collect()
    }

    /// Applies a full-partition transition: what the branch state holds
    /// after the commit.
    pub(super) fn apply(&mut self, transition: &AggregateGroupPartitionTransition) {
        let gone = transition
            .retired
            .iter()
            .chain(transition.assignments.iter().map(|a| &a.identity));
        for identity in gone {
            if let Some(group) = self.groups.remove(identity) {
                for (name, _) in group.members.iter() {
                    self.owner.remove(name);
                }
            }
        }
        self.retired.extend(transition.retired.iter().copied());
        for assignment in &transition.assignments {
            self.retired.remove(&assignment.identity);
            for (name, _) in assignment.members.iter() {
                self.owner.insert(name.clone(), assignment.identity);
            }
            let group = AggregatePriorGroup {
                identity: assignment.identity,
                anchor: assignment.anchor.clone(),
                sources: assignment.sources.clone(),
                members: assignment.members.clone(),
            };
            self.groups.insert(assignment.identity, group);
        }
    }
}

impl AggregateGroupStateView for MemoryState {
    fn group(&self, identity: &AggregateIdentity) -> Option<AggregateGroupHeader> {
        let group = self.groups.get(identity)?;
        Some(AggregateGroupHeader {
            identity: group.identity,
            anchor: group.anchor.clone(),
            sources: group.sources.clone(),
            counts: AggregateGroupRoleCounts::of_members(&group.members),
        })
    }

    fn membership(&self, name: &Name) -> Option<AggregateGroupMembership> {
        let group = *self.owner.get(name)?;
        let member = self.groups[&group].members.get(name)?.clone();
        Some(AggregateGroupMembership { group, member })
    }

    fn is_retired(&self, identity: &AggregateIdentity) -> bool {
        self.retired.contains(identity)
    }

    fn least_member(
        &self,
        group: &AggregateIdentity,
        role: Option<Role>,
        skip: &BTreeSet<Name>,
    ) -> Option<Name> {
        self.groups
            .get(group)?
            .members
            .iter()
            .filter(|(name, member)| !skip.contains(*name) && role.is_none_or(|r| member.role == r))
            .map(|(name, _)| name.clone())
            .next()
    }

    fn members(&self, group: &AggregateIdentity) -> AggregateGroupMembers {
        self.groups
            .get(group)
            .map(|group| group.members.clone())
            .unwrap_or_default()
    }
}
