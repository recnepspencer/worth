//! The real group state's kept-delta transitions as a conformance trace for
//! the proved Lean model (`proofs/lean/canonical_group_delta`).
//!
//! One group is created, attached to, detached from (one and two members),
//! re-carried, re-roled, retired and revived, then attached to again. Every
//! step's transition is encoded and decoded through the WPGL v2 frame and
//! applied by `BranchGroupState::apply` (verification and installation); a
//! second state replays every decoded frame from empty, as `GROUP_LOG`
//! replay does. Each step records the group's prior membership, the
//! recorded delta, the installed and replayed memberships and the delta's
//! wire bytes. With `WORTH_GROUP_DELTA_TRACE=<path>` the steps are written as
//! NDJSON, which `lake exe conformance <path>` checks against the proved
//! `applyDelta`, `diff` and `encode`.
use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent as Event;
use crate::contracts::aggregate_identity::transition::group::delta::{
    AggregateGroupAssignment, AggregateGroupChange, AggregateGroupMemberDelta,
};
use crate::contracts::aggregate_identity::transition::group::model::{
    AggregateGroupMember, AggregateGroupMembers, AggregateMemberRole as Role,
};
use crate::contracts::aggregate_identity::transition::group::state_view::AggregateGroupStateView;
use crate::contracts::graph_member::NativeMemberKey;
use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::lineage::data::naming::{
    PersistentNameIdentity as Name, PersistentNameIdentityOwnerPort as _,
};
use crate::durability::naming::{decode_group_lineage_frame, encode_group_lineage_frame};
use crate::lineage::tiered::NamingReadScope;

use super::kept_delta_installation::{carrier, committed, group, name};
use super::{BranchGroupState, CommittedGroupTransition};

const GROUP: u8 = 10;

/// One step of the trace.
enum Step {
    Born(Vec<(u8, u32, Role)>),
    Kept(Vec<u8>, Vec<(u8, u32, Role)>),
    Retired,
    Revived(Vec<(u8, u32, Role)>),
}

fn members(entries: &[(u8, u32, Role)]) -> AggregateGroupMembers {
    let mut set = AggregateGroupMembers::new();
    for &(n, generation, role) in entries {
        set.insert(name(n), carrier(n, generation), role).unwrap();
    }
    set
}

fn transition(step: &Step) -> CommittedGroupTransition {
    let assign = |change| AggregateGroupAssignment {
        identity: group(GROUP),
        anchor: name(1),
        sources: Vec::new(),
        change,
    };
    match step {
        Step::Born(entries) => committed(
            vec![assign(AggregateGroupChange::Born(members(entries)))],
            BTreeSet::from([Event::Born {
                group: group(GROUP),
            }]),
        ),
        Step::Revived(entries) => committed(
            vec![assign(AggregateGroupChange::Revived(members(entries)))],
            BTreeSet::from([Event::Revived {
                group: group(GROUP),
            }]),
        ),
        Step::Kept(removed, upserted) => {
            let mut delta = AggregateGroupMemberDelta::new();
            for n in removed {
                delta.remove(name(*n)).unwrap();
            }
            for &(n, generation, role) in upserted {
                delta.upsert(name(n), carrier(n, generation), role).unwrap();
            }
            committed(
                vec![assign(AggregateGroupChange::Kept(delta))],
                BTreeSet::new(),
            )
        }
        Step::Retired => {
            let mut retired = committed(
                Vec::new(),
                BTreeSet::from([Event::Retired {
                    group: group(GROUP),
                }]),
            );
            retired.transition.retired.insert(group(GROUP));
            retired
        }
    }
}

/// The step's frame with its membership change emptied: the difference in
/// bytes is the change's own wire size.
fn emptied(step: &Step) -> CommittedGroupTransition {
    match step {
        Step::Born(_) => transition(&Step::Born(Vec::new())),
        Step::Revived(_) => transition(&Step::Revived(Vec::new())),
        Step::Kept(..) => transition(&Step::Kept(Vec::new(), Vec::new())),
        Step::Retired => transition(&Step::Retired),
    }
}

fn group_members(state: &BranchGroupState) -> AggregateGroupMembers {
    let scope = NamingReadScope::new();
    let read = state.read(&scope);
    let members = read
        .scheme("t.loop")
        .map(|groups| AggregateGroupStateView::members(&groups, &group(GROUP)))
        .unwrap_or_default();
    assert!(scope.failure().is_none());
    members
}

fn json_members<'m>(entries: impl Iterator<Item = (&'m Name, &'m AggregateGroupMember)>) -> String {
    let mut out = String::from("[");
    for (at, (n, member)) in entries.enumerate() {
        let NativeMemberKey::Entity(entity) = &member.carrier else {
            panic!("entity carriers only");
        };
        let carried = entity.local_slot_value() * 1_000 + u64::from(entity.generation_value());
        let role = member.role as u8;
        let sep = if at == 0 { "" } else { "," };
        let _ = write!(out, "{sep}[{},{carried},{role}]", n.as_bytes()[0]);
    }
    out.push(']');
    out
}

fn steps() -> Vec<Step> {
    use Role::{Anchor, Boundary, Ordinary};
    let o = |n| (n, 1, Ordinary);
    vec![
        Step::Born(vec![(1, 1, Anchor), o(2), o(3), o(4), o(5)]),
        Step::Kept(Vec::new(), vec![o(6)]),
        Step::Kept(vec![3], Vec::new()),
        Step::Kept(Vec::new(), vec![(2, 2, Ordinary)]),
        Step::Kept(Vec::new(), vec![(4, 1, Boundary)]),
        Step::Kept(vec![5, 6], vec![o(7)]),
        Step::Retired,
        Step::Revived(vec![(1, 1, Anchor), (2, 2, Ordinary), o(8)]),
        Step::Kept(vec![8], vec![o(9), (2, 3, Boundary)]),
    ]
}

#[test]
fn kept_delta_transitions_replay_and_trace_against_the_proved_model() {
    let mut warm = BranchGroupState::default();
    let mut frames = Vec::new();
    let mut trace = String::new();
    for (at, step) in steps().iter().enumerate() {
        let commit = CommitId(at as u64 + 1);
        let prior = group_members(&warm);
        let committed = transition(step);
        let frame = encode_group_lineage_frame(std::slice::from_ref(&committed)).unwrap();
        let decoded = decode_group_lineage_frame(&frame).unwrap();
        assert_eq!(decoded, vec![committed.clone()], "WPGL v2 round-trips");
        warm.apply(commit, &decoded).unwrap();
        frames.push((commit, frame.clone()));
        let mut replayed = BranchGroupState::default();
        for (at, bytes) in &frames {
            replayed
                .apply(*at, &decode_group_lineage_frame(bytes).unwrap())
                .unwrap();
        }
        let (post, again) = (group_members(&warm), group_members(&replayed));
        assert_eq!(post, again, "GROUP_LOG replay reaches the warm state");
        let base = encode_group_lineage_frame(&[emptied(step)]).unwrap().len();
        let (kind, removed, upserted) = match step {
            Step::Born(entries) => ("born", Vec::new(), members(entries)),
            Step::Revived(entries) => ("revived", Vec::new(), members(entries)),
            Step::Kept(removed, entries) => ("kept", removed.clone(), members(entries)),
            Step::Retired => ("retired", Vec::new(), AggregateGroupMembers::new()),
        };
        let _ = writeln!(
            trace,
            r#"{{"step":{at},"kind":"{kind}","prior":{},"removed":{removed:?},"upserted":{},"post":{},"replayed":{},"wire_delta_bytes":{}}}"#,
            json_members(prior.iter()),
            json_members(upserted.iter()),
            json_members(post.iter()),
            json_members(again.iter()),
            frame.len() - base,
        );
    }
    if let Ok(path) = std::env::var("WORTH_GROUP_DELTA_TRACE") {
        std::fs::write(path, trace).unwrap();
    }
}
