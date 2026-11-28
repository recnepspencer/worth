//! The naming-page rows of a scheme's live groups.
//!
//! A live group is one `GROUP_LIVE` header (its name parts and member counts
//! by role) plus one `GROUP_MEMBERS` row per member, keyed by group, role and
//! name, so a group's least member holding a role is one ordered seek and a
//! `k`-member edit writes `O(k)` rows. `GROUP_MEMBER_OWNER` maps each member
//! name to its group and role. No row repeats a group's membership.
use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::transition::group::model::{
    AggregateMemberRole as Role, CanonicalGroupMemberIdentity as MemberName,
};
use crate::contracts::aggregate_identity::transition::group::state_view::{
    AggregateGroupHeader, AggregateGroupRoleCounts,
};
use crate::contracts::graph_member::NativeMemberKey;
use crate::contracts::lineage::data::naming::{
    PersistentNameIdentity as Name, PersistentNameIdentityOwnerPort as _,
};
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use crate::lineage::naming_pages::codec::{read_serde, write_serde, NamingCodec, Reader, Writer};

/// The `GROUP_LIVE` header row's format: name parts and role counts only.
const GROUP_HEADER_FORMAT: u8 = 3;

/// One member of one live group, ordered by group, then role, then name.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct GroupMemberKey {
    pub(crate) group: AggregateIdentity,
    pub(crate) role: Role,
    pub(crate) name: MemberName,
}

impl GroupMemberKey {
    /// The least key of `group`'s members holding `role`.
    pub(crate) fn least(group: AggregateIdentity, role: Role) -> Self {
        Self {
            group,
            role,
            name: MemberName::Entity(Name::from_minted_digest([0; 32])),
        }
    }
}

/// A member name's live group and its role there.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GroupOwnerRow {
    pub(crate) group: AggregateIdentity,
    pub(crate) role: Role,
}

pub(crate) const ROLES: [Role; 3] = [Role::Interior, Role::Anchor, Role::Boundary];

const fn role_tag(role: Role) -> u8 {
    match role {
        Role::Interior => 0,
        Role::Anchor => 1,
        Role::Boundary => 2,
    }
}

fn read_role(input: &mut Reader<'_>) -> Result<Role, Cause> {
    match input.u8()? {
        0 => Ok(Role::Interior),
        1 => Ok(Role::Anchor),
        2 => Ok(Role::Boundary),
        _ => Err(Cause::Damaged),
    }
}

impl NamingCodec for MemberName {
    fn write(&self, out: &mut Writer) {
        match self {
            Self::Entity(name) => {
                out.u8(0);
                name.write(out);
            }
            Self::ProjectedCase {
                carrier,
                domain,
                case,
            } => {
                out.u8(1);
                carrier.write(out);
                write_ordered_text(&domain.schema.0, out);
                out.u32(domain.version.0);
                write_ordered_text(&domain.name, out);
                write_ordered_text(case, out);
            }
        }
    }

    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        use crate::contracts::projected_member_domain::ProjectedMemberDomainIdentity;
        use crate::contracts::schema::data::{SchemaId, SchemaVersionId};

        match input.u8()? {
            0 => Ok(Self::Entity(Name::read(input)?)),
            1 => Ok(Self::ProjectedCase {
                carrier: Name::read(input)?,
                domain: ProjectedMemberDomainIdentity {
                    schema: SchemaId(read_ordered_text(input)?),
                    version: SchemaVersionId(input.u32()?),
                    name: read_ordered_text(input)?,
                },
                case: read_ordered_text(input)?,
            }),
            _ => Err(Cause::Damaged),
        }
    }
}

// Escaped NUL sorts before every non-NUL byte, and the terminator sorts
// before any continuation. This preserves Rust's string order in page seeks.
fn write_ordered_text(text: &str, out: &mut Writer) {
    for byte in text.bytes() {
        out.u8(byte);
        if byte == 0 {
            out.u8(1);
        }
    }
    out.u8(0);
    out.u8(0);
}

fn read_ordered_text(input: &mut Reader<'_>) -> Result<String, Cause> {
    let mut text = Vec::new();
    loop {
        let byte = input.u8()?;
        if byte == 0 {
            match input.u8()? {
                0 => return String::from_utf8(text).map_err(|_| Cause::Damaged),
                1 => {}
                _ => return Err(Cause::Damaged),
            }
        }
        text.push(byte);
    }
}

impl NamingCodec for GroupMemberKey {
    fn write(&self, out: &mut Writer) {
        self.group.write(out);
        out.u8(role_tag(self.role));
        self.name.write(out);
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        Ok(Self {
            group: NamingCodec::read(input)?,
            role: read_role(input)?,
            name: NamingCodec::read(input)?,
        })
    }
}

impl NamingCodec for GroupOwnerRow {
    fn write(&self, out: &mut Writer) {
        self.group.write(out);
        out.u8(role_tag(self.role));
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        Ok(Self {
            group: NamingCodec::read(input)?,
            role: read_role(input)?,
        })
    }
}

impl NamingCodec for NativeMemberKey {
    fn write(&self, out: &mut Writer) {
        write_serde(self, out);
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        read_serde(input)
    }
}

impl NamingCodec for AggregateGroupHeader {
    fn write(&self, out: &mut Writer) {
        out.u8(GROUP_HEADER_FORMAT);
        self.identity.write(out);
        self.anchor.write(out);
        self.sources.write(out);
        for count in self.counts.slots() {
            out.u64(count);
        }
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        if input.u8()? != GROUP_HEADER_FORMAT {
            return Err(Cause::Damaged);
        }
        let identity = NamingCodec::read(input)?;
        let anchor = NamingCodec::read(input)?;
        let sources: Vec<AggregateIdentity> = NamingCodec::read(input)?;
        if !sources.windows(2).all(|pair| pair[0] < pair[1]) {
            return Err(Cause::Damaged);
        }
        let counts = [input.u64()?, input.u64()?, input.u64()?];
        let counts = AggregateGroupRoleCounts::from_slots(counts);
        if counts.total() == 0 {
            return Err(Cause::Damaged);
        }
        Ok(Self {
            identity,
            anchor,
            sources,
            counts,
        })
    }
}

impl NamingCodec
    for crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent
{
    fn write(&self, out: &mut Writer) {
        use crate::durability::naming::format::group_event_parts;
        let (tag, first, second) = group_event_parts(*self);
        out.u8(tag);
        first.write(out);
        if let Some(second) = second {
            second.write(out);
        }
    }

    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        use crate::durability::naming::format::{group_event_arity, group_event_from_parts};
        let tag = input.u8()?;
        let arity = group_event_arity(tag).map_err(|_| Cause::Damaged)?;
        let first = AggregateIdentity::read(input)?;
        let second = (arity == 2)
            .then(|| AggregateIdentity::read(input))
            .transpose()?;
        group_event_from_parts(tag, first, second).map_err(|_| Cause::Damaged)
    }
}
