//! Candidate-basis name reads (Slice 3C-1): one edit envelope whose first
//! stage makes one mutation (`First`) and whose last stage, the probe, reads
//! one name on the unpublished candidate (`Read`). The reference answer is
//! `PersistentNamingCandidate.tla`'s `ImplRead`, transcribed: the basis
//! answer, admitted only where the candidate left every live record it rests
//! on unchanged, and the name the envelope gives a record the first stage
//! created (#426); otherwise a typed refusal, and the envelope commits
//! nothing.
use std::collections::BTreeSet;
use worth_dst::Rng;

use super::ops::{Op, Placement};
use super::reference::{Answer, Oracle, SCOPES};

/// The earlier stage's mutation.
#[derive(Clone, Debug)]
pub enum First {
    /// A field edit of the record a live name holds: no naming effect.
    Edit(usize),
    /// One naming commit: `Create`, `Preserve`, `Restore`, `Retire`,
    /// `Split` or an unlisted `MergeKeep`.
    Commit(Box<Op>),
}

/// The later stage's decision read.
#[derive(Clone, Copy, Debug)]
pub enum Read {
    Resolve {
        name: usize,
        scope: Option<&'static str>,
        all: bool,
    },
    /// `name_of` the basis record a live name holds.
    NameOf(usize),
    /// `name_of` the record the first stage created and named: the name
    /// the envelope gives it, which its commit then records (#426).
    NameOfCreated,
}

impl First {
    /// Names whose basis record the first stage ends.
    pub fn ended(&self) -> BTreeSet<usize> {
        let ended = match self {
            Self::Edit(_) => None,
            Self::Commit(op) => match op.as_ref() {
                Op::Preserve { name, .. } | Op::Retire { name } => Some(*name),
                Op::Split { parent, .. } => Some(*parent),
                Op::MergeKeep { other, .. } => Some(*other),
                _ => None,
            },
        };
        ended.into_iter().collect()
    }

    /// The name the first stage gives the (first) record it creates.
    pub fn created(&self) -> Option<usize> {
        match self {
            Self::Edit(_) => None,
            Self::Commit(op) => match op.as_ref() {
                Op::Create { name, .. } | Op::Preserve { name, .. } | Op::Restore { name, .. } => {
                    Some(*name)
                }
                Op::Split { pieces, .. } => Some(pieces[0]),
                _ => None,
            },
        }
    }

    fn creates(&self) -> bool {
        self.created().is_some()
    }
}

fn refused(status: &'static str) -> Answer {
    Answer {
        status,
        targets: BTreeSet::new(),
    }
}

/// Whether an answer is a candidate-basis refusal.
pub fn is_refusal(answer: &Answer) -> bool {
    answer.status.starts_with("Refused")
}

/// The reference answer of `read` after `first` on `model`'s basis.
pub fn expected(model: &Oracle, first: &First, read: Read) -> Answer {
    let ended = first.ended();
    match read {
        Read::Resolve { name, scope, all } => {
            if !model.is_born(name) {
                return refused("RefusedHistory");
            }
            // The live records the basis answer rests on, every owner.
            let matched = model.resolve(name, None, true).targets;
            if !matched.is_disjoint(&ended) {
                refused("RefusedEnded")
            } else if !model.is_live(name) {
                refused("RefusedHistory")
            } else {
                model.resolve(name, scope, all)
            }
        }
        // Unreachable through a decision (no handle), refused all the same.
        Read::NameOf(name) if ended.contains(&name) => refused("RefusedEnded"),
        Read::NameOf(name) => Answer {
            status: "Named",
            targets: BTreeSet::from([name]),
        },
        // Every record a first stage creates is born named.
        Read::NameOfCreated => match first.created() {
            Some(name) => Answer {
                status: "Named",
                targets: BTreeSet::from([name]),
            },
            None => refused("RefusedUnpublished"),
        },
    }
}

/// Picks a candidate envelope the model enables: a first stage, then a read
/// biased toward what that stage touched.
pub fn next(rng: &mut Rng, model: &Oracle, authored: &BTreeSet<usize>, minted: usize) -> Op {
    let live: Vec<usize> = model.live().into_iter().collect();
    let born: Vec<usize> = model.born().collect();
    let restorable: Vec<usize> = model
        .retired()
        .into_iter()
        .filter(|name| authored.contains(name))
        .collect();
    let live_authored: Vec<usize> = live
        .iter()
        .copied()
        .filter(|name| authored.contains(name))
        .collect();
    let room = super::ops::MAX_NAMES - minted;
    let mut kinds = Vec::new();
    for (kind, enabled) in [
        (0, !live.is_empty()),
        (1, room >= 1),
        (2, !live_authored.is_empty()),
        (3, !restorable.is_empty()),
        (4, !live.is_empty()),
        (5, !live.is_empty() && room >= 2),
        (6, live.len() >= 2),
    ] {
        if enabled {
            kinds.push(kind);
        }
    }
    let first = match rng.pick(&kinds) {
        0 => First::Edit(rng.pick(&live)),
        1 => commit(Op::Create {
            name: minted,
            at: Placement::pick(rng),
        }),
        2 => commit(Op::Preserve {
            name: rng.pick(&live_authored),
            at: Placement::pick(rng),
        }),
        3 => commit(Op::Restore {
            name: rng.pick(&restorable),
            at: Placement::pick(rng),
        }),
        4 => commit(Op::Retire {
            name: rng.pick(&live),
        }),
        5 => {
            let at = match rng.below(3) {
                0 => [Placement::Issued; 2],
                _ => [rng.pick(&["a", "b"]), rng.pick(&["a", "b"])].map(Placement::Owned),
            };
            commit(Op::Split {
                parent: rng.pick(&live),
                pieces: [minted, minted + 1],
                at,
            })
        }
        _ => {
            let keeper = rng.pick(&live);
            let rest: Vec<usize> = live.iter().copied().filter(|n| *n != keeper).collect();
            commit(Op::MergeKeep {
                keeper,
                other: rng.pick(&rest),
                listed: None,
            })
        }
    };
    let ended = first.ended();
    let touched: Vec<usize> = ended.iter().copied().chain(touched(&first)).collect();
    // A decision holds a handle only to a record live on the candidate, so
    // `name_of` reads a record the first stage left live.
    let kept: Vec<usize> = live
        .iter()
        .copied()
        .filter(|n| !ended.contains(n))
        .collect();
    let read = match rng.below(6) {
        0 if first.creates() => Read::NameOfCreated,
        1 | 2 if !kept.is_empty() => {
            let pool = if touched.iter().any(|n| kept.contains(n)) && rng.below(2) == 0 {
                touched
                    .iter()
                    .copied()
                    .filter(|n| kept.contains(n))
                    .collect()
            } else {
                kept.clone()
            };
            Read::NameOf(rng.pick(&pool))
        }
        _ if !born.is_empty() => {
            // Only a born name is held: an unborn one is the courts' case.
            let touched: Vec<usize> = touched.into_iter().filter(|n| born.contains(n)).collect();
            let pool = if !touched.is_empty() && rng.below(2) == 0 {
                touched
            } else {
                born.clone()
            };
            Read::Resolve {
                name: rng.pick(&pool),
                scope: rng.pick(&SCOPES),
                all: rng.below(2) == 0,
            }
        }
        // Nothing is born yet, so the first stage is a `Create`.
        _ => Read::NameOfCreated,
    };
    let expected = expected(model, &first, read);
    Op::Candidate {
        first,
        read,
        expected,
    }
}

fn commit(op: Op) -> First {
    First::Commit(Box::new(op))
}

/// The names a first stage targets beyond those it ends.
fn touched(first: &First) -> Vec<usize> {
    match first {
        First::Edit(name) => vec![*name],
        First::Commit(op) => match op.as_ref() {
            Op::Create { name, .. } | Op::Restore { name, .. } => vec![*name],
            Op::MergeKeep { keeper, .. } => vec![*keeper],
            _ => Vec::new(),
        },
    }
}
