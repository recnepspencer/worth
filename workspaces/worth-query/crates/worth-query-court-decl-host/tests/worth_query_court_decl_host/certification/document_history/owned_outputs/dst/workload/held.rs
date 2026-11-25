//! Workloads the sweep holds on a public issue. A held choice is narrowed to
//! what still runs, and `regressions` keeps its reproducer `#[ignore]`d on the
//! issue until the fix lands.
use super::{Op, StagePick};

/// Held on public #369 (per-stage retirement events): a retirement in a
/// later stage of one envelope refuses typed `LaterStageRetirementPostponed`.
/// #348 (PR #374) released retirement-first envelopes, but in the DST's push
/// order (`reference::push_order`: the child before its owner, anything
/// before a retirement of `e2`) a retirement paired with another stage is
/// always the later one, so every such pair stays narrowed. The generator
/// cannot tell a set from a creation without the state, so a retirement next
/// to any other stage is dropped, a creation's partner too; two keep the
/// first.
pub fn narrow(op: Op) -> Op {
    let narrow = |picks: [StagePick; 2]| match picks {
        [StagePick::Retire, StagePick::Retire] => [StagePick::Retire, StagePick::Skip],
        [StagePick::Retire, other] | [other, StagePick::Retire] if other != StagePick::Skip => {
            picks.map(|pick| if pick == StagePick::Retire { StagePick::Skip } else { pick })
        }
        picks => picks,
    };
    match op {
        Op::Edit(picks) => Op::Edit(narrow(picks)),
        Op::Stopped { stages, at } => Op::Stopped { stages: narrow(stages), at },
        Op::OutOfBand(picks) => Op::OutOfBand(narrow(picks)),
        other => other,
    }
}
