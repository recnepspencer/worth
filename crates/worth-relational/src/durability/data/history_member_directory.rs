//! Immutable direct dependencies of layered history members, without payloads.
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RelationalCheckpointHistoryDependency {
    pub(crate) member: RelationalDurableMemberLocator,
    pub(crate) bases: Vec<RelationalDurableMemberLocator>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct RelationalHistoryMemberDirectory {
    entries: BTreeMap<RelationalDurableMemberLocator, Vec<RelationalDurableMemberLocator>>,
}

impl RelationalHistoryMemberDirectory {
    pub(crate) fn readmit(
        declarations: &[RelationalCheckpointHistoryDependency],
        roots: impl IntoIterator<Item = RelationalDurableMemberLocator>,
    ) -> Result<Self, Cause> {
        let mut entries = BTreeMap::new();
        let mut previous = None;
        for declaration in declarations {
            if previous.is_some_and(|member| member >= declaration.member)
                || declaration.bases.is_empty()
                || declaration.bases.windows(2).any(|pair| pair[0] >= pair[1])
                || declaration.bases.contains(&declaration.member)
            {
                return Err(Cause::Damaged);
            }
            previous = Some(declaration.member);
            entries.insert(declaration.member, declaration.bases.clone());
        }
        let directory = Self { entries };
        directory.verify_reachable_acyclic(roots)?;
        Ok(directory)
    }

    pub(crate) fn bases(
        &self,
        member: RelationalDurableMemberLocator,
    ) -> &[RelationalDurableMemberLocator] {
        self.entries.get(&member).map_or(&[], Vec::as_slice)
    }

    pub(crate) fn members(
        &self,
        root: RelationalDurableMemberLocator,
    ) -> BTreeSet<RelationalDurableMemberLocator> {
        let mut members = BTreeSet::new();
        let mut pending = vec![root];
        while let Some(member) = pending.pop() {
            if members.insert(member) {
                pending.extend(self.bases(member).iter().copied());
            }
        }
        members
    }

    pub(crate) fn declarations_for(
        &self,
        members: &BTreeSet<RelationalDurableMemberLocator>,
    ) -> Vec<RelationalCheckpointHistoryDependency> {
        members
            .iter()
            .filter_map(|member| {
                self.entries
                    .get(member)
                    .map(|bases| RelationalCheckpointHistoryDependency {
                        member: *member,
                        bases: bases.clone(),
                    })
            })
            .collect()
    }

    fn verify_reachable_acyclic(
        &self,
        roots: impl IntoIterator<Item = RelationalDurableMemberLocator>,
    ) -> Result<(), Cause> {
        let mut active = BTreeSet::new();
        let mut complete = BTreeSet::new();
        for root in roots {
            let mut pending = vec![(root, false)];
            while let Some((member, leaving)) = pending.pop() {
                if leaving {
                    active.remove(&member);
                    complete.insert(member);
                } else if !complete.contains(&member) {
                    if !active.insert(member) {
                        return Err(Cause::Damaged);
                    }
                    pending.push((member, true));
                    pending.extend(self.bases(member).iter().rev().map(|base| (*base, false)));
                }
            }
        }
        if self.entries.keys().any(|member| !complete.contains(member)) {
            return Err(Cause::Damaged);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(index: u32) -> RelationalDurableMemberLocator {
        let mut sha256 = [0; 32];
        sha256[..4].copy_from_slice(&index.to_be_bytes());
        RelationalDurableMemberLocator { sha256, length: 1 }
    }

    fn edge(index: u32, bases: &[u32]) -> RelationalCheckpointHistoryDependency {
        RelationalCheckpointHistoryDependency {
            member: member(index),
            bases: bases.iter().map(|base| member(*base)).collect(),
        }
    }

    #[test]
    fn each_root_exposes_only_its_dependency_members() {
        let declarations = vec![edge(1, &[3, 4]), edge(2, &[4]), edge(3, &[4])];
        let directory =
            RelationalHistoryMemberDirectory::readmit(&declarations, [member(1), member(2)])
                .unwrap();
        let first = directory.members(member(1));
        assert_eq!(first, BTreeSet::from([member(1), member(3), member(4)]));
        assert_eq!(
            directory.members(member(2)),
            BTreeSet::from([member(2), member(4)])
        );
        assert_eq!(
            directory.declarations_for(&first),
            vec![edge(1, &[3, 4]), edge(3, &[4])]
        );
    }

    #[test]
    fn cycles_ambiguous_order_and_unreachable_declarations_are_refused() {
        for declarations in [
            vec![edge(1, &[1])],
            vec![edge(1, &[2]), edge(2, &[1])],
            vec![edge(1, &[2]), edge(1, &[3])],
            vec![edge(2, &[3]), edge(1, &[3])],
            vec![edge(1, &[3, 2])],
            vec![edge(1, &[2, 2])],
            vec![edge(1, &[])],
            vec![edge(1, &[2]), edge(3, &[4])],
        ] {
            assert_eq!(
                RelationalHistoryMemberDirectory::readmit(&declarations, [member(1)]),
                Err(Cause::Damaged)
            );
        }
    }

    #[test]
    fn a_deep_history_directory_uses_iterative_admission_and_traversal() {
        let declarations = (1..16_384)
            .map(|index| edge(index, &[index + 1]))
            .collect::<Vec<_>>();
        let directory =
            RelationalHistoryMemberDirectory::readmit(&declarations, [member(1)]).unwrap();
        let closure = directory.members(member(1));
        for index in 1..=16_384 {
            assert!(closure.contains(&member(index)));
        }
    }
}
