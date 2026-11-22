//! Phase 5B2 court proving birth-classified `Restore` for `Split` children
//! when re-splitting a restored parent with replayed original child births:
//! exact emitted events (zero duplicate `Create`), exact lineage reservation
//! width, index rebuild equivalence, and oracle-verified historical resolution.

use std::collections::BTreeMap;

use crate::facade::history::CommitId;
use crate::facade::identity::{EntityId, LineageId, VersionId};
use crate::facade::lineage::{
    LineageCommitPostureInput, LineageDecisionKind, LineageEventKind, LineagePostureClaim,
    LineageRecordTarget, LineageSplitChild, LineageSubject, PersistentNameBranchIndex,
    PersistentNameCommittedBatch, PersistentNameHolder, PersistentNameIdentity, PersistentNameKey,
    PersistentNameResolution, PersistentNameSelection, PersistentNameTarget,
};
use crate::facade::transactions::RecordRef;
use crate::lineage::authority::finalize_postures_in_working_state;
use crate::runtime::LineageIdentityAllocator;

use super::name_event_fixtures::synthetic_name;
use super::name_index_fixtures::{
    all, bounded, exact, main_branch, plural_keys, rebuild, resolve, retired, side, unknown,
};
use super::name_resolution_oracle::{answer, view_at};
use super::posture_fixtures::{
    cad_scheme_registry, empty_working_state, face_birth, push_entity, receipt_at, retire_entity,
    staged_entity_lineage,
};

pub(super) fn exact_entity(
    name: PersistentNameIdentity,
    entity: EntityId,
    lineage: LineageId,
) -> super::name_index_fixtures::Answer {
    Ok(PersistentNameResolution::Exact(
        PersistentNameTarget::Entity {
            name,
            entity,
            lineage,
        },
    ))
}

pub(super) fn assert_oracle_agrees(
    log: &[PersistentNameCommittedBatch],
    index: &PersistentNameBranchIndex,
    selections: &[PersistentNameSelection],
) {
    assert_eq!(rebuild(log), *index, "index equals full rebuild from log");
    let head = log.last().unwrap().commit().0;
    let keys = index.name_keys();
    for basis in 0..=head + 1 {
        let view = view_at(log, basis);
        for &held in &keys {
            for selection in selections {
                assert_eq!(
                    index.resolve(held, CommitId(basis), selection).result,
                    answer(&view, held, selection),
                    "oracle mismatch for {held:?} at basis {basis} under {selection:?}"
                );
            }
        }
    }
}

#[test]
fn resplit_restored_parent_replays_retired_children_as_birth_restores() {
    let allocator = LineageIdentityAllocator::new();
    let registry = cad_scheme_registry();
    let mut staged = empty_working_state();
    let mut index = PersistentNameBranchIndex::new(main_branch(), None);
    let mut log = Vec::new();
    let mut record = |batch: PersistentNameCommittedBatch| {
        index.apply(&batch).expect("canonical batch ingests");
        log.push(batch);
    };

    // Commit 1: Create parent P.
    let p1 = push_entity(&mut staged, VersionId(1));
    let c1 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(1, 1),
        &[],
        &[RecordRef::Entity(p1)],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            postures: vec![LineagePostureClaim::Create {
                target: LineageRecordTarget::Entity(p1),
                birth: face_birth("occ.p", "authored", vec![], "parent", p1),
            }],
            ..Default::default()
        },
    )
    .expect("commit 1 succeeds");
    let p1_lin = staged_entity_lineage(&staged, p1).unwrap();
    let p_name = c1.artifact().event_batch().events()[0]
        .name_birth()
        .unwrap()
        .identity();
    record(PersistentNameCommittedBatch::new(CommitId(1)).with_prepared_lineage(&c1));

    // Commit 2: Split P into L and R.
    retire_entity(&mut staged, p1, VersionId(2));
    let l1 = push_entity(&mut staged, VersionId(2));
    let r1 = push_entity(&mut staged, VersionId(2));
    let c2 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(2, 2),
        &[],
        &[
            RecordRef::Entity(p1),
            RecordRef::Entity(l1),
            RecordRef::Entity(r1),
        ],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            live_holders: BTreeMap::from([(p_name, PersistentNameHolder::Entity(p1))]),
            live_lineages: BTreeMap::from([(PersistentNameHolder::Entity(p1), p1_lin)]),
            postures: vec![LineagePostureClaim::Split {
                parent_target: LineageRecordTarget::Entity(p1),
                parent_name: p_name,
                children: vec![
                    LineageSplitChild {
                        target: LineageRecordTarget::Entity(l1),
                        birth: face_birth("occ.split", "cut_piece", vec![p_name], "left", l1),
                    },
                    LineageSplitChild {
                        target: LineageRecordTarget::Entity(r1),
                        birth: face_birth("occ.split", "cut_piece", vec![p_name], "right", r1),
                    },
                ],
            }],
            ..Default::default()
        },
    )
    .expect("commit 2 split succeeds");
    let (l1_lin, r1_lin) = (
        staged_entity_lineage(&staged, l1).unwrap(),
        staged_entity_lineage(&staged, r1).unwrap(),
    );
    let name_for = |lin| {
        c2.artifact()
            .event_batch()
            .events()
            .iter()
            .find(|e| e.kind() == LineageEventKind::Create && e.targets() == [lin])
            .unwrap()
            .name_birth()
            .unwrap()
            .identity()
    };
    let (l_name, r_name) = (name_for(l1_lin), name_for(r1_lin));
    let (c_old, c_new) = (synthetic_name(0xd1, 1), synthetic_name(0xd1, 2));
    record(
        PersistentNameCommittedBatch::new(CommitId(2))
            .with_prepared_lineage(&c2)
            .with_carriers(PersistentNameKey::Entity(l_name), [c_old]),
    );

    // Commit 3: Retire L and R.
    retire_entity(&mut staged, l1, VersionId(3));
    retire_entity(&mut staged, r1, VersionId(3));
    let c3 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(3, 3),
        &[],
        &[RecordRef::Entity(l1), RecordRef::Entity(r1)],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            live_holders: BTreeMap::from([
                (l_name, PersistentNameHolder::Entity(l1)),
                (r_name, PersistentNameHolder::Entity(r1)),
            ]),
            live_lineages: BTreeMap::from([
                (PersistentNameHolder::Entity(l1), l1_lin),
                (PersistentNameHolder::Entity(r1), r1_lin),
            ]),
            retired_lineages: BTreeMap::from([(p_name, (LineageSubject::Entity, p1_lin))]),
            postures: vec![
                LineagePostureClaim::Retire {
                    target: LineageRecordTarget::Entity(l1),
                    name: l_name,
                },
                LineagePostureClaim::Retire {
                    target: LineageRecordTarget::Entity(r1),
                    name: r_name,
                },
            ],
        },
    )
    .expect("commit 3 retire succeeds");
    record(PersistentNameCommittedBatch::new(CommitId(3)).with_prepared_lineage(&c3));

    // Commit 4: Restore P (undo split + delete).
    let p2 = push_entity(&mut staged, VersionId(4));
    let c4 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(4, 4),
        &[],
        &[RecordRef::Entity(p2)],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            retired_lineages: BTreeMap::from([
                (p_name, (LineageSubject::Entity, p1_lin)),
                (l_name, (LineageSubject::Entity, l1_lin)),
                (r_name, (LineageSubject::Entity, r1_lin)),
            ]),
            postures: vec![LineagePostureClaim::Restore {
                target: LineageRecordTarget::Entity(p2),
                name: p_name,
            }],
            ..Default::default()
        },
    )
    .expect("commit 4 restore parent succeeds");
    let p2_lin = staged_entity_lineage(&staged, p2).unwrap();
    record(PersistentNameCommittedBatch::new(CommitId(4)).with_prepared_lineage(&c4));

    // Commit 5: Re-split restored P with replayed original L and R births.
    retire_entity(&mut staged, p2, VersionId(5));
    let l2 = push_entity(&mut staged, VersionId(5));
    let r2 = push_entity(&mut staged, VersionId(5));
    let (lin_before, _) = allocator.frontiers();
    let c5 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(5, 5),
        &[],
        &[
            RecordRef::Entity(p2),
            RecordRef::Entity(l2),
            RecordRef::Entity(r2),
        ],
        &LineageCommitPostureInput {
            scheme_registry: registry,
            live_holders: BTreeMap::from([(p_name, PersistentNameHolder::Entity(p2))]),
            live_lineages: BTreeMap::from([(PersistentNameHolder::Entity(p2), p2_lin)]),
            retired_lineages: BTreeMap::from([
                (l_name, (LineageSubject::Entity, l1_lin)),
                (r_name, (LineageSubject::Entity, r1_lin)),
            ]),
            postures: vec![LineagePostureClaim::Split {
                parent_target: LineageRecordTarget::Entity(p2),
                parent_name: p_name,
                children: vec![
                    LineageSplitChild {
                        target: LineageRecordTarget::Entity(l2),
                        birth: face_birth("occ.split", "cut_piece", vec![p_name], "left", l2),
                    },
                    LineageSplitChild {
                        target: LineageRecordTarget::Entity(r2),
                        birth: face_birth("occ.split", "cut_piece", vec![p_name], "right", r2),
                    },
                ],
            }],
        },
    )
    .expect("commit 5 re-split with birth-classified child restores succeeds");
    let (lin_after, _) = allocator.frontiers();
    assert_eq!(
        lin_after - lin_before,
        2,
        "re-split must reserve exactly 1 LineageId per restored child (zero duplicate Create)"
    );
    let (l2_lin, r2_lin) = (
        staged_entity_lineage(&staged, l2).unwrap(),
        staged_entity_lineage(&staged, r2).unwrap(),
    );
    let c5_ev = c5.artifact().event_batch().events();
    assert_eq!(
        c5_ev.len(),
        3,
        "expected 2 Restores + 1 Split, got {c5_ev:?}"
    );
    assert!(c5_ev.iter().all(|e| e.kind() != LineageEventKind::Create));
    assert!(c5_ev.iter().any(|e| {
        e.kind() == LineageEventKind::Restore
            && e.sources() == [l1_lin]
            && e.targets() == [l2_lin]
            && e.kept_name() == Some(l_name)
            && e.name_birth().is_none()
    }));
    assert!(c5_ev.iter().any(|e| {
        e.kind() == LineageEventKind::Restore
            && e.sources() == [r1_lin]
            && e.targets() == [r2_lin]
            && e.kept_name() == Some(r_name)
            && e.name_birth().is_none()
    }));
    let mut split_targets = vec![l2_lin, r2_lin];
    split_targets.sort_unstable();
    assert!(c5_ev.iter().any(|e| {
        e.kind() == LineageEventKind::Split
            && e.sources() == [p2_lin]
            && e.targets() == split_targets.as_slice()
    }));
    let decisions = c5.artifact().decision_log().decisions();
    assert_eq!(
        decisions
            .iter()
            .filter(|d| d.kind() == &LineageDecisionKind::RestoreAccepted)
            .count(),
        2
    );
    assert_eq!(
        decisions
            .iter()
            .filter(|d| d.kind() == &LineageDecisionKind::SplitAccepted)
            .count(),
        1
    );

    record(
        PersistentNameCommittedBatch::new(CommitId(5))
            .with_prepared_lineage(&c5)
            .with_carriers(PersistentNameKey::Entity(l_name), [c_new]),
    );
    let (pk, lk, rk) = (
        PersistentNameKey::Entity(p_name),
        PersistentNameKey::Entity(l_name),
        PersistentNameKey::Entity(r_name),
    );
    assert_eq!(
        resolve(&index, pk, 1, &exact()),
        exact_entity(p_name, p1, p1_lin)
    );
    assert_eq!(resolve(&index, lk, 1, &exact()), unknown(lk));
    assert_eq!(
        resolve(&index, pk, 2, &side("left")),
        exact_entity(l_name, l1, l1_lin)
    );
    assert_eq!(
        resolve(&index, pk, 2, &bounded(&[c_old])),
        exact_entity(l_name, l1, l1_lin)
    );
    assert_eq!(resolve(&index, pk, 3, &exact()), retired(3));
    assert_eq!(resolve(&index, lk, 3, &exact()), retired(3));
    assert_eq!(
        resolve(&index, pk, 4, &exact()),
        exact_entity(p_name, p2, p2_lin)
    );
    assert_eq!(resolve(&index, lk, 4, &exact()), retired(3));
    assert_eq!(plural_keys(&resolve(&index, pk, 5, &all())), {
        let mut keys = vec![lk, rk];
        keys.sort_unstable();
        keys
    });
    assert_eq!(
        resolve(&index, pk, 5, &side("left")),
        exact_entity(l_name, l2, l2_lin)
    );
    assert_eq!(
        resolve(&index, pk, 5, &side("right")),
        exact_entity(r_name, r2, r2_lin)
    );
    assert_eq!(
        resolve(&index, pk, 5, &bounded(&[c_new])),
        exact_entity(l_name, l2, l2_lin)
    );
    assert_eq!(
        resolve(&index, lk, 5, &exact()),
        exact_entity(l_name, l2, l2_lin)
    );
    assert_oracle_agrees(
        &log,
        &index,
        &[
            exact(),
            all(),
            side("left"),
            side("right"),
            bounded(&[c_old]),
            bounded(&[c_new]),
        ],
    );
}
