//! Canonical encoding of one commit's group lineage frame.
//!
//! The frame is the canonical commit envelope's group lineage: one
//! [`CommittedGroupTransition`] per touched installed group scheme, in
//! ascending scheme order. Every collection is written in its one canonical
//! ascending order, so equal transitions always encode to equal bytes and the
//! decoder refuses any other ordering.
//!
//! Version 2 records a kept group by its member delta, so a `k`-member edit
//! of an `M`-member group encodes `O(k)` bytes; a born or revived group
//! carries its initial members. Version 1 frames (complete kept membership)
//! are refused typed.
//!
//! ```text
//! frame      := "WPGL" version:u8 count:u32 transition*
//! transition := kind:u8 scheme:text rule:u8
//!               count:u32 assignment*  count:u32 retired:digest*
//!               count:u32 event*
//! assignment := identity:digest anchor:digest count:u32 source:digest*
//!               origin:u8 body
//! body       := kept:    count:u32 removed:digest* count:u32 member*
//!             | born, revived: count:u32 member*
//! member     := name:digest carrier-tag:u8 entity role:u8
//! ```

use crate::contracts::aggregate_identity::transition::group::delta::{
    AggregateGroupAssignment, AggregateGroupChange,
};
use crate::contracts::aggregate_identity::transition::group::model::{
    AggregateGroupMember, AggregateGroupOrigin, AggregateMemberRole,
};
use crate::contracts::graph_member::NativeMemberKey;
use crate::contracts::lineage::data::naming::PersistentNameIdentity;
use crate::lineage::groups::CommittedGroupTransition;

use super::format::group_event_parts;
use super::refusal::PersistentNameDurabilityError as Refusal;
use super::wire::WireWriter;

pub(super) const GROUP_FRAME_MAGIC: &[u8; 4] = b"WPGL";
pub(super) const GROUP_FRAME_VERSION: u8 = 2;

pub(super) const fn origin_tag(origin: AggregateGroupOrigin) -> u8 {
    match origin {
        AggregateGroupOrigin::Kept => 1,
        AggregateGroupOrigin::Born => 2,
        AggregateGroupOrigin::Revived => 3,
    }
}

pub(super) const fn role_tag(role: AggregateMemberRole) -> u8 {
    match role {
        AggregateMemberRole::Ordinary => 1,
        AggregateMemberRole::Anchor => 2,
        AggregateMemberRole::Boundary => 3,
    }
}

/// Encodes one commit's group transitions; no transitions encode to no bytes,
/// so a commit without group lineage keeps its envelope bytes unchanged.
///
/// Complexity: O(groups, sources, events, kept deltas and born or revived
/// members): never a kept group's untouched members.
pub fn encode_group_lineage_frame(
    transitions: &[CommittedGroupTransition],
) -> Result<Vec<u8>, Refusal> {
    if transitions.is_empty() {
        return Ok(Vec::new());
    }
    let ascending = transitions
        .windows(2)
        .all(|pair| pair[0].scheme_id() < pair[1].scheme_id());
    if !ascending {
        return Err(Refusal::NonCanonical {
            field: "group transition scheme order",
        });
    }
    let mut writer = WireWriter::default();
    writer.raw(GROUP_FRAME_MAGIC);
    writer.u8(GROUP_FRAME_VERSION);
    writer.count("group transitions", transitions.len())?;
    for committed in transitions {
        encode_transition(&mut writer, committed)?;
    }
    Ok(writer.into_bytes())
}

fn encode_transition(
    writer: &mut WireWriter,
    committed: &CommittedGroupTransition,
) -> Result<(), Refusal> {
    writer.u8(committed.kind.tag());
    writer.text("group scheme id", committed.scheme_id())?;
    writer.u8(committed.scheme.rule().canonical_tag());
    let transition = &committed.transition;
    let ascending = transition
        .assignments
        .windows(2)
        .all(|pair| pair[0].identity < pair[1].identity);
    if !ascending {
        return Err(Refusal::NonCanonical {
            field: "group assignment order",
        });
    }
    writer.count("group assignments", transition.assignments.len())?;
    for assignment in &transition.assignments {
        encode_assignment(writer, assignment)?;
    }
    writer.count("retired groups", transition.retired.len())?;
    for group in &transition.retired {
        writer.digest(group.bytes());
    }
    writer.count("group lineage events", transition.lineage.len())?;
    for event in &transition.lineage {
        let (tag, first, second) = group_event_parts(*event);
        writer.u8(tag);
        writer.digest(first.bytes());
        if let Some(second) = second {
            writer.digest(second.bytes());
        }
    }
    Ok(())
}

fn encode_assignment(
    writer: &mut WireWriter,
    assignment: &AggregateGroupAssignment,
) -> Result<(), Refusal> {
    writer.digest(assignment.identity.bytes());
    writer.digest(assignment.anchor.as_bytes());
    let canonical_sources = assignment.sources.windows(2).all(|pair| pair[0] < pair[1]);
    if !canonical_sources {
        return Err(Refusal::NonCanonical {
            field: "group sources",
        });
    }
    writer.count("group sources", assignment.sources.len())?;
    for source in &assignment.sources {
        writer.digest(source.bytes());
    }
    writer.u8(origin_tag(assignment.origin()));
    match &assignment.change {
        AggregateGroupChange::Kept(delta) => {
            writer.count("removed group members", delta.removed_len())?;
            for name in delta.removed() {
                writer.digest(name.as_bytes());
            }
            writer.count("upserted group members", delta.upserted_len())?;
            for (name, member) in delta.upserted() {
                encode_member(writer, name, member)?;
            }
        }
        AggregateGroupChange::Born(members) | AggregateGroupChange::Revived(members) => {
            writer.count("group members", members.len())?;
            for (name, member) in members.iter() {
                encode_member(writer, name, member)?;
            }
        }
    }
    Ok(())
}

fn encode_member(
    writer: &mut WireWriter,
    name: &PersistentNameIdentity,
    member: &AggregateGroupMember,
) -> Result<(), Refusal> {
    writer.digest(name.as_bytes());
    let NativeMemberKey::Entity(entity) = &member.carrier else {
        return Err(Refusal::NonCanonical {
            field: "group member carrier",
        });
    };
    writer.u8(1);
    encode_entity(writer, *entity);
    writer.u8(role_tag(member.role));
    Ok(())
}

fn encode_entity(
    writer: &mut WireWriter,
    entity: crate::contracts::identity::data::record_ids::EntityId,
) {
    writer.u32(entity.partition_value());
    writer.u64(entity.local_slot_value());
    writer.u32(entity.generation_value());
}
