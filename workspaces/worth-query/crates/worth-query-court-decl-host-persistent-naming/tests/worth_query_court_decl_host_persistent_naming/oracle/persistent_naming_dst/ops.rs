//! Seeded operation choice: every step picks one action the model enables
//! from the current reference state, deterministically from a `u64` seed.
use std::collections::BTreeSet;
use worth_dst::Rng;

use super::reference::Oracle;

/// Names one seed may mint; the trace model declares `n1..n16`.
pub const MAX_NAMES: usize = 14;

/// Where a created record lives: in model `a`'s partition owned by model
/// `a` or `b`, or in the partition the commit issues, owned by no model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Placement {
    Owned(&'static str),
    Issued,
}

impl Placement {
    pub fn pick(rng: &mut Rng) -> Self {
        rng.pick(&[Self::Owned("a"), Self::Owned("b"), Self::Issued])
    }

    pub fn part(self) -> &'static str {
        match self {
            Self::Owned(_) => "p1",
            Self::Issued => "p2",
        }
    }

    pub fn owner(self) -> Option<&'static str> {
        match self {
            Self::Owned(owner) => Some(owner),
            Self::Issued => None,
        }
    }
}

/// One DST action, by model name index.
#[derive(Clone, Debug)]
pub enum Op {
    Create {
        name: usize,
        at: Placement,
    },
    /// Re-mints a live name's exact birth: must refuse typed, zero effects.
    Duplicate {
        name: usize,
    },
    /// Creates a record of the schemed kind without naming it (beside a
    /// named sibling when `pair`): must refuse `MissingBirth`, zero effects.
    Unnamed {
        at: Placement,
        pair: bool,
    },
    Preserve {
        name: usize,
        at: Placement,
    },
    /// Both pieces share one creation partition (`at`); owned pieces may be
    /// held by two distinct models (#133).
    Split {
        parent: usize,
        pieces: [usize; 2],
        at: [Placement; 2],
    },
    /// Keeps `keeper` in place. `listed` merges through `merge_least` with
    /// the parents in that seeded order; the keeper is then the least name.
    MergeKeep {
        keeper: usize,
        other: usize,
        listed: Option<[usize; 2]>,
    },
    /// A least-keeper merge before the harness settles its keeper.
    MergeLeast {
        parents: [usize; 2],
    },
    MergeMint {
        parents: [usize; 2],
        minted: usize,
        at: Placement,
    },
    Retire {
        name: usize,
    },
    Restore {
        name: usize,
        at: Placement,
    },
    Rebuild,
    /// One edit envelope whose two authored stages each create a record and
    /// birth one fresh name: their plans compose into the one commit's plan.
    Coupled {
        names: [usize; 2],
        at: [Placement; 2],
    },
    /// Both stages of one envelope mint fresh name `name`: the composed plan
    /// must refuse `DuplicateBirth` with zero effects from either stage.
    CoupledDuplicate {
        name: usize,
        at: Placement,
    },
    /// One envelope: `first` stages a mutation, then a later stage reads a
    /// name on the unpublished candidate; `expected` is the reference
    /// answer, and a refusal leaves zero effects from either stage.
    Candidate {
        first: super::candidate::First,
        read: super::candidate::Read,
        expected: super::reference::Answer,
    },
}

#[derive(Clone, Copy)]
enum Kind {
    Create,
    Duplicate,
    Unnamed,
    Preserve,
    Split,
    MergeKeep,
    MergeLeast,
    MergeMint,
    Retire,
    Restore,
    Rebuild,
    Coupled,
    CoupledDuplicate,
    Candidate,
}

/// Picks the next enabled action. `authored` are the names whose exact
/// birth the harness can re-mint (first births and their restorations);
/// a seed destroys and rebuilds its index at most once (`rebuilt`).
pub fn next_op(
    rng: &mut Rng,
    model: &Oracle,
    authored: &BTreeSet<usize>,
    minted: usize,
    rebuilt: bool,
) -> Op {
    let live: Vec<usize> = model.live().into_iter().collect();
    let live_authored: Vec<usize> = live
        .iter()
        .copied()
        .filter(|n| authored.contains(n))
        .collect();
    let restorable: Vec<usize> = model
        .retired()
        .into_iter()
        .filter(|n| authored.contains(n))
        .collect();
    let room = MAX_NAMES - minted;
    let mut kinds = Vec::new();
    let mut enable = |kind: Kind, weight: usize, enabled: bool| {
        if enabled {
            kinds.extend(std::iter::repeat_n(kind, weight));
        }
    };
    enable(Kind::Create, if live.len() < 2 { 8 } else { 3 }, room >= 1);
    enable(Kind::Duplicate, 1, !live_authored.is_empty());
    enable(Kind::Unnamed, 1, true);
    enable(Kind::Preserve, 2, !live_authored.is_empty());
    enable(Kind::Split, 3, !live.is_empty() && room >= 2);
    enable(Kind::MergeKeep, 2, live.len() >= 2);
    enable(Kind::MergeLeast, 2, live.len() >= 2);
    enable(Kind::MergeMint, 2, live.len() >= 2 && room >= 1);
    enable(Kind::Retire, 2, !live.is_empty());
    enable(Kind::Restore, 2, !restorable.is_empty());
    enable(Kind::Rebuild, 1, model.events() > 0 && !rebuilt);
    enable(Kind::Coupled, 2, room >= 2);
    enable(Kind::CoupledDuplicate, 1, room >= 1);
    enable(
        Kind::Candidate,
        3,
        !live.is_empty() || room >= 1 || !restorable.is_empty(),
    );
    let two = |rng: &mut Rng| {
        let first = rng.pick(&live);
        let rest: Vec<usize> = live.iter().copied().filter(|n| *n != first).collect();
        [first, rng.pick(&rest)]
    };
    match rng.pick(&kinds) {
        Kind::Create => Op::Create {
            name: minted,
            at: Placement::pick(rng),
        },
        Kind::Duplicate => Op::Duplicate {
            name: rng.pick(&live_authored),
        },
        Kind::Unnamed => Op::Unnamed {
            at: Placement::pick(rng),
            pair: rng.below(2) == 0,
        },
        Kind::Preserve => Op::Preserve {
            name: rng.pick(&live_authored),
            at: Placement::pick(rng),
        },
        Kind::Split => {
            let at = if rng.below(4) == 0 {
                [Placement::Issued; 2]
            } else {
                // Mostly across two distinct owners: the #133 multi-body case.
                let first = rng.pick(&["a", "b"]);
                let second = if rng.below(4) == 0 {
                    first
                } else if first == "a" {
                    "b"
                } else {
                    "a"
                };
                [Placement::Owned(first), Placement::Owned(second)]
            };
            Op::Split {
                parent: rng.pick(&live),
                pieces: [minted, minted + 1],
                at,
            }
        }
        Kind::MergeKeep => {
            let [keeper, other] = two(rng);
            Op::MergeKeep {
                keeper,
                other,
                listed: None,
            }
        }
        Kind::MergeLeast => Op::MergeLeast { parents: two(rng) },
        Kind::MergeMint => Op::MergeMint {
            parents: two(rng),
            minted,
            at: Placement::pick(rng),
        },
        Kind::Retire => Op::Retire {
            name: rng.pick(&live),
        },
        Kind::Restore => Op::Restore {
            name: rng.pick(&restorable),
            at: Placement::pick(rng),
        },
        Kind::Rebuild => Op::Rebuild,
        Kind::Coupled => Op::Coupled {
            names: [minted, minted + 1],
            at: [Placement::pick(rng), Placement::pick(rng)],
        },
        Kind::CoupledDuplicate => Op::CoupledDuplicate {
            name: minted,
            at: Placement::pick(rng),
        },
        Kind::Candidate => super::candidate::next(rng, model, authored, minted),
    }
}
