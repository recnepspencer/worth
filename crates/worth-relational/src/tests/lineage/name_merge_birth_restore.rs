//! Phase 5B2 court proving birth-classified `Restore` for `Merge(MintNew)`
//! when re-minting a previously retired merged name: exact emitted events
//! (zero duplicate `Create`), exact lineage reservation width, index rebuild
//! equivalence, and oracle-verified historical resolution.

use std::collections::BTreeMap;

use crate::facade::history::CommitId;
use crate::facade::identity::VersionId;
use crate::facade::lineage::{
    LineageCommitPostureInput, LineageDecisionKind, LineageEventKind, LineageMergeKeeper,
    LineagePostureClaim, LineageRecordTarget, LineageSubject, PersistentNameBranchIndex,
    PersistentNameCommittedBatch, PersistentNameHolder, PersistentNameKey,
};
use crate::facade::transactions::RecordRef;
use crate::lineage::authority::finalize_postures_in_working_state;
use crate::runtime::LineageIdentityAllocator;

use super::name_index_fixtures::{all, exact, main_branch, resolve, retired, side, unknown};
use super::name_split_birth_restore::{assert_oracle_agrees, exact_entity};
use super::posture_fixtures::{
    cad_scheme_registry, empty_working_state, face_birth, push_entity, receipt_at, retire_entity,
    staged_entity_lineage,
};

#[test]
fn remerge_mint_new_replays_retired_merged_name_as_birth_restore() {
    let allocator = LineageIdentityAllocator::new();
    let registry = cad_scheme_registry();
    let mut staged = empty_working_state();
    let mut index = PersistentNameBranchIndex::new(main_branch(), None);
    let mut log = Vec::new();
    let mut record = |batch: PersistentNameCommittedBatch| {
        index.apply(&batch).expect("canonical batch ingests");
        log.push(batch);
    };

    // Commit 1: Create A and B.
    let a1 = push_entity(&mut staged, VersionId(1));
    let b1 = push_entity(&mut staged, VersionId(1));
    let c1 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(1, 1),
        &[],
        &[RecordRef::Entity(a1), RecordRef::Entity(b1)],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            postures: vec![
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Entity(a1),
                    birth: face_birth("occ.ab", "authored", vec![], "a", a1),
                },
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Entity(b1),
                    birth: face_birth("occ.ab", "authored", vec![], "b", b1),
                },
            ],
            ..Default::default()
        },
    )
    .expect("commit 1 succeeds");
    let (a1_lin, b1_lin) = (
        staged_entity_lineage(&staged, a1).unwrap(),
        staged_entity_lineage(&staged, b1).unwrap(),
    );
    let name_for = |c: &crate::lineage::data::PreparedLineageFinalization, lin| {
        c.artifact()
            .event_batch()
            .events()
            .iter()
            .find(|e| e.kind() == LineageEventKind::Create && e.targets() == [lin])
            .unwrap()
            .name_birth()
            .unwrap()
            .identity()
    };
    let (a_name, b_name) = (name_for(&c1, a1_lin), name_for(&c1, b1_lin));
    record(PersistentNameCommittedBatch::new(CommitId(1)).with_prepared_lineage(&c1));

    // Commit 2: Merge(MintNew) A + B -> M.
    retire_entity(&mut staged, a1, VersionId(2));
    retire_entity(&mut staged, b1, VersionId(2));
    let m1 = push_entity(&mut staged, VersionId(2));
    let c2 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(2, 2),
        &[],
        &[
            RecordRef::Entity(a1),
            RecordRef::Entity(b1),
            RecordRef::Entity(m1),
        ],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            live_holders: BTreeMap::from([
                (a_name, PersistentNameHolder::Entity(a1)),
                (b_name, PersistentNameHolder::Entity(b1)),
            ]),
            live_lineages: BTreeMap::from([
                (PersistentNameHolder::Entity(a1), a1_lin),
                (PersistentNameHolder::Entity(b1), b1_lin),
            ]),
            postures: vec![LineagePostureClaim::Merge {
                parents: vec![
                    (LineageRecordTarget::Entity(a1), a_name),
                    (LineageRecordTarget::Entity(b1), b_name),
                ],
                target: LineageRecordTarget::Entity(m1),
                keeper: LineageMergeKeeper::MintNew {
                    birth: face_birth(
                        "occ.merge",
                        "merged_face",
                        vec![a_name, b_name],
                        "union",
                        m1,
                    ),
                },
            }],
            ..Default::default()
        },
    )
    .expect("commit 2 merge succeeds");
    let m1_lin = staged_entity_lineage(&staged, m1).unwrap();
    let m_name = name_for(&c2, m1_lin);
    record(PersistentNameCommittedBatch::new(CommitId(2)).with_prepared_lineage(&c2));

    // Commit 3: Retire M.
    retire_entity(&mut staged, m1, VersionId(3));
    let c3 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(3, 3),
        &[],
        &[RecordRef::Entity(m1)],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            live_holders: BTreeMap::from([(m_name, PersistentNameHolder::Entity(m1))]),
            live_lineages: BTreeMap::from([(PersistentNameHolder::Entity(m1), m1_lin)]),
            retired_lineages: BTreeMap::from([
                (a_name, (LineageSubject::Entity, a1_lin)),
                (b_name, (LineageSubject::Entity, b1_lin)),
            ]),
            postures: vec![LineagePostureClaim::Retire {
                target: LineageRecordTarget::Entity(m1),
                name: m_name,
            }],
        },
    )
    .expect("commit 3 retire M succeeds");
    record(PersistentNameCommittedBatch::new(CommitId(3)).with_prepared_lineage(&c3));

    // Commit 4: Restore A and B.
    let a2 = push_entity(&mut staged, VersionId(4));
    let b2 = push_entity(&mut staged, VersionId(4));
    let c4 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(4, 4),
        &[],
        &[RecordRef::Entity(a2), RecordRef::Entity(b2)],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            retired_lineages: BTreeMap::from([
                (a_name, (LineageSubject::Entity, a1_lin)),
                (b_name, (LineageSubject::Entity, b1_lin)),
                (m_name, (LineageSubject::Entity, m1_lin)),
            ]),
            postures: vec![
                LineagePostureClaim::Restore {
                    target: LineageRecordTarget::Entity(a2),
                    name: a_name,
                },
                LineagePostureClaim::Restore {
                    target: LineageRecordTarget::Entity(b2),
                    name: b_name,
                },
            ],
            ..Default::default()
        },
    )
    .expect("commit 4 restore A and B succeeds");
    let (a2_lin, b2_lin) = (
        staged_entity_lineage(&staged, a2).unwrap(),
        staged_entity_lineage(&staged, b2).unwrap(),
    );
    record(PersistentNameCommittedBatch::new(CommitId(4)).with_prepared_lineage(&c4));

    // Commit 5: Merge(MintNew) A + B replaying retired M birth.
    retire_entity(&mut staged, a2, VersionId(5));
    retire_entity(&mut staged, b2, VersionId(5));
    let m2 = push_entity(&mut staged, VersionId(5));
    let (lin_before, _) = allocator.frontiers();
    let c5 = finalize_postures_in_working_state(
        &allocator,
        &mut staged,
        &receipt_at(5, 5),
        &[],
        &[
            RecordRef::Entity(a2),
            RecordRef::Entity(b2),
            RecordRef::Entity(m2),
        ],
        &LineageCommitPostureInput {
            scheme_registry: registry,
            live_holders: BTreeMap::from([
                (a_name, PersistentNameHolder::Entity(a2)),
                (b_name, PersistentNameHolder::Entity(b2)),
            ]),
            live_lineages: BTreeMap::from([
                (PersistentNameHolder::Entity(a2), a2_lin),
                (PersistentNameHolder::Entity(b2), b2_lin),
            ]),
            retired_lineages: BTreeMap::from([(m_name, (LineageSubject::Entity, m1_lin))]),
            postures: vec![LineagePostureClaim::Merge {
                parents: vec![
                    (LineageRecordTarget::Entity(a2), a_name),
                    (LineageRecordTarget::Entity(b2), b_name),
                ],
                target: LineageRecordTarget::Entity(m2),
                keeper: LineageMergeKeeper::MintNew {
                    birth: face_birth(
                        "occ.merge",
                        "merged_face",
                        vec![b_name, a_name],
                        "union",
                        m2,
                    ),
                },
            }],
        },
    )
    .expect("commit 5 re-merge with birth-classified Restore succeeds");
    let (lin_after, _) = allocator.frontiers();
    assert_eq!(
        lin_after - lin_before,
        1,
        "re-merge must reserve exactly 1 LineageId for restored merged target (zero duplicate Create)"
    );
    let m2_lin = staged_entity_lineage(&staged, m2).unwrap();
    let c5_ev = c5.artifact().event_batch().events();
    assert_eq!(
        c5_ev.len(),
        2,
        "expected 1 Restore + 1 Merge, got {c5_ev:?}"
    );
    assert!(c5_ev.iter().all(|e| e.kind() != LineageEventKind::Create));
    assert!(c5_ev.iter().any(|e| {
        e.kind() == LineageEventKind::Restore
            && e.sources() == [m1_lin]
            && e.targets() == [m2_lin]
            && e.kept_name() == Some(m_name)
            && e.name_birth().is_none()
    }));
    let mut merge_sources = vec![a2_lin, b2_lin];
    merge_sources.sort_unstable();
    assert!(c5_ev.iter().any(|e| {
        e.kind() == LineageEventKind::Merge
            && e.sources() == merge_sources.as_slice()
            && e.targets() == [m2_lin]
            && e.kept_name().is_none()
    }));
    let decisions = c5.artifact().decision_log().decisions();
    assert_eq!(
        decisions
            .iter()
            .filter(|d| d.kind() == &LineageDecisionKind::RestoreAccepted)
            .count(),
        1
    );
    assert_eq!(
        decisions
            .iter()
            .filter(|d| d.kind() == &LineageDecisionKind::MergeAccepted)
            .count(),
        1
    );

    record(PersistentNameCommittedBatch::new(CommitId(5)).with_prepared_lineage(&c5));
    let (ak, bk, mk) = (
        PersistentNameKey::Entity(a_name),
        PersistentNameKey::Entity(b_name),
        PersistentNameKey::Entity(m_name),
    );
    assert_eq!(
        resolve(&index, ak, 1, &exact()),
        exact_entity(a_name, a1, a1_lin)
    );
    assert_eq!(resolve(&index, mk, 1, &exact()), unknown(mk));
    assert_eq!(
        resolve(&index, ak, 2, &exact()),
        exact_entity(m_name, m1, m1_lin)
    );
    assert_eq!(resolve(&index, ak, 3, &exact()), retired(3));
    assert_eq!(
        resolve(&index, ak, 4, &exact()),
        exact_entity(a_name, a2, a2_lin)
    );
    assert_eq!(
        resolve(&index, bk, 4, &exact()),
        exact_entity(b_name, b2, b2_lin)
    );
    assert_eq!(resolve(&index, mk, 4, &exact()), retired(3));
    assert_eq!(
        resolve(&index, ak, 5, &exact()),
        exact_entity(m_name, m2, m2_lin)
    );
    assert_eq!(
        resolve(&index, bk, 5, &side("union")),
        exact_entity(m_name, m2, m2_lin)
    );
    assert_eq!(
        resolve(&index, mk, 5, &exact()),
        exact_entity(m_name, m2, m2_lin)
    );
    assert_oracle_agrees(&log, &index, &[exact(), all(), side("union"), side("a")]);
}
