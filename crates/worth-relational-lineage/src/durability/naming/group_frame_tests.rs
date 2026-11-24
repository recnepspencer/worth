//! The canonical group lineage frame round-trips exactly and refuses every
//! non-canonical, truncated or tampered encoding typed.
use std::collections::BTreeSet;

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent as Event;
use crate::contracts::aggregate_identity::transition::group::model::{
    AggregateContinuityRule, AggregateGroupAssignment, AggregateGroupMembers, AggregateGroupOrigin,
    AggregateGroupScheme, AggregateGroupSchemeId, AggregateMemberRole,
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
                    AggregateMemberRole::Ordinary
                };
                members.insert(name, carrier, role).unwrap();
            }
            AggregateGroupAssignment {
                identity: group(*byte),
                anchor: PersistentNameIdentity::from_minted_digest([*byte; 32]),
                sources: vec![group(200), group(201)],
                members,
                origin: AggregateGroupOrigin::Born,
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
