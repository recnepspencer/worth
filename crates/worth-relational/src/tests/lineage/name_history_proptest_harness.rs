//! State-machine harness for [`super::name_history_proptest`]: stages single
//! random operations and multi-commit undo/redo `Split` / `Merge(MintNew)`
//! replay sequences, verifying rebuild equivalence, append-only immutability,
//! fresh generations, and oracle agreement after every commit.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_relational_lineage::internal_port::lineage::index::batch::PersistentNameCommittedBatchFixturePort as _;

use crate::facade::history::CommitId;
use crate::facade::lineage::{
    LineageSubject, PersistentNameBranchIndex, PersistentNameCommittedBatch, PersistentNameHolder,
    PersistentNameResolution, PersistentNameTarget,
};

use super::name_group_world::GroupWorld;
use super::name_history_proptest::{
    carrier_pool, partition, selections, subject_of, HistoryOp, PlanStep,
};
use super::name_index_fixtures::{exact, main_branch, rebuild};
use super::name_resolution_oracle::{answer, view_at};
use super::name_world::{key, Keep, LineageWorld, Name, Subject};

pub(super) struct ReplayHits<'a> {
    pub(super) split: &'a AtomicUsize,
    pub(super) merge: &'a AtomicUsize,
}

pub(super) struct Harness<'a> {
    world: LineageWorld,
    groups: GroupWorld,
    log: Vec<PersistentNameCommittedBatch>,
    serialized_history: Vec<Vec<String>>,
    index: PersistentNameBranchIndex,
    seen_retired_holders: BTreeSet<PersistentNameHolder>,
    previous_live_holders: BTreeSet<PersistentNameHolder>,
    hits: ReplayHits<'a>,
}

impl<'a> Harness<'a> {
    pub(super) fn new(hits: ReplayHits<'a>) -> Self {
        let world = LineageWorld::new();
        let log = vec![world.log()[0].clone()];
        let serialized_history = vec![log[0]
            .lineage_events_for_test()
            .iter()
            .map(|e| serde_json::to_string(e).unwrap())
            .collect()];
        let mut index = PersistentNameBranchIndex::new(main_branch(), None);
        index.apply(&log[0]).unwrap();
        Self {
            world,
            groups: GroupWorld::new(),
            log,
            serialized_history,
            index,
            seen_retired_holders: BTreeSet::new(),
            previous_live_holders: BTreeSet::new(),
            hits,
        }
    }

    fn stage_op(
        &mut self,
        op: HistoryOp,
        used_live: &mut BTreeSet<Name>,
        used_retired: &mut BTreeSet<Name>,
    ) {
        let free_live: Vec<_> = self
            .world
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
                self.world.create(subject, label);
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
                    self.world
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
                    self.hits.split.fetch_add(1, Ordering::Relaxed);
                    self.world.resplit_replaying_retired(parent, &replayable)
                } else {
                    let labels = ["p0", "p1", "p2"];
                    self.world.split(parent, &labels[..usize::from(pieces)])
                };
                let [a, b, c] = carrier_pool();
                for (child, carriers) in children.iter().zip([&[a, b][..], &[b][..], &[c][..]]) {
                    self.world.bind_carriers(child, carriers);
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
                    .filter(|(name, other)| {
                        *name != one && subject_of(other) == subject_of(&holder)
                    })
                    .cloned()
                    .collect();
                let Some((two, _)) = pick(second, &peers) else {
                    return;
                };
                used_live.extend([one, two]);
                let keep = [Keep::InPlace, Keep::Recreated, Keep::MintNew][usize::from(keep)];
                if keep == Keep::MintNew
                    && replay_retired
                    && let Some(retired_merged) = self
                        .world
                        .retired_merge_of(&[one, two])
                        .filter(|m| !used_retired.contains(m))
                {
                    used_retired.insert(retired_merged);
                    self.hits.merge.fetch_add(1, Ordering::Relaxed);
                    self.world
                        .remerge_mint_new_replaying_retired(&[one, two], retired_merged);
                } else {
                    self.world.merge(&[one, two], keep);
                }
            }
            HistoryOp::Regenerate { pick: at } => {
                let Some((name, _)) = pick(at, &free_live) else {
                    return;
                };
                used_live.insert(name);
                self.world.regenerate(name);
            }
            HistoryOp::Retire { pick: at } => {
                let Some((name, _)) = pick(at, &free_live) else {
                    return;
                };
                used_live.insert(name);
                self.world.retire(name);
            }
            HistoryOp::Restore {
                pick: at,
                via_birth,
            } => {
                let free_retired: Vec<(Name, LineageSubject)> = self
                    .world
                    .retired_names()
                    .into_iter()
                    .filter(|(name, _)| !used_retired.contains(name))
                    .collect();
                if free_retired.is_empty() {
                    return;
                }
                let (name, _) = free_retired[at % free_retired.len()];
                used_retired.insert(name);
                let fresh = self.world.restore(name, via_birth);
                if via_birth {
                    self.world.bind_carriers(&fresh, &[carrier_pool()[0]]);
                }
            }
        }
    }

    fn commit_and_verify(&mut self, slots: Option<&[u8]>) {
        let mut batch = self.world.commit();
        if let Some(slots) = slots {
            let proposed = partition(slots);
            let proposed: Vec<&[u8]> = proposed.iter().map(Vec::as_slice).collect();
            if let Ok(transition) = self.groups.step(&proposed) {
                batch = batch.with_published_group_transitions(transition);
            }
        }
        self.index.apply(&batch).expect("committed batch ingests");
        self.log.push(batch.clone());
        self.serialized_history.push(
            batch
                .lineage_events_for_test()
                .iter()
                .map(|e| serde_json::to_string(e).unwrap())
                .collect(),
        );
        assert_eq!(rebuild(&self.log), self.index, "incremental equals rebuild");
        for (idx, expected) in self.serialized_history.iter().enumerate() {
            let actual: Vec<String> = self.log[idx]
                .lineage_events_for_test()
                .iter()
                .map(|e| serde_json::to_string(e).unwrap())
                .collect();
            assert_eq!(&actual, expected, "prior commit {idx} events mutated");
        }
        let head = self.world.head().0;
        let mut current_live = BTreeSet::new();
        for (name, holder) in self.world.live_names() {
            assert!(!self.seen_retired_holders.contains(&holder));
            assert!(current_live.insert(holder.clone()));
            let at_head = self
                .index
                .resolve(key(&holder, name), CommitId(head), &exact());
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
                other => panic!("live name must resolve exact to live holder, got {other:?}"),
            }
        }
        for old in &self.previous_live_holders {
            if !current_live.contains(old) {
                self.seen_retired_holders.insert(old.clone());
            }
        }
        self.previous_live_holders = current_live;
        let keys = self.index.name_keys();
        for basis in 0..=head + 1 {
            let view = view_at(&self.log, basis);
            for held in &keys {
                for selection in selections() {
                    let derived = self
                        .index
                        .resolve(*held, CommitId(basis), &selection)
                        .result;
                    let expected = answer(&view, *held, &selection);
                    assert_eq!(derived, expected, "{held:?} at {basis} under {selection:?}");
                }
            }
        }
    }

    pub(super) fn run_step(&mut self, step: &PlanStep) {
        match step {
            PlanStep::Random { ops, slots } => {
                let (mut used_live, mut used_retired) = (BTreeSet::new(), BTreeSet::new());
                for op in ops {
                    self.stage_op(*op, &mut used_live, &mut used_retired);
                }
                self.commit_and_verify(slots.as_deref());
            }
            PlanStep::UndoRedoSplit {
                relation,
                pieces,
                via_birth,
                interleave,
            } => {
                let subj = if *relation {
                    Subject::Relation
                } else {
                    Subject::Entity
                };
                let parent_holder = self.world.create(subj, "p_root");
                self.commit_and_verify(None);
                let parent = self.world.name_of(&parent_holder);
                let labels = ["p0", "p1", "p2"];
                let kids = self.world.split(parent, &labels[..usize::from(*pieces)]);
                self.commit_and_verify(None);
                let kid_names: Vec<Name> = kids.iter().map(|h| self.world.name_of(h)).collect();
                for &k in &kid_names {
                    self.world.retire(k);
                }
                self.commit_and_verify(None);
                let mut used_live = BTreeSet::new();
                let mut used_retired = BTreeSet::from([parent]);
                used_retired.extend(kid_names.iter().copied());
                self.world.restore(parent, *via_birth);
                for op in interleave {
                    self.stage_op(*op, &mut used_live, &mut used_retired);
                }
                self.commit_and_verify(None);
                let mut used_live = BTreeSet::from([parent]);
                let mut used_retired: BTreeSet<Name> = kid_names.iter().copied().collect();
                let replayed = self.world.resplit_replaying_retired(parent, &kid_names);
                self.world.bind_carriers(&replayed[0], &[carrier_pool()[0]]);
                self.hits.split.fetch_add(1, Ordering::Relaxed);
                for op in interleave {
                    self.stage_op(*op, &mut used_live, &mut used_retired);
                }
                self.commit_and_verify(None);
            }
            PlanStep::UndoRedoMergeMintNew {
                relation,
                via_birth,
                interleave,
            } => {
                let subj = if *relation {
                    Subject::Relation
                } else {
                    Subject::Entity
                };
                let (h1, h2) = (
                    self.world.create(subj, "m_a"),
                    self.world.create(subj, "m_b"),
                );
                self.commit_and_verify(None);
                let (n1, n2) = (self.world.name_of(&h1), self.world.name_of(&h2));
                let merged_h = self.world.merge(&[n1, n2], Keep::MintNew);
                self.commit_and_verify(None);
                let merged = self.world.name_of(&merged_h);
                self.world.retire(merged);
                self.commit_and_verify(None);
                let mut used_live = BTreeSet::new();
                let mut used_retired = BTreeSet::from([n1, n2, merged]);
                self.world.restore(n1, *via_birth);
                self.world.restore(n2, *via_birth);
                for op in interleave {
                    self.stage_op(*op, &mut used_live, &mut used_retired);
                }
                self.commit_and_verify(None);
                let mut used_live = BTreeSet::from([n1, n2]);
                let mut used_retired = BTreeSet::from([merged]);
                self.world
                    .remerge_mint_new_replaying_retired(&[n1, n2], merged);
                self.hits.merge.fetch_add(1, Ordering::Relaxed);
                for op in interleave {
                    self.stage_op(*op, &mut used_live, &mut used_retired);
                }
                self.commit_and_verify(None);
            }
        }
    }
}
