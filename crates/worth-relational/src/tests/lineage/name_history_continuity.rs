//! Phase 5 behavioral courts for 3-way re-minting classification
//! (in-commit regeneration vs cross-commit restore vs live conflict) and
//! independent-occurrence isolation across Entities and Relations.

use std::collections::BTreeMap;
use worth_relational_contracts::internal_port::lineage::data::artifacts::finalization_artifact::PreparedLineageFinalizationFixturePort as _;
use worth_relational_lineage::internal_port::lineage::index::batch::PersistentNameCommittedBatchFixturePort as _;

use crate::facade::history::CommitId;
use crate::facade::identity::VersionId;
use crate::facade::lineage::{
    LineageDecisionKind, LineageEventKind, LineageNameDenial, LineagePostureClaim,
    LineagePostureKind, LineageRecordTarget, LineageSubject, PersistentNameBranchIndex,
    PersistentNameCommittedBatch, PersistentNameHolder, PersistentNameKey,
    PersistentNameResolution, PersistentNameTarget,
};
use crate::facade::transactions::RecordRef;
use crate::lineage::authority::finalize_postures_in_working_state_for_test;
use crate::lineage::authority::posture_fixture::LineageCommitPostureInput;
use crate::runtime::LineageIdentityAllocator;

use super::name_index_fixtures::{exact, main_branch, resolve, retired};
use super::posture_fixtures::{
    cad_scheme_registry, empty_working_state, face_birth, push_entity, push_relation, receipt_at,
    retire_entity, retire_relation, seam_birth, staged_entity_lineage,
};

#[test]
fn regeneration_vs_restore_vs_live_conflict() {
    let allocator = LineageIdentityAllocator::new();
    let registry = cad_scheme_registry();
    let mut staged = empty_working_state();

    // Commit 1: birth one entity and one relation.
    let e1 = push_entity(&mut staged, VersionId(1));
    let anchor = push_entity(&mut staged, VersionId(1));
    let r1 = push_relation(&mut staged, VersionId(1), e1, anchor);
    let e_birth = face_birth("occ.cls", "authored", vec![], "front", e1);
    let r_birth = seam_birth("occ.cls", "seam", vec![], "edge", r1);
    let c1 = finalize_postures_in_working_state_for_test(
        &allocator,
        &mut staged,
        &receipt_at(1, 1),
        &[],
        &[
            RecordRef::Entity(e1),
            RecordRef::Entity(anchor),
            RecordRef::Relation(r1),
        ],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            postures: vec![
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Entity(e1),
                    birth: e_birth,
                },
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Relation(r1),
                    birth: r_birth,
                },
            ],
            ..Default::default()
        },
    )
    .expect("initial births succeed");
    let e1_lin = staged_entity_lineage(&staged, e1).unwrap();
    let r1_lin = c1.relation_lineages_for_test()[&r1];
    let name_for = |lin| {
        c1.artifact()
            .event_batch()
            .events()
            .iter()
            .find(|e| e.targets() == [lin])
            .unwrap()
            .name_birth()
            .unwrap()
            .identity()
    };
    let (e_name, r_name) = (name_for(e1_lin), name_for(r1_lin));
    let live_holders = BTreeMap::from([
        (e_name, PersistentNameHolder::Entity(e1)),
        (r_name, PersistentNameHolder::Relation(r1)),
    ]);
    let live_lineages = BTreeMap::from([
        (PersistentNameHolder::Entity(e1), e1_lin),
        (PersistentNameHolder::Relation(r1), r1_lin),
    ]);

    // Case 3 (live conflict): re-minting live entity, live relation, or explicit Restore of live name refuses DuplicateBirth with zero allocator advance.
    let e_dup = push_entity(&mut staged, VersionId(2));
    let r_dup = push_relation(&mut staged, VersionId(2), e1, anchor);
    let before = allocator.frontiers();
    for (changed, retired_lineages, posture, expected_id) in [
        (
            vec![RecordRef::Entity(e_dup)],
            BTreeMap::new(),
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Entity(e_dup),
                birth: face_birth("occ.cls", "authored", vec![], "front", e_dup),
            },
            e_name,
        ),
        (
            vec![RecordRef::Relation(r_dup)],
            BTreeMap::new(),
            LineagePostureClaim::Create {
                target: LineageRecordTarget::Relation(r_dup),
                birth: seam_birth("occ.cls", "seam", vec![], "edge", r_dup),
            },
            r_name,
        ),
        (
            vec![RecordRef::Entity(e_dup)],
            BTreeMap::from([(e_name, (LineageSubject::Entity, e1_lin))]),
            LineagePostureClaim::Restore {
                target: LineageRecordTarget::Entity(e_dup),
                name: e_name,
            },
            e_name,
        ),
    ] {
        let err = finalize_postures_in_working_state_for_test(
            &allocator,
            &mut staged,
            &receipt_at(2, 2),
            &[],
            &changed,
            &LineageCommitPostureInput {
                scheme_registry: registry.clone(),
                live_holders: live_holders.clone(),
                live_lineages: live_lineages.clone(),
                retired_lineages,
                postures: vec![posture],
            },
        )
        .err()
        .expect("live conflict must refuse DuplicateBirth");
        assert!(matches!(
            err.lineage_name_denial_ref(),
            Some(LineageNameDenial::DuplicateBirth { identity, .. }) if *identity == expected_id
        ));
        assert_eq!(allocator.frontiers(), before);
        assert_eq!(staged_entity_lineage(&staged, e_dup), None);
    }

    // Case 1 (in-commit regeneration): retire + re-mint exact identity in SAME commit -> Create(kept_name) + Replace, ZERO Retire.
    retire_entity(&mut staged, e1, VersionId(2));
    retire_relation(&mut staged, r1, VersionId(2));
    let e2 = push_entity(&mut staged, VersionId(2));
    let r2 = push_relation(&mut staged, VersionId(2), e2, anchor);
    let c2 = finalize_postures_in_working_state_for_test(
        &allocator,
        &mut staged,
        &receipt_at(2, 2),
        &[],
        &[
            RecordRef::Entity(e1),
            RecordRef::Entity(e2),
            RecordRef::Relation(r1),
            RecordRef::Relation(r2),
        ],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            live_holders,
            live_lineages,
            retired_lineages: BTreeMap::new(),
            postures: vec![
                LineagePostureClaim::Retire {
                    target: LineageRecordTarget::Entity(e1),
                    name: e_name,
                },
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Entity(e2),
                    birth: face_birth("occ.cls", "authored", vec![], "front", e2),
                },
                LineagePostureClaim::Retire {
                    target: LineageRecordTarget::Relation(r1),
                    name: r_name,
                },
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Relation(r2),
                    birth: seam_birth("occ.cls", "seam", vec![], "edge", r2),
                },
            ],
        },
    )
    .expect("in-commit regeneration emits Preserve/Replace continuity");
    let c2_ev = c2.artifact().event_batch().events();
    assert!(c2_ev.iter().all(|e| e.kind() != LineageEventKind::Retire));
    let e2_lin = staged_entity_lineage(&staged, e2).unwrap();
    let r2_lin = c2.relation_lineages_for_test()[&r2];
    assert_ne!(e2, e1);
    assert_ne!(r2, r1);
    assert!(c2_ev.iter().any(|e| {
        e.kind() == LineageEventKind::Replace
            && e.subject() == LineageSubject::Entity
            && e.sources() == [e1_lin]
            && e.targets() == [e2_lin]
            && e.kept_name() == Some(e_name)
    }));
    assert!(c2_ev.iter().any(|e| {
        e.kind() == LineageEventKind::Replace
            && e.subject() == LineageSubject::Relation
            && e.sources() == [r1_lin]
            && e.targets() == [r2_lin]
            && e.kept_name() == Some(r_name)
    }));

    // Commit 3: Retire both e2 and r2.
    retire_entity(&mut staged, e2, VersionId(3));
    retire_relation(&mut staged, r2, VersionId(3));
    let c3 = finalize_postures_in_working_state_for_test(
        &allocator,
        &mut staged,
        &receipt_at(3, 3),
        &[],
        &[RecordRef::Entity(e2), RecordRef::Relation(r2)],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            live_holders: BTreeMap::from([
                (e_name, PersistentNameHolder::Entity(e2)),
                (r_name, PersistentNameHolder::Relation(r2)),
            ]),
            live_lineages: BTreeMap::from([
                (PersistentNameHolder::Entity(e2), e2_lin),
                (PersistentNameHolder::Relation(r2), r2_lin),
            ]),
            retired_lineages: BTreeMap::new(),
            postures: vec![
                LineagePostureClaim::Retire {
                    target: LineageRecordTarget::Entity(e2),
                    name: e_name,
                },
                LineagePostureClaim::Retire {
                    target: LineageRecordTarget::Relation(r2),
                    name: r_name,
                },
            ],
        },
    )
    .expect("retire succeeds");

    // Wrong-subject and duplicate same-commit Restore refuse before ID reservation.
    let e3 = push_entity(&mut staged, VersionId(4));
    let e3_dup = push_entity(&mut staged, VersionId(4));
    let r3 = push_relation(&mut staged, VersionId(4), e3, anchor);
    let retired_basis = BTreeMap::from([
        (e_name, (LineageSubject::Entity, e2_lin)),
        (r_name, (LineageSubject::Relation, r2_lin)),
    ]);
    let before4 = allocator.frontiers();
    let bad_subj = finalize_postures_in_working_state_for_test(
        &allocator,
        &mut staged,
        &receipt_at(4, 4),
        &[],
        &[RecordRef::Entity(e3)],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            retired_lineages: retired_basis.clone(),
            postures: vec![LineagePostureClaim::Restore {
                target: LineageRecordTarget::Entity(e3),
                name: r_name,
            }],
            ..Default::default()
        },
    )
    .err()
    .expect("wrong subject Restore must refuse");
    assert!(matches!(
        bad_subj.lineage_name_denial_ref(),
        Some(LineageNameDenial::LineagePostureMismatch {
            posture: LineagePostureKind::Restore,
            ..
        })
    ));
    let double_restore = finalize_postures_in_working_state_for_test(
        &allocator,
        &mut staged,
        &receipt_at(4, 4),
        &[],
        &[RecordRef::Entity(e3), RecordRef::Entity(e3_dup)],
        &LineageCommitPostureInput {
            scheme_registry: registry.clone(),
            retired_lineages: retired_basis.clone(),
            postures: vec![
                LineagePostureClaim::Restore {
                    target: LineageRecordTarget::Entity(e3),
                    name: e_name,
                },
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Entity(e3_dup),
                    birth: face_birth("occ.cls", "authored", vec![], "front", e3_dup),
                },
            ],
            ..Default::default()
        },
    )
    .err()
    .expect("restoring same retired name twice in one commit must refuse");
    assert!(matches!(
        double_restore.lineage_name_denial_ref(),
        Some(LineageNameDenial::DuplicateBirth { identity, .. }) if *identity == e_name
    ));
    assert_eq!(allocator.frontiers(), before4);

    // Case 2 (cross-commit restore): explicit Restore for entity + birth-classified Restore for relation.
    let c4 = finalize_postures_in_working_state_for_test(
        &allocator,
        &mut staged,
        &receipt_at(4, 4),
        &[],
        &[RecordRef::Entity(e3), RecordRef::Relation(r3)],
        &LineageCommitPostureInput {
            scheme_registry: registry,
            retired_lineages: retired_basis,
            postures: vec![
                LineagePostureClaim::Restore {
                    target: LineageRecordTarget::Entity(e3),
                    name: e_name,
                },
                LineagePostureClaim::Create {
                    target: LineageRecordTarget::Relation(r3),
                    birth: seam_birth("occ.cls", "seam", vec![], "edge", r3),
                },
            ],
            ..Default::default()
        },
    )
    .expect("cross-commit explicit and birth-classified Restore succeed");
    let e3_lin = staged_entity_lineage(&staged, e3).unwrap();
    let r3_lin = c4.relation_lineages_for_test()[&r3];
    let c4_ev = c4.artifact().event_batch().events();
    assert_eq!(c4_ev.len(), 2);
    assert!(c4_ev
        .iter()
        .all(|e| e.kind() == LineageEventKind::Restore && e.name_birth().is_none()));
    assert!(c4
        .artifact()
        .decision_log()
        .decisions()
        .iter()
        .all(|d| d.kind() == &LineageDecisionKind::RestoreAccepted));

    // Derived index agrees at every historical and restored basis.
    let mut index = PersistentNameBranchIndex::new(main_branch(), None);
    for (commit, prepared) in [(1, &c1), (2, &c2), (3, &c3), (4, &c4)] {
        let batch = PersistentNameCommittedBatch::new(CommitId(commit))
            .with_prepared_lineage_for_test(prepared);
        index.apply(&batch).expect("canonical commit ingests");
    }
    let (ek, rk) = (
        PersistentNameKey::Entity(e_name),
        PersistentNameKey::Relation(r_name),
    );
    let exact_entity = |entity, lineage| {
        Ok(PersistentNameResolution::Exact(
            PersistentNameTarget::Entity {
                name: e_name,
                entity,
                lineage,
            },
        ))
    };
    let exact_relation = |relation, lineage| {
        Ok(PersistentNameResolution::Exact(
            PersistentNameTarget::Relation {
                name: r_name,
                relation,
                lineage,
            },
        ))
    };
    assert_eq!(resolve(&index, ek, 1, &exact()), exact_entity(e1, e1_lin));
    assert_eq!(resolve(&index, ek, 2, &exact()), exact_entity(e2, e2_lin));
    assert_eq!(resolve(&index, ek, 3, &exact()), retired(3));
    assert_eq!(resolve(&index, ek, 4, &exact()), exact_entity(e3, e3_lin));
    assert_eq!(resolve(&index, rk, 1, &exact()), exact_relation(r1, r1_lin));
    assert_eq!(resolve(&index, rk, 2, &exact()), exact_relation(r2, r2_lin));
    assert_eq!(resolve(&index, rk, 3, &exact()), retired(3));
    assert_eq!(resolve(&index, rk, 4, &exact()), exact_relation(r3, r3_lin));
}
