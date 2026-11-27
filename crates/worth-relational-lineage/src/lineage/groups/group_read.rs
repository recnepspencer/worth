//! Read views of a branch's group state inside one read boundary's scope,
//! and the naming-page codec of a live group.
use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::transition::group::model::{
    AggregateContinuityRule, AggregateGroupMembers, AggregateMemberRole, AggregatePriorGroup,
};
use crate::contracts::lineage::data::naming::PersistentNameIdentity;
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use crate::lineage::naming_pages::codec::{read_serde, write_serde, NamingCodec, Reader, Writer};
use crate::lineage::naming_pages::tables::scoped_prefix;
use crate::lineage::tiered::NamingReadScope;

use super::branch_state::{BranchGroupState, GroupStateDenial, SchemeGroups};
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

    /// The live group `identity`.
    ///
    /// Complexity: O(log G) and at most one page fault.
    pub fn group(&self, identity: &AggregateIdentity) -> Option<&'s AggregatePriorGroup> {
        self.groups
            .live
            .get_in(&self.groups.key(*identity), self.scope)
    }

    /// The live group `member` belongs to.
    ///
    /// Complexity: O(log M) and at most one page fault.
    pub fn owner_of(&self, member: &PersistentNameIdentity) -> Option<AggregateIdentity> {
        self.groups
            .member_owner
            .get_in(&self.groups.key(*member), self.scope)
            .copied()
    }

    /// Whether `identity` is a retired, unrevived group name.
    pub fn is_retired(&self, identity: &AggregateIdentity) -> bool {
        self.groups
            .retired
            .get_in(&self.groups.key(*identity), self.scope)
            .is_some()
    }

    /// Every live group in identity order: a scan of this scheme's range.
    ///
    /// Complexity: O(live groups of the scheme).
    pub fn live(&self) -> Vec<&'s AggregatePriorGroup> {
        let mut live = Vec::new();
        let prefix = scoped_prefix(self.groups.live.table(), &self.groups.id);
        let scanned = self.groups.live.scan_merged(&prefix, |_, group| {
            live.push(self.scope.alloc(group.into_owned()));
            Ok(true)
        });
        if let Err(cause) = scanned {
            self.scope.fail(cause);
        }
        live
    }
}

fn role_tag(role: AggregateMemberRole) -> u8 {
    match role {
        AggregateMemberRole::Ordinary => 0,
        AggregateMemberRole::Anchor => 1,
        AggregateMemberRole::Boundary => 2,
    }
}

impl NamingCodec for AggregatePriorGroup {
    fn write(&self, out: &mut Writer) {
        self.identity.write(out);
        self.anchor.write(out);
        self.sources.write(out);
        out.u32(u32::try_from(self.members.len()).unwrap_or(u32::MAX));
        for (name, member) in self.members.iter() {
            name.write(out);
            write_serde(&member.carrier, out);
            out.u8(role_tag(member.role));
        }
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        let identity = NamingCodec::read(input)?;
        let anchor = NamingCodec::read(input)?;
        let sources = NamingCodec::read(input)?;
        let mut members = AggregateGroupMembers::new();
        for _ in 0..input.count()? {
            let name = PersistentNameIdentity::read(input)?;
            let carrier = read_serde(input)?;
            let role = match input.u8()? {
                0 => AggregateMemberRole::Ordinary,
                1 => AggregateMemberRole::Anchor,
                2 => AggregateMemberRole::Boundary,
                _ => return Err(Cause::Damaged),
            };
            if members.iter().last().is_some_and(|(last, _)| *last >= name) {
                return Err(Cause::Damaged);
            }
            members
                .insert(name, carrier, role)
                .map_err(|_| Cause::Damaged)?;
        }
        Ok(Self {
            identity,
            anchor,
            sources,
            members,
        })
    }
}
