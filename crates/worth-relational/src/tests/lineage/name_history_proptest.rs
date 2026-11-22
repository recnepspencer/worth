//! Hostile interleaving state-machine `proptest` for Phase 5 persistent name
//! continuity across `Create`, `Split`, `Merge`, in-commit `Regenerate`,
//! `Retire`, `Restore` (both explicit and birth-classified), and independent
//! occurrences over Entities, Relations, and Groups.

use std::collections::BTreeSet;

use proptest::prelude::*;

use crate::facade::history::CommitId;
use crate::facade::lineage::{
    LineageSubject, PersistentNameBranchIndex, PersistentNameHolder, PersistentNameResolution,
    PersistentNameSelection, PersistentNameTarget,
};

use super::name_event_fixtures::synthetic_name;
use super::name_group_world::{member, GroupWorld};
use super::name_index_fixtures::{all, bounded, exact, main_branch, rebuild, side};
use super::name_resolution_oracle::{answer, view_at};
use super::name_world::{key, Keep, LineageWorld, Name, Subject};

#[derive(Clone, Copy, Debug)]
enum HistoryOp {
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

fn arb_history_op() -> impl Strategy<Value = HistoryOp> {
    prop_oneof![
        3 => (any::<bool>(), any::<bool>())
            .prop_map(|(relation, identical_label)| HistoryOp::Create { relation, identical_label }),
        3 => (any::<usize>(), 2u8..=3, any::<bool>()).prop_map(|(pick, pieces, replay_retired)| {
            HistoryOp::Split {
                pick,
                pieces,
                replay_retired,
            }
        }),
        3 => (any::<usize>(), any::<usize>(), 0u8..3, any::<bool>()).prop_map(
            |(first, second, keep, replay_retired)| HistoryOp::Merge {
                first,
                second,
                keep,
                replay_retired,
            },
        ),
        3 => any::<usize>().prop_map(|pick| HistoryOp::Regenerate { pick }),
        3 => any::<usize>().prop_map(|pick| HistoryOp::Retire { pick }),
        4 => (any::<usize>(), any::<bool>())
            .prop_map(|(pick, via_birth)| HistoryOp::Restore { pick, via_birth }),
    ]
}

type CommitPlan = (Vec<HistoryOp>, Option<Vec<u8>>);

fn arb_commit() -> impl Strategy<Value = CommitPlan> {
    (
        prop::collection::vec(arb_history_op(), 1..=4),
        prop::option::weighted(0.6, prop::collection::vec(0u8..4, 6)),
    )
}

fn carrier_pool() -> [Name; 3] {
    [
        synthetic_name(0xb2, 1),
        synthetic_name(0xb2, 2),
        synthetic_name(0xb2, 3),
    ]
}

fn selections() -> Vec<PersistentNameSelection> {
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

fn subject_of(holder: &PersistentNameHolder) -> Subject {
    if matches!(holder, PersistentNameHolder::Relation(_)) {
        Subject::Relation
    } else {
        Subject::Entity
    }
}

fn stage(
    world: &mut LineageWorld,
    op: HistoryOp,
    used_live: &mut BTreeSet<Name>,
    used_retired: &mut BTreeSet<Name>,
) {
    let free_live: Vec<_> = world
        .live_names()
        .into_iter()
        .filter(|(name, _)| !used_live.contains(name))
        .collect();
    let pick = |index: usize, items: &[(Name, PersistentNameHolder)]| {
        (!items.is_empty()).then(|| items[index % items.len()].clone())
    };
    match op {
        HistoryOp::Create {
            relation,
            identical_label,
        } => {
            let subject = if relation {
                Subject::Relation
            } else {
                Subject::Entity
            };
            let label = if identical_label {
                "shared"
            } else if relation {
                "s"
            } else {
                ""
            };
            world.create(subject, label);
        }
        HistoryOp::Split {
            pick: at,
            pieces,
            replay_retired,
        } => {
            let Some((parent, _)) = pick(at, &free_live) else {
                return;
            };
            used_live.insert(parent);
            let replayable: Vec<Name> = if replay_retired {
                world
                    .retired_children_of(parent)
                    .into_iter()
                    .filter(|c| !used_retired.contains(c))
                    .take(usize::from(pieces))
                    .collect()
            } else {
                Vec::new()
            };
            let children = if replayable.len() >= 2 {
                used_retired.extend(replayable.iter().copied());
                world.resplit_replaying_retired(parent, &replayable)
            } else {
                let labels = ["p0", "p1", "p2"];
                world.split(parent, &labels[..usize::from(pieces)])
            };
            let [a, b, c] = carrier_pool();
            let carriers: [&[Name]; 3] = [&[a, b], &[b], &[c]];
            for (child, carriers) in children.iter().zip(carriers) {
                world.bind_carriers(child, carriers);
            }
        }
        HistoryOp::Merge {
            first,
            second,
            keep,
            replay_retired,
        } => {
            let Some((one, holder)) = pick(first, &free_live) else {
                return;
            };
            let peers: Vec<_> = free_live
                .iter()
                .filter(|(name, other)| *name != one && subject_of(other) == subject_of(&holder))
                .cloned()
                .collect();
            let Some((two, _)) = pick(second, &peers) else {
                return;
            };
            used_live.extend([one, two]);
            let keep = [Keep::InPlace, Keep::Recreated, Keep::MintNew][usize::from(keep)];
            if keep == Keep::MintNew
                && replay_retired
                && let Some(retired_merged) = world
                    .retired_merge_of(&[one, two])
                    .filter(|m| !used_retired.contains(m))
            {
                used_retired.insert(retired_merged);
                world.remerge_mint_new_replaying_retired(&[one, two], retired_merged);
            } else {
                world.merge(&[one, two], keep);
            }
        }
        HistoryOp::Regenerate { pick: at } => {
            let Some((name, _)) = pick(at, &free_live) else {
                return;
            };
            used_live.insert(name);
            world.regenerate(name);
        }
        HistoryOp::Retire { pick: at } => {
            let Some((name, _)) = pick(at, &free_live) else {
                return;
            };
            used_live.insert(name);
            world.retire(name);
        }
        HistoryOp::Restore {
            pick: at,
            via_birth,
        } => {
            let free_retired: Vec<(Name, LineageSubject)> = world
                .retired_names()
                .into_iter()
                .filter(|(name, _)| !used_retired.contains(name))
                .collect();
            if free_retired.is_empty() {
                return;
            }
            let (name, _) = free_retired[at % free_retired.len()];
            used_retired.insert(name);
            let fresh = world.restore(name, via_birth);
            let [a, _, _] = carrier_pool();
            if via_birth {
                world.bind_carriers(&fresh, &[a]);
            }
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
    let mut serialized_history: Vec<Vec<String>> = vec![log[0]
        .lineage_events
        .iter()
        .map(|e| serde_json::to_string(e).unwrap())
        .collect()];
    let mut index = PersistentNameBranchIndex::new(main_branch(), None);
    index.apply(&log[0]).unwrap();
    let mut seen_retired_holders = BTreeSet::<PersistentNameHolder>::new();
    let mut previous_live_holders = BTreeSet::<PersistentNameHolder>::new();

    for (ops, slots) in plans {
        let mut used_live = BTreeSet::new();
        let mut used_retired = BTreeSet::new();
        for op in ops {
            stage(&mut world, *op, &mut used_live, &mut used_retired);
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
        log.push(batch.clone());
        serialized_history.push(
            batch
                .lineage_events
                .iter()
                .map(|e| serde_json::to_string(e).unwrap())
                .collect(),
        );

        // (b) Arch 18 rebuild equivalence
        assert_eq!(rebuild(&log), index, "incremental index equals rebuild");

        // (c) Arch 8/14 append-only immutability of every prior commit's lineage events
        for (idx, expected) in serialized_history.iter().enumerate() {
            let actual: Vec<String> = log[idx]
                .lineage_events
                .iter()
                .map(|e| serde_json::to_string(e).unwrap())
                .collect();
            assert_eq!(&actual, expected, "prior commit {idx} events mutated");
        }

        // (d) Fresh-generation distinctness: no live record ever reuses a retired handle
        let head = world.head().0;
        let mut current_live_holders = BTreeSet::new();
        for (name, holder) in world.live_names() {
            assert!(
                !seen_retired_holders.contains(&holder),
                "live name {name:?} reused retired holder {holder:?}"
            );
            assert!(current_live_holders.insert(holder.clone()));
            let at_head = index.resolve(key(&holder, name), CommitId(head), &exact());
            match at_head.result {
                Ok(PersistentNameResolution::Exact(PersistentNameTarget::Entity {
                    entity,
                    ..
                })) => {
                    assert_eq!(PersistentNameHolder::Entity(entity), holder);
                }
                Ok(PersistentNameResolution::Exact(PersistentNameTarget::Relation {
                    relation,
                    ..
                })) => {
                    assert_eq!(PersistentNameHolder::Relation(relation), holder);
                }
                other => panic!("live name must resolve exact to its live holder, got {other:?}"),
            }
        }
        for old in &previous_live_holders {
            if !current_live_holders.contains(old) {
                seen_retired_holders.insert(old.clone());
            }
        }
        previous_live_holders = current_live_holders;

        // (a) 100% agreement with brute-force oracle at every basis under every selection
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
    fn hostile_history_interleaving_preserves_oracle_rebuild_immutability_and_fresh_generations(
        plans in prop::collection::vec(arb_commit(), 3..=8)
    ) {
        run(&plans);
    }
}
