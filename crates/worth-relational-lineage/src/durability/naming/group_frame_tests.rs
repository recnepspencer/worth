//! The canonical group lineage frame round-trips exactly and refuses every
//! non-canonical, truncated or tampered encoding typed.
use std::collections::BTreeSet;

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent as Event;
use crate::contracts::aggregate_identity::transition::group::delta::{
    AggregateGroupAssignment, AggregateGroupChange, AggregateGroupMemberDelta,
};
use crate::contracts::aggregate_identity::transition::group::model::{
    AggregateContinuityRule, AggregateGroupMembers, AggregateGroupScheme, AggregateGroupSchemeId,
    AggregateMemberRole,
};
use crate::contracts::aggregate_identity::transition::group::transition_result::AggregateGroupTransition;
use crate::contracts::graph_member::NativeMemberKey;
use crate::contracts::identity::data::primitives::PartitionId;
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::lineage::data::naming::{
    PersistentNameIdentity, PersistentNameIdentityOwnerPort as _,
};
use crate::lineage::groups::{CommittedGroupTransition, GroupTransitionKind};

use super::refusal::PersistentNameDurabilityError as Refusal;
use super::{decode_group_lineage_frame, encode_group_lineage_frame};

fn group(byte: u8) -> AggregateIdentity {
    AggregateIdentity::from_bytes([byte; 32])
}

fn transition(scheme: &str, groups: &[u8]) -> CommittedGroupTransition {
    let assignments = groups
        .iter()
        .map(|byte| {
            let mut members = AggregateGroupMembers::new();
            for offset in 0..2u8 {
                let name = PersistentNameIdentity::from_minted_digest([byte + offset; 32]);
                let carrier = NativeMemberKey::Entity(EntityId::new(
                    PartitionId(1),
                    u64::from(*byte + offset),
                    1,
                ));
                let role = if offset == 0 {
                    AggregateMemberRole::Anchor
                } else {
                    AggregateMemberRole::Interior
                };
                members.insert(name, carrier, role).unwrap();
            }
            AggregateGroupAssignment {
                identity: group(*byte),
                anchor: PersistentNameIdentity::from_minted_digest([*byte; 32]).into(),
                sources: vec![group(200), group(201)],
                change: AggregateGroupChange::Born(members),
            }
        })
        .collect();
    CommittedGroupTransition {
        kind: GroupTransitionKind::Partition,
        scheme: AggregateGroupScheme::new(
            AggregateGroupSchemeId::new(scheme).unwrap(),
            AggregateContinuityRule::Anchor,
        ),
        transition: AggregateGroupTransition {
            assignments,
            retired: BTreeSet::from([group(200)]),
            lineage: BTreeSet::from([
                Event::Born { group: group(10) },
                Event::Retired { group: group(200) },
                Event::SplitFrom {
                    fragment: group(10),
                    source: group(201),
                },
            ]),
        },
    }
}

#[test]
fn frames_round_trip_and_no_transition_is_no_bytes() {
    assert_eq!(encode_group_lineage_frame(&[]).unwrap(), Vec::<u8>::new());
    assert_eq!(decode_group_lineage_frame(&[]).unwrap(), Vec::new());
    let committed = vec![
        transition("a.loop.v1", &[10, 20]),
        transition("b.shell.v1", &[30]),
    ];
    let bytes = encode_group_lineage_frame(&committed).unwrap();
    assert_eq!(decode_group_lineage_frame(&bytes).unwrap(), committed);
    assert_eq!(
        encode_group_lineage_frame(&decode_group_lineage_frame(&bytes).unwrap()).unwrap(),
        bytes
    );
}

#[test]
fn non_canonical_frames_refuse_typed() {
    let reversed = vec![
        transition("b.shell.v1", &[30]),
        transition("a.loop.v1", &[10]),
    ];
    assert!(matches!(
        encode_group_lineage_frame(&reversed),
        Err(Refusal::NonCanonical { .. })
    ));
    let mut unsorted = transition("a.loop.v1", &[10, 20]);
    unsorted.transition.assignments.reverse();
    assert!(matches!(
        encode_group_lineage_frame(&[unsorted]),
        Err(Refusal::NonCanonical { .. })
    ));
    // A frame naming one scheme twice is not canonical either.
    let doubled = [
        transition("a.loop.v1", &[10]),
        transition("a.loop.v1", &[20]),
    ];
    assert!(matches!(
        encode_group_lineage_frame(&doubled),
        Err(Refusal::NonCanonical { .. })
    ));
}

#[test]
fn tampered_truncated_and_padded_frames_refuse_typed() {
    let bytes = encode_group_lineage_frame(&[transition("a.loop.v1", &[10])]).unwrap();
    let mut magic = bytes.clone();
    magic[0] ^= 0xff;
    assert!(matches!(
        decode_group_lineage_frame(&magic),
        Err(Refusal::UnrecognizedMagic { .. })
    ));
    let mut version = bytes.clone();
    version[4] = 9;
    assert!(matches!(
        decode_group_lineage_frame(&version),
        Err(Refusal::UnsupportedFormatVersion { found: 9, .. })
    ));
    let mut kind = bytes.clone();
    kind[9] = 7;
    assert!(matches!(
        decode_group_lineage_frame(&kind),
        Err(Refusal::InvalidTag { .. })
    ));
    for cut in [5, bytes.len() / 2, bytes.len() - 1] {
        assert!(
            matches!(
                decode_group_lineage_frame(&bytes[..cut]),
                Err(Refusal::Truncated { .. })
            ),
            "{cut}"
        );
    }
    let mut padded = bytes.clone();
    padded.push(0);
    assert!(matches!(
        decode_group_lineage_frame(&padded),
        Err(Refusal::TrailingBytes { .. })
    ));
}

fn name(byte: u8) -> PersistentNameIdentity {
    PersistentNameIdentity::from_minted_digest([byte; 32])
}

/// A transition keeping group 10 with `removed` leaving and member 99
/// joining.
fn kept(removed: &[u8]) -> CommittedGroupTransition {
    let mut delta = AggregateGroupMemberDelta::new();
    for byte in removed {
        delta.remove(name(*byte)).unwrap();
    }
    let carrier = NativeMemberKey::Entity(EntityId::new(PartitionId(1), 99, 1));
    delta
        .upsert(name(99), carrier, AggregateMemberRole::Interior)
        .unwrap();
    let mut committed = transition("a.loop.v1", &[10]);
    committed.transition.assignments[0].change = AggregateGroupChange::Kept(delta);
    committed.transition.lineage = BTreeSet::new();
    committed.transition.retired = BTreeSet::new();
    committed
}

#[test]
fn kept_groups_encode_their_member_delta_alone() {
    let committed = vec![kept(&[3, 7])];
    let bytes = encode_group_lineage_frame(&committed).unwrap();
    assert_eq!(bytes[4], 3, "WPGL v3 records kept groups by delta");
    assert_eq!(decode_group_lineage_frame(&bytes).unwrap(), committed);
    // One more removed member costs exactly one entity member key: the frame
    // never repeats a kept group's untouched members.
    let wider = encode_group_lineage_frame(&[kept(&[3, 7, 8])]).unwrap();
    assert_eq!(wider.len(), bytes.len() + 37);
}

#[test]
fn version_one_and_non_canonical_deltas_refuse_typed() {
    let bytes = encode_group_lineage_frame(&[kept(&[3, 7])]).unwrap();
    let mut legacy = bytes.clone();
    legacy[4] = 1;
    assert!(matches!(
        decode_group_lineage_frame(&legacy),
        Err(Refusal::UnsupportedFormatVersion {
            found: 1,
            supported: 3
        })
    ));
    // Swap the two removed names: a descending delta is not canonical.
    let first = bytes
        .windows(32)
        .position(|window| window == [3; 32])
        .unwrap();
    let mut swapped = bytes.clone();
    swapped[first..first + 32].copy_from_slice(&[7; 32]);
    swapped[first + 37..first + 69].copy_from_slice(&[3; 32]);
    assert!(matches!(
        decode_group_lineage_frame(&swapped),
        Err(Refusal::NonCanonical { .. })
    ));
}
