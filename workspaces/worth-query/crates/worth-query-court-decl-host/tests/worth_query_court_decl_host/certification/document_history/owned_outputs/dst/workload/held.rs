//! Workloads the sweep holds on a public issue. A held choice is narrowed to
//! what still runs, and `regressions` keeps its reproducer `#[ignore]`d on the
//! issue until the fix lands.
use super::{Op, StagePick};

/// Held on public #348: one envelope that retires an owner and has another
/// stage on a live member refuses, though each stage commits alone. Traced:
/// a set then a retirement (seed 0x7b020009, `OwnedPublicationRequiresCascade`;
/// retirement first, `MissingCoupledOwner` by probe) and two retirements
/// (seed 0x7b020020, `OwnedPublicationRequiresCascade`). The generator cannot
/// tell a set from a creation without the state, so a retirement next to any
/// other stage is dropped, a creation's partner too; two keep the first.
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
