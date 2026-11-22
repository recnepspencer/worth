//! Resolution consistency laws: random multi-commit lineage DAGs over real
//! Phase 2 entity/relation finalization and real Phase 3 group transitions.
//! After every commit, every name ever minted resolves identically through
//! the derived index and the brute-force oracle at every basis under every
//! selection, and the incrementally maintained index equals a rebuild.

use std::collections::BTreeSet;

use proptest::prelude::*;

use crate::facade::history::CommitId;
use crate::facade::lineage::{
    PersistentNameBranchIndex, PersistentNameHolder, PersistentNameKey, PersistentNameResolution,
    PersistentNameSelection,
};

use super::name_event_fixtures::synthetic_name;
use super::name_group_world::{member, GroupWorld};
use super::name_index_fixtures::{all, bounded, exact, main_branch, rebuild, side};
use super::name_resolution_oracle::{answer, view_at};
use super::name_world::{key, Keep, LineageWorld, Name, Subject};

#[derive(Clone, Copy, Debug)]
enum Op {
    Create {
        relation: bool,
    },
    Split {
        pick: usize,
        pieces: u8,
    },
    Merge {
        first: usize,
        second: usize,
        keep: u8,
    },
    Retire {
        pick: usize,
    },
    Replace {
        pick: usize,
    },
}

fn arb_op() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => any::<bool>().prop_map(|relation| Op::Create { relation }),
        3 => (any::<usize>(), 2u8..=3).prop_map(|(pick, pieces)| Op::Split { pick, pieces }),
        4 => (any::<usize>(), any::<usize>(), 0u8..3)
            .prop_map(|(first, second, keep)| Op::Merge { first, second, keep }),
        2 => any::<usize>().prop_map(|pick| Op::Retire { pick }),
        1 => any::<usize>().prop_map(|pick| Op::Replace { pick }),
    ]
}

/// One commit: entity/relation postures and an optional group partition
/// assigning members 1..=6 to slots 0..3 (slot 3 is absent).
type CommitPlan = (Vec<Op>, Option<Vec<u8>>);

fn arb_commit() -> impl Strategy<Value = CommitPlan> {
    (
        prop::collection::vec(arb_op(), 1..=4),
        prop::option::weighted(0.7, prop::collection::vec(0u8..4, 6)),
    )
}

fn carrier_pool() -> [Name; 3] {
    [
        synthetic_name(0xa1, 1),
        synthetic_name(0xa1, 2),
        synthetic_name(0xa1, 3),
    ]
}

fn selections() -> Vec<PersistentNameSelection> {
    let [a, b, c] = carrier_pool();
    vec![
        exact(),
        all(),
        side("p0"),
        side("p1"),
        side("absent"),
        bounded(&[a]),
        bounded(&[b]),
        bounded(&[c]),
        bounded(&[a, b]),
        bounded(&[member(2)]),
        bounded(&[member(3), member(4)]),
        // Empty filter payloads must refuse, never match as wildcards.
        side(""),
        bounded(&[]),
    ]
}

fn subject_of(holder: &PersistentNameHolder) -> Subject {
    if matches!(holder, PersistentNameHolder::Relation(_)) {
        Subject::Relation
    } else {
        Subject::Entity
    }
}

/// Stages `op` against names not yet claimed in this commit.
fn stage(world: &mut LineageWorld, op: Op, used: &mut BTreeSet<Name>) {
    let free: Vec<_> = world
        .live_names()
        .into_iter()
        .filter(|(name, _)| !used.contains(name))
        .collect();
    let pick = |index: usize, candidates: &[(Name, PersistentNameHolder)]| {
        (!candidates.is_empty()).then(|| candidates[index % candidates.len()].clone())
    };
    match op {
        Op::Create { relation } => {
            // Faces are born without a discriminator, seams with one.
            let (subject, label) = if relation {
                (Subject::Relation, "c")
            } else {
                (Subject::Entity, "")
            };
            world.create(subject, label);
        }
        Op::Split { pick: at, pieces } => {
            let Some((parent, _)) = pick(at, &free) else {
                return;
            };
            used.insert(parent);
            let labels = ["p0", "p1", "p2"];
            let children = world.split(parent, &labels[..usize::from(pieces)]);
            let [a, b, c] = carrier_pool();
            let carriers: [&[Name]; 3] = [&[a, b], &[b], &[c]];
            for (child, carriers) in children.iter().zip(carriers) {
                world.bind_carriers(child, carriers);
            }
        }
        Op::Merge {
            first,
            second,
            keep,
        } => {
            let Some((one, holder)) = pick(first, &free) else {
                return;
            };
            let peers: Vec<_> = free
                .iter()
                .filter(|(name, other)| *name != one && subject_of(other) == subject_of(&holder))
                .cloned()
                .collect();
            let Some((two, _)) = pick(second, &peers) else {
                return;
            };
            used.extend([one, two]);
            let keep = [Keep::InPlace, Keep::Recreated, Keep::MintNew][usize::from(keep)];
            world.merge(&[one, two], keep);
        }
        Op::Retire { pick: at } => {
            let Some((name, _)) = pick(at, &free) else {
                return;
            };
            used.insert(name);
            world.retire(name);
        }
        Op::Replace { pick: at } => {
            let faces: Vec<_> = free
                .iter()
                .filter(|(_, holder)| subject_of(holder) == Subject::Entity)
                .cloned()
                .collect();
            let Some((name, _)) = pick(at, &faces) else {
                return;
            };
            used.insert(name);
            world.replace(name);
        }
    }
}

fn partition(slots: &[u8]) -> Vec<Vec<u8>> {
    let mut groups = vec![Vec::new(); 3];
    for (n, slot) in (1u8..=6).zip(slots) {
        if *slot < 3 {
            groups[usize::from(*slot)].push(n);
        }
    }
    groups.retain(|group| !group.is_empty());
    groups
}

fn run(plans: &[CommitPlan]) {
    let mut world = LineageWorld::new();
    let mut groups = GroupWorld::new();
    let mut log = vec![world.log()[0].clone()];
    let mut index = PersistentNameBranchIndex::new(main_branch(), None);
    index.apply(&log[0]).unwrap();
    for (ops, slots) in plans {
        let mut used = BTreeSet::new();
        for op in ops {
            stage(&mut world, *op, &mut used);
        }
        let mut batch = world.commit();
        if let Some(slots) = slots {
            let proposed = partition(slots);
            let proposed: Vec<&[u8]> = proposed.iter().map(Vec::as_slice).collect();
            if let Ok(transition) = groups.step(&proposed) {
                batch = batch.with_group_transition(&transition);
            }
        }
        index.apply(&batch).expect("committed batch ingests");
        log.push(batch);
        assert_eq!(rebuild(&log), index, "incremental index equals a rebuild");

        let head = world.head().0;
        for (name, holder) in world.live_names() {
            let at_head = index.resolve(key(&holder, name), CommitId(head), &exact());
            let Ok(PersistentNameResolution::Exact(target)) = at_head.result else {
                panic!("a live name resolves to itself");
            };
            assert_eq!(target.key(), key(&holder, name));
        }
        for group in groups.live_groups() {
            let held = PersistentNameKey::Group(group);
            assert_eq!(
                index
                    .resolve(held, CommitId(head), &exact())
                    .metrics
                    .index_probes,
                1
            );
        }
        let keys = index.name_keys();
        for basis in 0..=head + 1 {
            let view = view_at(&log, basis);
            for held in &keys {
                for selection in selections() {
                    let derived = index.resolve(*held, CommitId(basis), &selection).result;
                    let expected = answer(&view, *held, &selection);
                    assert_eq!(derived, expected, "{held:?} at {basis} under {selection:?}");
                }
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn derived_index_resolution_agrees_with_brute_force_reachability(
        plans in prop::collection::vec(arb_commit(), 2..=7)
    ) {
        run(&plans);
    }
}
