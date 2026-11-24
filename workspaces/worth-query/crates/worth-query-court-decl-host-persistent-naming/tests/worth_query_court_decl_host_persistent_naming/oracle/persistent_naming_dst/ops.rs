//! Seeded operation choice: every step picks one action the model enables
//! from the current reference state, deterministically from a `u64` seed.
use std::collections::BTreeSet;

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
}

/// SplitMix64: a tiny, portable, seed-stable generator.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }

    fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.below(items.len())]
    }

    fn placement(&mut self) -> Placement {
        self.pick(&[
            Placement::Owned("a"),
            Placement::Owned("b"),
            Placement::Issued,
        ])
    }
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
    let two = |rng: &mut Rng| {
        let first = rng.pick(&live);
        let rest: Vec<usize> = live.iter().copied().filter(|n| *n != first).collect();
        [first, rng.pick(&rest)]
    };
    match rng.pick(&kinds) {
        Kind::Create => Op::Create {
            name: minted,
            at: rng.placement(),
        },
        Kind::Duplicate => Op::Duplicate {
            name: rng.pick(&live_authored),
        },
        Kind::Unnamed => Op::Unnamed {
            at: rng.placement(),
            pair: rng.below(2) == 0,
        },
        Kind::Preserve => Op::Preserve {
            name: rng.pick(&live_authored),
            at: rng.placement(),
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
            at: rng.placement(),
        },
        Kind::Retire => Op::Retire {
            name: rng.pick(&live),
        },
        Kind::Restore => Op::Restore {
            name: rng.pick(&restorable),
            at: rng.placement(),
        },
        Kind::Rebuild => Op::Rebuild,
        Kind::Coupled => Op::Coupled {
            names: [minted, minted + 1],
            at: [rng.placement(), rng.placement()],
        },
        Kind::CoupledDuplicate => Op::CoupledDuplicate {
            name: minted,
            at: rng.placement(),
        },
    }
}
