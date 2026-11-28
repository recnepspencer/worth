//! Installation of one verified canonical group transition into a scheme's
//! tables: a kept group's delta writes only its touched member rows and its
//! header's counts; a retired group removes its rows; a born or revived
//! group writes its initial members.
use std::collections::BTreeSet;

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::transition::group::delta::AggregateGroupChange;
use crate::contracts::aggregate_identity::transition::group::model::AggregateMemberRole as Role;
use crate::contracts::aggregate_identity::transition::group::model::CanonicalGroupMemberIdentity as Name;
use crate::contracts::aggregate_identity::transition::group::state_view::{
    AggregateGroupHeader, AggregateGroupRoleCounts,
};
use crate::contracts::graph_member::NativeMemberKey;
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;

use super::branch_state::SchemeGroups;
use super::group_rows::{GroupMemberKey, GroupOwnerRow, ROLES};
use super::CommittedGroupTransition;

impl SchemeGroups {
    /// Every member row key of live group `group`, ascending.
    ///
    /// Complexity: O(M + log N): one walk of the group's member range.
    pub(super) fn member_keys(
        &self,
        group: AggregateIdentity,
    ) -> Result<Vec<GroupMemberKey>, Cause> {
        let mut keys = Vec::new();
        self.scan_members(group, None, |key, _| {
            keys.push(key);
            Ok(true)
        })?;
        Ok(keys)
    }

    /// Visits `group`'s member rows holding `role` (every role when `None`),
    /// ascending by role then name, until `visit` returns `false`.
    ///
    /// Complexity: O(log N + visited rows).
    pub(super) fn scan_members(
        &self,
        group: AggregateIdentity,
        role: Option<Role>,
        mut visit: impl FnMut(GroupMemberKey, &NativeMemberKey) -> Result<bool, Cause>,
    ) -> Result<(), Cause> {
        let lower = self.key(GroupMemberKey::least(group, role.unwrap_or(ROLES[0])));
        let mut prefix = self.members.row_key(&lower);
        // Drop the name (and, for every role, the role byte) from the seek key.
        prefix.truncate(prefix.len() - 33 - usize::from(role.is_none()));
        self.members
            .scan_prefix_from(&lower, &prefix, |(_, key), carrier| visit(key, &carrier))
    }

    /// Applies one transition the scheme verified.
    ///
    /// Complexity: O(touched members * log N), plus each retired group's
    /// member rows.
    pub(super) fn install(&mut self, committed: &CommittedGroupTransition) -> Result<(), Cause> {
        let transition = &committed.transition;
        for retired in &transition.retired {
            self.remove_group(*retired)?;
            self.retired.insert(self.key(*retired), ());
        }
        // Every departure before any arrival: a member moving between two
        // groups of the scheme ends in its new group.
        let mut kept = Vec::new();
        for assignment in &transition.assignments {
            if let AggregateGroupChange::Kept(delta) = &assignment.change {
                let header = self.live.get(&self.key(assignment.identity))?;
                let mut header = header.ok_or(Cause::Damaged)?.into_owned();
                let leaving = delta
                    .removed()
                    .chain(delta.upserted().map(|(name, _)| name));
                for name in leaving {
                    if let Some(role) = self.release(assignment.identity, name.clone())? {
                        header.counts.remove(role).ok_or(Cause::Damaged)?;
                    }
                }
                kept.push(header);
            }
        }
        let mut kept = kept.into_iter();
        for assignment in &transition.assignments {
            let identity = assignment.identity;
            match &assignment.change {
                AggregateGroupChange::Kept(delta) => {
                    let mut header = kept.next().ok_or(Cause::Damaged)?;
                    for (name, member) in delta.upserted() {
                        self.join(identity, name.clone(), &member.carrier, member.role);
                        header.counts.add(member.role);
                    }
                    self.live.insert(self.key(identity), header);
                }
                AggregateGroupChange::Born(members) | AggregateGroupChange::Revived(members) => {
                    let key = self.key(identity);
                    if self.retired.contains_key(&key)? {
                        self.retired.remove(&key)?;
                    }
                    for (name, member) in members.iter() {
                        self.join(identity, name.clone(), &member.carrier, member.role);
                    }
                    let header = AggregateGroupHeader {
                        identity,
                        anchor: assignment.anchor.clone(),
                        sources: assignment.sources.clone(),
                        counts: AggregateGroupRoleCounts::of_members(members),
                    };
                    self.live.insert(key, header);
                }
            }
        }
        Ok(())
    }

    /// Removes `name`'s rows from `group`; returns its role there, or `None`
    /// when it was not `group`'s member.
    fn release(&mut self, group: AggregateIdentity, name: Name) -> Result<Option<Role>, Cause> {
        let owner_key = self.key(name.clone());
        let Some(owner) = self.member_owner.get(&owner_key)?.map(|row| *row) else {
            return Ok(None);
        };
        if owner.group != group {
            return Ok(None);
        }
        self.member_owner.remove_blind(&owner_key);
        let member = GroupMemberKey {
            group,
            role: owner.role,
            name,
        };
        self.members.remove_blind(&self.key(member));
        Ok(Some(owner.role))
    }

    fn join(
        &mut self,
        group: AggregateIdentity,
        name: Name,
        carrier: &NativeMemberKey,
        role: Role,
    ) {
        let member = GroupMemberKey {
            group,
            role,
            name: name.clone(),
        };
        self.members.insert(self.key(member), carrier.clone());
        self.member_owner
            .insert(self.key(name), GroupOwnerRow { group, role });
    }

    /// Removes a retired group's header, member rows and owned owner rows.
    fn remove_group(&mut self, group: AggregateIdentity) -> Result<(), Cause> {
        let names: BTreeSet<Name> = self
            .member_keys(group)?
            .into_iter()
            .map(|key| key.name)
            .collect();
        for name in names {
            self.release(group, name)?;
        }
        self.live.remove_blind(&self.key(group));
        Ok(())
    }
}
