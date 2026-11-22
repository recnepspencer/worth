//! Hostile interleaving state-machine `proptest` for Phase 5 persistent name
//! continuity across `Create`, `Split`, `Merge`, in-commit `Regenerate`,
//! `Retire`, `Restore` (explicit and birth-classified, including re-split child
//! and `Merge(MintNew)` birth-restore replay), and independent occurrences over
//! Entities, Relations, and Groups.

use std::sync::atomic::{AtomicUsize, Ordering};

use proptest::prelude::*;
use proptest::test_runner::TestRunner;

use crate::facade::lineage::{PersistentNameHolder, PersistentNameSelection};

use super::name_event_fixtures::synthetic_name;
use super::name_group_world::member;
use super::name_history_proptest_harness::{Harness, ReplayHits};
use super::name_index_fixtures::{all, bounded, exact, side};
use super::name_world::{Name, Subject};

#[derive(Clone, Copy, Debug)]
pub(super) enum HistoryOp {
    Create {
        relation: bool,
        identical_label: bool,
    },
    Split {
        pick: usize,
        pieces: u8,
        replay_retired: bool,
    },
    Merge {
        first: usize,
        second: usize,
        keep: u8,
        replay_retired: bool,
    },
    Regenerate {
        pick: usize,
    },
    Retire {
        pick: usize,
    },
    Restore {
        pick: usize,
        via_birth: bool,
    },
}

#[derive(Clone, Debug)]
pub(super) enum PlanStep {
    Random {
        ops: Vec<HistoryOp>,
        slots: Option<Vec<u8>>,
    },
    UndoRedoSplit {
        relation: bool,
        pieces: u8,
        via_birth: bool,
        interleave: Vec<HistoryOp>,
    },
    UndoRedoMergeMintNew {
        relation: bool,
        via_birth: bool,
        interleave: Vec<HistoryOp>,
    },
}

pub(super) fn carrier_pool() -> [Name; 3] {
    [
        synthetic_name(0xb2, 1),
        synthetic_name(0xb2, 2),
        synthetic_name(0xb2, 3),
    ]
}

pub(super) fn selections() -> Vec<PersistentNameSelection> {
    let [a, b, c] = carrier_pool();
    vec![
        exact(),
        all(),
        side("p0"),
        side("p1"),
        side("shared"),
        side("absent"),
        bounded(&[a]),
        bounded(&[b]),
        bounded(&[a, b]),
        bounded(&[c]),
        bounded(&[member(2)]),
    ]
}

pub(super) fn subject_of(holder: &PersistentNameHolder) -> Subject {
    if matches!(holder, PersistentNameHolder::Relation(_)) {
        Subject::Relation
    } else {
        Subject::Entity
    }
}

pub(super) fn partition(slots: &[u8]) -> Vec<Vec<u8>> {
    let mut groups = vec![Vec::new(); 3];
    for (n, slot) in (1u8..=6).zip(slots) {
        if *slot < 3 {
            groups[usize::from(*slot)].push(n);
        }
    }
    groups.retain(|group| !group.is_empty());
    groups
}

fn arb_history_op() -> impl Strategy<Value = HistoryOp> {
    prop_oneof![
        3 => (any::<bool>(), any::<bool>())
            .prop_map(|(relation, identical_label)| HistoryOp::Create {
                relation,
                identical_label,
            }),
        3 => (any::<usize>(), 2u8..=3, any::<bool>()).prop_map(
            |(pick, pieces, replay_retired)| HistoryOp::Split {
                pick,
                pieces,
                replay_retired,
            }
        ),
        3 => (any::<usize>(), any::<usize>(), 0u8..3, any::<bool>()).prop_map(
            |(first, second, keep, replay_retired)| HistoryOp::Merge {
                first,
                second,
                keep,
                replay_retired,
            }
        ),
        3 => any::<usize>().prop_map(|pick| HistoryOp::Regenerate { pick }),
        3 => any::<usize>().prop_map(|pick| HistoryOp::Retire { pick }),
        4 => (any::<usize>(), any::<bool>())
            .prop_map(|(pick, via_birth)| HistoryOp::Restore { pick, via_birth }),
    ]
}

fn arb_step() -> impl Strategy<Value = PlanStep> {
    prop_oneof![
        4 => (
            prop::collection::vec(arb_history_op(), 1..=3),
            prop::option::weighted(0.6, prop::collection::vec(0u8..4, 6)),
        )
            .prop_map(|(ops, slots)| PlanStep::Random { ops, slots }),
        2 => (
            any::<bool>(),
            2u8..=3,
            any::<bool>(),
            prop::collection::vec(arb_history_op(), 0..=2),
        )
            .prop_map(|(relation, pieces, via_birth, interleave)| PlanStep::UndoRedoSplit {
                relation,
                pieces,
                via_birth,
                interleave,
            }),
        2 => (
            any::<bool>(),
            any::<bool>(),
            prop::collection::vec(arb_history_op(), 0..=2),
        )
            .prop_map(|(relation, via_birth, interleave)| PlanStep::UndoRedoMergeMintNew {
                relation,
                via_birth,
                interleave,
            }),
    ]
}

#[test]
fn hostile_history_interleaving_preserves_oracle_rebuild_immutability_and_fresh_generations() {
    let split_hits = AtomicUsize::new(0);
    let merge_hits = AtomicUsize::new(0);
    let mut runner = TestRunner::new(ProptestConfig::with_cases(48));
    let strategy = prop::collection::vec(arb_step(), 2..=3);
    runner
        .run(&strategy, |steps| {
            let mut harness = Harness::new(ReplayHits {
                split: &split_hits,
                merge: &merge_hits,
            });
            for step in &steps {
                harness.run_step(step);
            }
            Ok(())
        })
        .expect("hostile interleaving state machine must preserve all invariants");
    let split = split_hits.load(Ordering::Relaxed);
    let merge = merge_hits.load(Ordering::Relaxed);
    eprintln!("name_history_proptest replay hits over 48 cases: split={split}, merge={merge}");
    assert!(
        split >= 4 && merge >= 4,
        "expected >=4 split and merge replay hits across 48 cases, got split={split}, merge={merge}"
    );
}
