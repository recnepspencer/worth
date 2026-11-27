//! Read views of a branch's group state inside one read boundary's scope:
//! point and ordered probes for the incremental group authority, and
//! complete group walks only for history selection.
use std::collections::BTreeSet;

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::transition::group::model::{
    AggregateContinuityRule, AggregateGroupMember, AggregateGroupMembers, AggregateMemberRole,
    AggregatePriorGroup,
};
use crate::contracts::aggregate_identity::transition::group::state_view::{
    AggregateGroupHeader, AggregateGroupMembership, AggregateGroupStateView,
};
use crate::contracts::lineage::data::naming::PersistentNameIdentity;
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use crate::lineage::naming_pages::tables::scoped_prefix;
use crate::lineage::tiered::NamingReadScope;

use super::branch_state::{BranchGroupState, GroupStateDenial, SchemeGroups};
use super::group_rows::{GroupMemberKey, GroupOwnerRow, ROLES};
use super::CommittedGroupTransition;

/// A branch's group state as one read boundary sees it.
#[derive(Clone, Copy, Debug)]
pub struct GroupStateRead<'s> {
    state: &'s BranchGroupState,
    scope: &'s NamingReadScope,
}

/// One installed scheme's groups as one read boundary sees them.
#[derive(Clone, Copy, Debug)]
pub struct SchemeGroupsRead<'s> {
    groups: &'s SchemeGroups,
    scope: &'s NamingReadScope,
}

impl BranchGroupState {
    /// This state read inside `scope`.
    pub fn read<'s>(&'s self, scope: &'s NamingReadScope) -> GroupStateRead<'s> {
        GroupStateRead { state: self, scope }
    }
}

impl<'s> GroupStateRead<'s> {
    pub fn scheme(&self, scheme_id: &str) -> Option<SchemeGroupsRead<'s>> {
        let groups = self.state.schemes.get(scheme_id)?;
        Some(SchemeGroupsRead {
            groups,
            scope: self.scope,
        })
    }

    pub fn schemes(&self) -> impl Iterator<Item = (&'s str, SchemeGroupsRead<'s>)> + use<'s> {
        let scope = self.scope;
        self.state
            .schemes
            .iter()
            .map(move |(id, groups)| (id.as_str(), SchemeGroupsRead { groups, scope }))
    }

    pub const fn latest_commit(
        &self,
    ) -> Option<crate::contracts::history::data::coordinates::CommitId> {
        self.state.latest_commit
    }

    pub fn verify(&self, transitions: &[CommittedGroupTransition]) -> Result<(), GroupStateDenial> {
        self.state.verify(transitions)
    }
}

impl<'s> SchemeGroupsRead<'s> {
    pub const fn rule(&self) -> AggregateContinuityRule {
        self.groups.rule
    }

    /// The live group `identity`'s header.
    ///
    /// Complexity: O(log G) and at most one page fault.
    pub fn header(&self, identity: &AggregateIdentity) -> Option<&'s AggregateGroupHeader> {
        self.groups
            .live
            .get_in(&self.groups.key(*identity), self.scope)
    }

    /// The live group `member` belongs to.
    ///
    /// Complexity: O(log M) and at most one page fault.
    pub fn owner_of(&self, member: &PersistentNameIdentity) -> Option<AggregateIdentity> {
        self.owner_row(member).map(|row| row.group)
    }

    fn owner_row(&self, member: &PersistentNameIdentity) -> Option<&'s GroupOwnerRow> {
        self.groups
            .member_owner
            .get_in(&self.groups.key(*member), self.scope)
    }

    /// Whether `identity` is a retired, unrevived group name.
    pub fn is_retired(&self, identity: &AggregateIdentity) -> bool {
        self.groups
            .retired
            .get_in(&self.groups.key(*identity), self.scope)
            .is_some()
    }

    /// The live group `identity` with its complete membership.
    ///
    /// Complexity: O(M log M): a walk of the group's member rows.
    pub fn group(&self, identity: &AggregateIdentity) -> Option<AggregatePriorGroup> {
        let header = self.header(identity)?;
        Some(AggregatePriorGroup {
            identity: header.identity,
            anchor: header.anchor,
            sources: header.sources.clone(),
            members: self.members(identity),
        })
    }

    /// Every live group in identity order with its complete membership: a
    /// scan of this scheme's range, for history selection.
    ///
    /// Complexity: O(live groups and members of the scheme).
    pub fn live(&self) -> Vec<AggregatePriorGroup> {
        let mut headers = Vec::new();
        let prefix = scoped_prefix(self.groups.live.table(), &self.groups.id);
        let scanned = self.groups.live.scan_merged(&prefix, |_, header| {
            headers.push(header.into_owned());
            Ok(true)
        });
        if let Err(cause) = scanned {
            self.scope.fail(cause);
        }
        headers
            .into_iter()
            .map(|header| AggregatePriorGroup {
                members: self.members(&header.identity),
                identity: header.identity,
                anchor: header.anchor,
                sources: header.sources,
            })
            .collect()
    }

    fn latch<T>(&self, read: Result<T, Cause>) -> Option<T> {
        read.map_err(|cause| self.scope.fail(cause)).ok()
    }
}

impl AggregateGroupStateView for SchemeGroupsRead<'_> {
    fn group(&self, identity: &AggregateIdentity) -> Option<AggregateGroupHeader> {
        self.header(identity).cloned()
    }

    fn membership(&self, name: &PersistentNameIdentity) -> Option<AggregateGroupMembership> {
        let owner = *self.owner_row(name)?;
        let key = GroupMemberKey {
            group: owner.group,
            role: owner.role,
            name: *name,
        };
        let carrier = self
            .groups
            .members
            .get_in(&self.groups.key(key), self.scope)?;
        Some(AggregateGroupMembership {
            group: owner.group,
            member: AggregateGroupMember {
                carrier: carrier.clone(),
                role: owner.role,
            },
        })
    }

    fn is_retired(&self, identity: &AggregateIdentity) -> bool {
        SchemeGroupsRead::is_retired(self, identity)
    }

    fn least_member(
        &self,
        group: &AggregateIdentity,
        role: Option<AggregateMemberRole>,
        skip: &BTreeSet<PersistentNameIdentity>,
    ) -> Option<PersistentNameIdentity> {
        let single;
        let roles: &[AggregateMemberRole] = match role {
            Some(role) => {
                single = [role];
                &single
            }
            None => &ROLES,
        };
        let counts = self.header(group)?.counts;
        let mut least: Option<PersistentNameIdentity> = None;
        for role in roles.iter().filter(|role| counts.of(**role) > 0) {
            let mut found = None;
            let scanned = self.groups.scan_members(*group, Some(*role), |key, _| {
                if skip.contains(&key.name) {
                    return Ok(true);
                }
                found = Some(key.name);
                Ok(false)
            });
            self.latch(scanned)?;
            least = match (least, found) {
                (Some(least), Some(found)) => Some(least.min(found)),
                (least, found) => least.or(found),
            };
        }
        least
    }

    fn members(&self, group: &AggregateIdentity) -> AggregateGroupMembers {
        let mut members = AggregateGroupMembers::new();
        let scanned = self.groups.scan_members(*group, None, |key, carrier| {
            members
                .insert(key.name, carrier.clone(), key.role)
                .map_err(|_| Cause::Damaged)?;
            Ok(true)
        });
        self.latch(scanned);
        members
    }
}
