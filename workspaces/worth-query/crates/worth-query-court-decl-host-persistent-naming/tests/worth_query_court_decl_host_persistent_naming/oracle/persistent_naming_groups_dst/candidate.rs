//! Candidate envelopes (#361), transcribed from
//! `specs/tla/persistent_naming/PersistentNamingGroupsCandidate.tla`: an
//! earlier stage applies one group action (or a field edit) to the
//! envelope's unpublished candidate, and a later stage reads a member's loop
//! and resolves one loop. Query's rule predicts every read: a claimed scheme
//! refuses, an answer resting on name history refuses, and one resting on a
//! member the candidate ended refuses; otherwise the basis answer stands.
//! The envelope commits exactly when every read answered, and then each
//! answer must equal what the committed envelope made true.
use std::collections::BTreeSet;

use super::model::{Answer, Model, Op};

/// The earlier stage: a group action, or a field edit of member `m`.
#[derive(Clone, Debug)]
pub enum First {
    Op(Op),
    Edit(usize),
}

/// One later-stage read: a member's loop, or a loop resolved
/// (`all`: `AllDescendants`).
#[derive(Clone, Copy, Debug)]
pub enum Read {
    MemberOf(usize),
    Resolve(usize, bool),
}

pub const REFUSALS: [&str; 3] = ["RefusedClaim", "RefusedEnded", "RefusedHistory"];

pub fn refused(answer: &Answer) -> bool {
    REFUSALS.contains(&answer.0)
}

fn refusal(status: &'static str) -> Answer {
    (status, BTreeSet::new())
}

/// Whether `first` claims the loop scheme: a declared partition, revival,
/// split, merge or dissolve.
pub fn claims(first: &First) -> bool {
    matches!(
        first,
        First::Op(
            Op::Born { .. }
                | Op::Revive { .. }
                | Op::Split { .. }
                | Op::SplitAway { .. }
                | Op::Merge { .. }
                | Op::Dissolve { .. }
                | Op::RefuseOverlap { .. }
        )
    )
}

/// The members `first` leaves live on the candidate.
fn candidate_live(model: &Model, first: &First) -> Vec<bool> {
    let mut live = model.live_member.clone();
    match first {
        First::Op(Op::DeleteFree { m } | Op::DeleteMember { m, .. }) => live[*m] = false,
        First::Op(Op::DeleteAnchor { g, .. }) => live[model.groups[g].anchor] = false,
        _ => {}
    }
    live
}

/// Query's answer to `read` on the candidate `first` leaves.
pub fn expected(model: &Model, first: &First, read: Read) -> Answer {
    if claims(first) {
        return refusal("RefusedClaim");
    }
    let live = candidate_live(model, first);
    let ended = |members: &BTreeSet<usize>| members.iter().any(|m| !live[*m]);
    match read {
        Read::MemberOf(m) => {
            if !model.live_member[m] {
                return refusal("RefusedHistory");
            }
            let home = model.owner(m);
            let mut rests = BTreeSet::from([m]);
            if let Some(g) = home {
                rests.extend(model.groups[&g].members.iter().copied());
            }
            if ended(&rests) {
                return refusal("RefusedEnded");
            }
            ("member", home.into_iter().collect())
        }
        Read::Resolve(g, all) => {
            let group = &model.groups[&g];
            if !group.live {
                return refusal("RefusedHistory");
            }
            if ended(&group.members) {
                return refusal("RefusedEnded");
            }
            model.resolve(g, all)
        }
    }
}

/// What the committed world (`model`, after `first` applied) makes true of
/// `read`.
pub fn truth(model: &Model, read: Read) -> Answer {
    match read {
        Read::MemberOf(m) => ("member", model.owner(m).into_iter().collect()),
        Read::Resolve(g, all) => model.resolve(g, all),
    }
}
