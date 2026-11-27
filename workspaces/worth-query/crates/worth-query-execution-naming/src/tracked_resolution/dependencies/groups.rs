//! Group dependencies of tracked name reads, and whether one commit's
//! canonical group lineage stales them.

use std::collections::{BTreeMap, BTreeSet};

use worth_relational::facade::aggregate_identity::AggregateGroupTransition;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::PersistentNameDurabilityError;

use super::{NameReadCurrentness, NameReadDependencies, NameReadStaleness};

impl NameReadDependencies {
    /// Records the groups a group resolution rested on.
    pub(crate) fn record_groups(
        &mut self,
        branch: &BranchId,
        groups: impl IntoIterator<Item = [u8; 32]>,
    ) {
        self.branch_mut(branch).groups.extend(groups);
    }

    /// Records one member's group lookup under `scheme` and its answer.
    pub(crate) fn record_group_member(
        &mut self,
        branch: &BranchId,
        scheme: &str,
        member: [u8; 32],
        answer: Option<[u8; 32]>,
    ) {
        let deps = self.branch_mut(branch);
        let lookups = deps.group_members.entry(scheme.to_owned()).or_default();
        let earlier = lookups.insert(member, answer).flatten();
        let by_group = deps.members_by_group.entry(scheme.to_owned()).or_default();
        if let Some(group) = earlier
            && let Some(members) = by_group.get_mut(&group)
        {
            members.remove(&member);
            if members.is_empty() {
                by_group.remove(&group);
            }
        }
        if let Some(group) = answer {
            by_group.entry(group).or_default().insert(member);
        }
    }

    /// Whether a read on `branch` rested on group state.
    pub fn reads_groups_on(&self, branch: &BranchId) -> bool {
        self.branches
            .get(branch)
            .is_some_and(|deps| !deps.groups.is_empty() || !deps.group_members.is_empty())
    }

    /// Whether one commit's group lineage on `branch` stales a recorded read.
    /// A frame that cannot be decoded stales every group read, never none.
    ///
    /// Complexity: O((touched groups + assigned members) * log dependencies):
    /// each touched group and each assigned member is one probe; no recorded
    /// dependency the commit does not touch is ever visited.
    pub fn currentness_after_group_lineage(
        &self,
        branch: &BranchId,
        lineage: Result<Vec<(String, AggregateGroupTransition)>, PersistentNameDurabilityError>,
    ) -> NameReadCurrentness {
        let Some(deps) = self.branches.get(branch) else {
            return NameReadCurrentness::Clean;
        };
        let mut stale = BTreeSet::new();
        let transitions = match lineage {
            Ok(transitions) => transitions,
            Err(_) => {
                stale.extend(deps.groups.iter().copied());
                let lookups = deps.group_members.values().flat_map(BTreeMap::iter);
                stale.extend(lookups.map(|(member, answer)| answer.unwrap_or(*member)));
                return staleness(stale, deps.awaited_births.clone());
            }
        };
        let mut births = BTreeSet::new();
        for (scheme_id, transition) in &transitions {
            let touched = transition
                .retired
                .iter()
                .chain(transition.assignments.iter().map(|a| &a.identity))
                .map(|group| *group.bytes());
            let lookups = deps.group_members.get(scheme_id);
            let answered = deps.members_by_group.get(scheme_id);
            for group in touched {
                if deps.groups.contains(&group) {
                    stale.insert(group);
                }
                if deps.awaited_births.contains(&group) {
                    births.insert(group);
                }
                // A member whose answered group the commit touched.
                if answered.is_some_and(|by_group| by_group.contains_key(&group)) {
                    stale.insert(group);
                }
            }
            // A member the commit regrouped: one probe per assigned member.
            let Some(lookups) = lookups else { continue };
            let assigned = transition.assignments.iter();
            for (name, _) in assigned.flat_map(|assignment| assignment.members.iter()) {
                if let Some(answer) = lookups.get(name.as_bytes()) {
                    stale.insert(answer.unwrap_or(*name.as_bytes()));
                }
            }
        }
        staleness(stale, births)
    }
}

fn staleness(groups: BTreeSet<[u8; 32]>, births: BTreeSet<[u8; 32]>) -> NameReadCurrentness {
    if groups.is_empty() && births.is_empty() {
        return NameReadCurrentness::Clean;
    }
    NameReadCurrentness::Stale(NameReadStaleness {
        lineages: Vec::new(),
        births: births.into_iter().collect(),
        groups: groups.into_iter().collect(),
    })
}

impl NameReadCurrentness {
    /// Both verdicts over one commit: stale when either is.
    pub fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::Clean, other) | (other, Self::Clean) => other,
            (Self::Stale(first), Self::Stale(second)) => {
                let mut merged = first;
                merged.lineages.extend(second.lineages);
                merged.births.extend(second.births);
                merged.births.sort_unstable();
                merged.births.dedup();
                merged.groups.extend(second.groups);
                Self::Stale(merged)
            }
        }
    }
}
