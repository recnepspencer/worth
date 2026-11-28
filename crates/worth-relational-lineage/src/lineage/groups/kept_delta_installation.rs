//! A branch's group state installs kept-group member deltas exactly and
//! refuses every delta its posture contradicts: removing a stranger,
//! repeating an unchanged member, emptying a kept group, or pulling in a
//! member its group does not release.
use std::collections::BTreeSet;

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent as Event;
use crate::contracts::aggregate_identity::transition::group::delta::{
    AggregateGroupAssignment, AggregateGroupChange, AggregateGroupMemberDelta,
};
use crate::contracts::aggregate_identity::transition::group::model::{
    AggregateContinuityRule, AggregateGroupMembers, AggregateGroupScheme, AggregateGroupSchemeId,
    AggregateMemberRole as Role, CanonicalGroupMemberIdentity,
};
use crate::contracts::aggregate_identity::transition::group::state_view::AggregateGroupStateView;
use crate::contracts::aggregate_identity::transition::group::transition_result::AggregateGroupTransition;
use crate::contracts::graph_member::NativeMemberKey;
use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::identity::data::primitives::PartitionId;
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::lineage::data::naming::{
    PersistentNameIdentity as Name, PersistentNameIdentityOwnerPort as _,
};
use crate::lineage::tiered::NamingReadScope;

use super::{BranchGroupState, CommittedGroupTransition, GroupStateDenial, GroupTransitionKind};

pub(super) fn name(n: u8) -> CanonicalGroupMemberIdentity {
    Name::from_minted_digest([n; 32]).into()
}

pub(super) fn group(n: u8) -> AggregateIdentity {
    AggregateIdentity::from_bytes([n; 32])
}

pub(super) fn carrier(n: u8, generation: u32) -> NativeMemberKey {
    NativeMemberKey::Entity(EntityId::new(PartitionId(1), u64::from(n), generation))
}

pub(super) fn committed(
    assignments: Vec<AggregateGroupAssignment>,
    lineage: BTreeSet<Event>,
) -> CommittedGroupTransition {
    CommittedGroupTransition {
        kind: GroupTransitionKind::Partition,
        scheme: AggregateGroupScheme::new(
            AggregateGroupSchemeId::new("t.loop").unwrap(),
            AggregateContinuityRule::Anchor,
        ),
        transition: AggregateGroupTransition {
            assignments,
            retired: BTreeSet::new(),
            lineage,
        },
    }
}

/// Group `g` born holding `members`, the first as its anchor.
fn born(g: u8, members: &[u8]) -> CommittedGroupTransition {
    let mut set = AggregateGroupMembers::new();
    for (at, n) in members.iter().enumerate() {
        let role = if at == 0 {
            Role::Anchor
        } else {
            Role::Ordinary
        };
        set.insert(name(*n), carrier(*n, 1), role).unwrap();
    }
    let assignment = AggregateGroupAssignment {
        identity: group(g),
        anchor: name(members[0]),
        sources: Vec::new(),
        change: AggregateGroupChange::Born(set),
    };
    committed(
        vec![assignment],
        BTreeSet::from([Event::Born { group: group(g) }]),
    )
}

fn kept(g: u8, anchor: u8, delta: AggregateGroupMemberDelta) -> AggregateGroupAssignment {
    AggregateGroupAssignment {
        identity: group(g),
        anchor: name(anchor),
        sources: Vec::new(),
        change: AggregateGroupChange::Kept(delta),
    }
}

fn state() -> BranchGroupState {
    let mut state = BranchGroupState::default();
    state
        .apply(CommitId(1), &[born(10, &[1, 2, 3, 4, 5])])
        .unwrap();
    state.apply(CommitId(2), &[born(20, &[6, 7])]).unwrap();
    state
}

#[test]
fn kept_deltas_install_exactly_the_touched_members() {
    let mut state = state();
    let mut delta = AggregateGroupMemberDelta::new();
    delta.remove(name(1)).unwrap();
    delta
        .upsert(name(3), carrier(3, 2), Role::Boundary)
        .unwrap();
    delta
        .upsert(name(9), carrier(9, 1), Role::Ordinary)
        .unwrap();
    let change = committed(vec![kept(10, 1, delta)], BTreeSet::new());
    state.apply(CommitId(3), &[change]).unwrap();
    let scope = NamingReadScope::new();
    let read = state.read(&scope);
    let groups = read.scheme("t.loop").unwrap();
    let members = AggregateGroupStateView::members(&groups, &group(10));
    let held: Vec<(u8, Role)> = members
        .iter()
        .map(|(n, m)| (n.carrier_name().as_bytes()[0], m.role))
        .collect();
    let expected = [2, 3, 4, 5, 9].map(|n| {
        (
            n,
            if n == 3 {
                Role::Boundary
            } else {
                Role::Ordinary
            },
        )
    });
    assert_eq!(held, expected);
    assert_eq!(members.get(&name(3)).unwrap().carrier, carrier(3, 2));
    let header = groups.header(&group(10)).unwrap();
    assert_eq!(header.counts.slots(), [4, 0, 1]);
    assert_eq!(groups.owner_of(&name(1)), None);
    assert_eq!(groups.owner_of(&name(9)), Some(group(10)));
    let skip = BTreeSet::from([name(2)]);
    assert_eq!(groups.least_member(&group(10), None, &skip), Some(name(3)));
    assert_eq!(
        groups.least_member(&group(10), Some(Role::Ordinary), &skip),
        Some(name(4))
    );
    assert_eq!(
        groups.least_member(&group(10), Some(Role::Anchor), &skip),
        None
    );
    assert!(scope.failure().is_none());
}

#[test]
fn non_canonical_kept_deltas_refuse_with_the_group_named() {
    let state = state();
    let posture = |result| matches!(result, Err(GroupStateDenial::GroupPosture { group: g, .. }) if g == group(10));
    let mut stranger = AggregateGroupMemberDelta::new();
    stranger.remove(name(6)).unwrap();
    assert!(posture(state.verify(&[committed(
        vec![kept(10, 1, stranger)],
        BTreeSet::new()
    )])));
    let mut unchanged = AggregateGroupMemberDelta::new();
    unchanged
        .upsert(name(2), carrier(2, 1), Role::Ordinary)
        .unwrap();
    assert!(posture(state.verify(&[committed(
        vec![kept(10, 1, unchanged)],
        BTreeSet::new()
    )])));
    let mut emptied = AggregateGroupMemberDelta::new();
    for n in 1..=5 {
        emptied.remove(name(n)).unwrap();
    }
    assert!(matches!(
        state.verify(&[committed(vec![kept(10, 1, emptied)], BTreeSet::new())]),
        Err(GroupStateDenial::EmptyGroup { .. })
    ));
}

#[test]
fn an_entering_member_must_leave_its_group_in_the_same_transition() {
    let state = state();
    let mut pull = AggregateGroupMemberDelta::new();
    pull.upsert(name(6), carrier(6, 1), Role::Ordinary).unwrap();
    let held = committed(vec![kept(10, 1, pull.clone())], BTreeSet::new());
    assert!(matches!(
        state.verify(&[held]),
        Err(GroupStateDenial::MemberOverlap { member, .. }) if member == name(6)
    ));
    let mut release = AggregateGroupMemberDelta::new();
    release.remove(name(6)).unwrap();
    let moved = committed(
        vec![kept(10, 1, pull), kept(20, 6, release)],
        BTreeSet::new(),
    );
    let mut state = state;
    state.apply(CommitId(3), &[moved]).unwrap();
    let scope = NamingReadScope::new();
    let groups = state.read(&scope).scheme("t.loop").unwrap();
    assert_eq!(groups.owner_of(&name(6)), Some(group(10)));
    assert_eq!(groups.header(&group(20)).unwrap().counts.total(), 1);
}
