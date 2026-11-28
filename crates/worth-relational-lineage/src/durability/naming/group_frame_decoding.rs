//! Strict decoding of one commit's canonical group lineage frame.
//!
//! Every collection must appear in its one canonical ascending order and the
//! frame must be consumed exactly. Member names are admitted as raw minted
//! digests only for recovery to re-verify: the group naming authority re-runs
//! each partition against the reconstructed prior groups, and the branch group
//! state refuses any transition its own posture contradicts.

use std::collections::BTreeSet;

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent;
use crate::contracts::aggregate_identity::transition::group::delta::{
    AggregateGroupAssignment, AggregateGroupChange, AggregateGroupMemberDelta,
};
use crate::contracts::aggregate_identity::transition::group::model::{
    AggregateContinuityRule, AggregateGroupMembers, AggregateGroupScheme, AggregateGroupSchemeId,
    AggregateMemberRole, CanonicalGroupMemberIdentity,
};
use crate::contracts::aggregate_identity::transition::group::transition_result::AggregateGroupTransition;
use crate::contracts::graph_member::key::{NativeMemberKey, ProjectedMemberCoordinate};
use crate::contracts::identity::data::{primitives::PartitionId, record_ids::EntityId};
use crate::lineage::groups::{CommittedGroupTransition, GroupTransitionKind};
use crate::lineage::naming_pages::codec::NamingCodec;
use std::sync::Arc;

use super::format::{group_event_arity, group_event_from_parts};
use super::group_frame_encoding::{GROUP_FRAME_MAGIC, GROUP_FRAME_VERSION};
use super::refusal::{PersistentNameDurabilityError as Refusal, PersistentNameWireSection};
use super::wire::{read_strictly_ascending, WireReader};

/// Decodes one commit's group lineage frame; no bytes decode to no
/// transitions.
///
/// Complexity: O(frame bytes).
pub fn decode_group_lineage_frame(bytes: &[u8]) -> Result<Vec<CommittedGroupTransition>, Refusal> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let mut reader = WireReader::new(bytes, PersistentNameWireSection::GroupLineage);
    if reader.take(GROUP_FRAME_MAGIC.len())? != GROUP_FRAME_MAGIC {
        return Err(Refusal::UnrecognizedMagic {
            section: PersistentNameWireSection::GroupLineage,
        });
    }
    let version = reader.u8()?;
    if version != GROUP_FRAME_VERSION {
        return Err(Refusal::UnsupportedFormatVersion {
            found: version,
            supported: GROUP_FRAME_VERSION,
        });
    }
    let transitions = read_strictly_ascending(
        &mut reader,
        "group transition scheme order",
        decode_transition,
        |committed| committed.scheme_id().to_owned(),
    )?;
    if transitions.is_empty() {
        return Err(Refusal::NonCanonical {
            field: "empty group lineage frame",
        });
    }
    reader.finish()?;
    Ok(transitions)
}

fn decode_transition(reader: &mut WireReader<'_>) -> Result<CommittedGroupTransition, Refusal> {
    let kind_tag = reader.u8()?;
    let kind = GroupTransitionKind::from_tag(kind_tag).ok_or(Refusal::InvalidTag {
        field: "group transition kind",
        tag: kind_tag,
    })?;
    let scheme_id = reader.text("group scheme id")?;
    let scheme_id = AggregateGroupSchemeId::new(scheme_id).map_err(|_| Refusal::NonCanonical {
        field: "group scheme id",
    })?;
    let rule = rule_from_tag(reader.u8()?)?;
    let assignments = read_strictly_ascending(
        reader,
        "group assignment order",
        decode_assignment,
        |assignment| assignment.identity,
    )?;
    let retired = read_strictly_ascending(reader, "retired groups", read_group, |group| *group)?;
    let lineage =
        read_strictly_ascending(reader, "group lineage events", decode_group_event, |e| *e)?;
    Ok(CommittedGroupTransition {
        kind,
        scheme: AggregateGroupScheme::new(scheme_id, rule),
        transition: AggregateGroupTransition {
            assignments,
            retired: retired.into_iter().collect::<BTreeSet<_>>(),
            lineage: lineage.into_iter().collect(),
        },
    })
}

fn read_group(reader: &mut WireReader<'_>) -> Result<AggregateIdentity, Refusal> {
    Ok(AggregateIdentity::from_bytes(reader.digest()?))
}

fn decode_assignment(reader: &mut WireReader<'_>) -> Result<AggregateGroupAssignment, Refusal> {
    let identity = read_group(reader)?;
    let anchor = read_name(reader)?;
    let sources = read_strictly_ascending(reader, "group sources", read_group, |source| *source)?;
    let non_canonical = |field| Refusal::NonCanonical { field };
    let change = match reader.u8()? {
        1 => {
            let removed =
                read_strictly_ascending(reader, "removed group members", read_name, |n| n.clone())?;
            let upserted =
                read_strictly_ascending(reader, "upserted group members", decode_member, |m| {
                    m.0.clone()
                })?;
            let mut delta = AggregateGroupMemberDelta::new();
            for name in removed {
                delta
                    .remove(name)
                    .map_err(|_| non_canonical("group member delta"))?;
            }
            for (name, carrier, role) in upserted {
                delta
                    .upsert(name, carrier, role)
                    .map_err(|_| non_canonical("group member delta"))?;
            }
            AggregateGroupChange::Kept(delta)
        }
        tag @ (2 | 3) => {
            let members =
                read_strictly_ascending(reader, "group members", decode_member, |m| m.0.clone())?;
            let mut set = AggregateGroupMembers::new();
            for (name, carrier, role) in members {
                set.insert(name, carrier, role)
                    .map_err(|_| non_canonical("group members"))?;
            }
            if tag == 2 {
                AggregateGroupChange::Born(set)
            } else {
                AggregateGroupChange::Revived(set)
            }
        }
        tag => {
            return Err(Refusal::InvalidTag {
                field: "group origin",
                tag,
            })
        }
    };
    Ok(AggregateGroupAssignment {
        identity,
        anchor,
        sources,
        change,
    })
}

fn read_name(reader: &mut WireReader<'_>) -> Result<CanonicalGroupMemberIdentity, Refusal> {
    let length = reader.count()?;
    CanonicalGroupMemberIdentity::from_bytes(reader.take(length)?).map_err(|_| {
        Refusal::NonCanonical {
            field: "group member identity",
        }
    })
}

type DecodedMember = (
    CanonicalGroupMemberIdentity,
    NativeMemberKey,
    AggregateMemberRole,
);

fn decode_member(reader: &mut WireReader<'_>) -> Result<DecodedMember, Refusal> {
    let name = read_name(reader)?;
    let entity = decode_entity(reader)?;
    let carrier = match &name {
        CanonicalGroupMemberIdentity::Entity(_) => NativeMemberKey::Entity(entity),
        CanonicalGroupMemberIdentity::ProjectedCase { domain, case, .. } => {
            NativeMemberKey::Projected(
                Arc::new(ProjectedMemberCoordinate {
                    domain: domain.clone(),
                    carrier: entity,
                    case: case.clone(),
                })
                .into(),
            )
        }
    };
    let role = match reader.u8()? {
        1 => AggregateMemberRole::Interior,
        2 => AggregateMemberRole::Anchor,
        3 => AggregateMemberRole::Boundary,
        tag => {
            return Err(Refusal::InvalidTag {
                field: "group member role",
                tag,
            })
        }
    };
    Ok((name, carrier, role))
}

fn rule_from_tag(tag: u8) -> Result<AggregateContinuityRule, Refusal> {
    match tag {
        1 => Ok(AggregateContinuityRule::Anchor),
        2 => Ok(AggregateContinuityRule::Boundary),
        3 => Ok(AggregateContinuityRule::ProducerRole),
        tag => Err(Refusal::InvalidTag {
            field: "group continuity rule",
            tag,
        }),
    }
}

fn decode_entity(reader: &mut WireReader<'_>) -> Result<EntityId, Refusal> {
    let (partition, slot, generation) = (reader.u32()?, reader.u64()?, reader.u32()?);
    Ok(EntityId::new(PartitionId(partition), slot, generation))
}

fn decode_group_event(reader: &mut WireReader<'_>) -> Result<AggregateGroupLineageEvent, Refusal> {
    let tag = reader.u8()?;
    let arity = group_event_arity(tag)?;
    let first = AggregateIdentity::from_bytes(reader.digest()?);
    let second = if arity == 2 {
        Some(AggregateIdentity::from_bytes(reader.digest()?))
    } else {
        None
    };
    group_event_from_parts(tag, first, second)
}
