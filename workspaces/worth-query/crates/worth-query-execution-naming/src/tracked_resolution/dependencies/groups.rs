//! Group dependencies of tracked name reads, and whether one commit's
//! canonical group lineage stales them.

use std::collections::BTreeSet;

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
        self.branch_mut(branch)
            .group_members
            .insert((scheme.to_owned(), member), answer);
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
    /// Complexity: O(touched groups and members * log dependencies).
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
                stale.extend(deps.group_members.values().flatten().copied());
                return staleness(stale, BTreeSet::new());
            }
        };
        let mut births = BTreeSet::new();
        for (scheme_id, transition) in &transitions {
            let touched: BTreeSet<[u8; 32]> = transition
                .retired
                .iter()
                .chain(transition.assignments.iter().map(|a| &a.identity))
                .map(|group| *group.bytes())
                .collect();
            for group in deps.groups.intersection(&touched) {
                stale.insert(*group);
            }
            for group in &touched {
                if deps.awaited_births.contains(group) {
                    births.insert(*group);
                }
            }
            for ((scheme, member), answer) in &deps.group_members {
                if scheme != scheme_id {
                    continue;
                }
                let regrouped = transition.assignments.iter().any(|assignment| {
                    assignment
                        .members
                        .iter()
                        .any(|(name, _)| name.as_bytes() == member)
                });
                let answered_touched = answer.is_some_and(|group| touched.contains(&group));
                if regrouped || answered_touched {
                    stale.insert(answer.unwrap_or(*member));
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
