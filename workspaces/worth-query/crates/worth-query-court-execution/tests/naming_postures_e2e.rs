//! End-to-end Slice 1B court for `ExecutionNameBirthPlanBuilder::add_split_posture`
//! and `add_merge_posture` (`KeepParent` and `MintNew`) plus fail-closed
//! `LineagePostureMismatch` refusal with unchanged head and naming basis.

use worth_execution::ExecutionRequest;
use worth_query_execution_naming::facade::{
    ExecutionNameBirthPlanBuilder, ExecutionNamedMergeKeeper, ExecutionNamingCommitAuthority,
    ExecutionNamingDenial, ExecutionPendingEntityBirth, ExecutionSplitChildSpec,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::{
    LineageEventKind, LineageNameDenial, LineagePostureKind, LineageRecordTarget,
    PendingBirthTarget, PendingMergeTarget, PersistentNameHolder,
};
use worth_relational::facade::transactions::{
    DeleteEntityIntent, EntityMutationIntent, MutationIntent,
};

use crate::naming_execution_fixture::{
    begin_staged_tx, create_face_intent, execution_lease_for_workers, face_ref, fresh_cad_runtime,
    lowered_cad_catalog, release_commit_snapshot, CadNamingSchema, Face, FaceSplitDisc,
    FACE_SCHEME_ID,
};

#[test]
fn end_to_end_split_and_merge_postures_commit_and_refuse_posture_mismatch() {
    let catalog = lowered_cad_catalog();
    let runtime = fresh_cad_runtime();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let main = BranchId("main".to_owned());

    let head = || {
        runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id()
    };

    // 1. Seed root face F1.
    let f1_ref = face_ref("f1");
    let tx1 = begin_staged_tx(&runtime, "seed-f1", vec![create_face_intent("f1")]);
    let mut b1 = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b1.add_entity_birth(ExecutionPendingEntityBirth::new(
        f1_ref.clone(),
        FACE_SCHEME_ID,
        "occ.seed.f1",
        "authored",
    ))
    .unwrap();
    let (res1, names1) = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx1,
        b1.build(),
    )
    .expect("seed F1 succeeds")
    .into_parts();
    let (f1_id, _) = names1
        .created_entity_name::<CadNamingSchema, Face>(&res1, &f1_ref)
        .unwrap();
    let f1_rel = names1.entity_relational_identity(f1_id).unwrap();
    release_commit_snapshot(&runtime, &res1);

    // 2. Fail-closed: Split without deleting parent refuses LineagePostureMismatch.
    let head_before = head();
    let basis_before = runtime.branch_live_persistent_names(&main);
    let tx_bad = begin_staged_tx(&runtime, "bad-split", vec![create_face_intent("bad-child")]);
    let mut b_bad = ExecutionNameBirthPlanBuilder::new(&catalog, [f1_rel]);
    b_bad
        .add_split_posture(
            LineageRecordTarget::Entity(f1_id),
            f1_rel,
            vec![ExecutionSplitChildSpec::entity(
                face_ref("bad-child"),
                FACE_SCHEME_ID,
                "occ.bad.split",
                "cut_piece",
            )
            .with_discriminator(&FaceSplitDisc::Side { side: 1 })],
        )
        .unwrap();
    let err = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_bad,
        b_bad.build(),
    )
    .expect_err("Split without deleting parent must refuse");
    assert!(matches!(
        err,
        ExecutionNamingDenial::RelationalNaming(LineageNameDenial::LineagePostureMismatch {
            posture: LineagePostureKind::Split,
            ..
        })
    ));
    assert_eq!(head(), head_before);
    assert_eq!(runtime.branch_live_persistent_names(&main), basis_before);

    // 3. Valid Split posture: F1 -> (F1a, F1b).
    let f1a_ref = face_ref("f1a");
    let f1b_ref = face_ref("f1b");
    let tx_split = begin_staged_tx(
        &runtime,
        "split-f1",
        vec![
            MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: f1_id,
            })),
            create_face_intent("f1a"),
            create_face_intent("f1b"),
        ],
    );
    let mut b_split = ExecutionNameBirthPlanBuilder::new(&catalog, [f1_rel]);
    b_split
        .add_split_posture(
            LineageRecordTarget::Entity(f1_id),
            f1_rel,
            vec![
                ExecutionSplitChildSpec::entity(f1a_ref.clone(), FACE_SCHEME_ID, "occ.cut.1", "cut_piece")
                    .with_discriminator(&FaceSplitDisc::Side { side: 1 }),
                ExecutionSplitChildSpec::entity(f1b_ref.clone(), FACE_SCHEME_ID, "occ.cut.1", "cut_piece")
                    .with_discriminator(&FaceSplitDisc::Side { side: 2 }),
            ],
        )
        .unwrap();
    let (res_split, names_split) = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_split,
        b_split.build(),
    )
    .expect("valid Split succeeds")
    .into_parts();
    assert!(res_split
        .envelope()
        .lineage_events()
        .iter()
        .any(|e| e.kind() == LineageEventKind::Split));
    let (f1a_id, f1a_name) = names_split
        .created_entity_name::<CadNamingSchema, Face>(&res_split, &f1a_ref)
        .unwrap();
    let (f1b_id, f1b_name) = names_split
        .created_entity_name::<CadNamingSchema, Face>(&res_split, &f1b_ref)
        .unwrap();
    assert_ne!(f1a_name, f1b_name);
    let f1a_rel = names_split.entity_relational_identity(f1a_id).unwrap();
    let f1b_rel = names_split.entity_relational_identity(f1b_id).unwrap();
    release_commit_snapshot(&runtime, &res_split);

    let live_after_split = runtime.branch_live_persistent_names(&main);
    assert!(!live_after_split.contains_key(&f1_rel));
    assert_eq!(live_after_split.get(&f1a_rel), Some(&PersistentNameHolder::Entity(f1a_id)));
    assert_eq!(live_after_split.get(&f1b_rel), Some(&PersistentNameHolder::Entity(f1b_id)));

    // 4. Valid Merge(KeepParent) in-place: (F1a, F1b) -> keeps F1a, retires F1b.
    let tx_keep = begin_staged_tx(
        &runtime,
        "merge-keep",
        vec![MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: f1b_id },
        ))],
    );
    let mut b_keep = ExecutionNameBirthPlanBuilder::new(&catalog, [f1a_rel, f1b_rel]);
    b_keep
        .add_merge_posture(
            vec![
                (LineageRecordTarget::Entity(f1a_id), f1a_rel),
                (LineageRecordTarget::Entity(f1b_id), f1b_rel),
            ],
            PendingMergeTarget::Existing(LineageRecordTarget::Entity(f1a_id)),
            ExecutionNamedMergeKeeper::KeepParent(f1a_rel),
        )
        .unwrap();
    let (res_keep, _) = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_keep,
        b_keep.build(),
    )
    .expect("Merge(KeepParent) succeeds")
    .into_parts();
    assert!(res_keep
        .envelope()
        .lineage_events()
        .iter()
        .any(|e| e.kind() == LineageEventKind::Merge));
    release_commit_snapshot(&runtime, &res_keep);
    let live_after_keep = runtime.branch_live_persistent_names(&main);
    assert!(live_after_keep.contains_key(&f1a_rel));
    assert!(!live_after_keep.contains_key(&f1b_rel));

    // 5. Seed F2, then Merge(MintNew): (F1a, F2) -> new F3.
    let f2_ref = face_ref("f2");
    let tx_f2 = begin_staged_tx(&runtime, "seed-f2", vec![create_face_intent("f2")]);
    let mut b_f2 = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b_f2.add_entity_birth(ExecutionPendingEntityBirth::new(
        f2_ref.clone(),
        FACE_SCHEME_ID,
        "occ.seed.f2",
        "authored",
    ))
    .unwrap();
    let (res_f2, names_f2) = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_f2,
        b_f2.build(),
    )
    .expect("seed F2 succeeds")
    .into_parts();
    let (f2_id, _) = names_f2
        .created_entity_name::<CadNamingSchema, Face>(&res_f2, &f2_ref)
        .unwrap();
    let f2_rel = names_f2.entity_relational_identity(f2_id).unwrap();
    release_commit_snapshot(&runtime, &res_f2);

    let f3_ref = face_ref("f3-merged");
    let tx_mint = begin_staged_tx(
        &runtime,
        "merge-mint",
        vec![
            MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: f1a_id,
            })),
            MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: f2_id,
            })),
            create_face_intent("f3-merged"),
        ],
    );
    let mut b_mint = ExecutionNameBirthPlanBuilder::new(&catalog, [f1a_rel, f2_rel]);
    b_mint
        .add_merge_posture(
            vec![
                (LineageRecordTarget::Entity(f1a_id), f1a_rel),
                (LineageRecordTarget::Entity(f2_id), f2_rel),
            ],
            PendingMergeTarget::Created(PendingBirthTarget::CreatedEntity(f3_ref.clone())),
            ExecutionNamedMergeKeeper::mint_new(
                FACE_SCHEME_ID,
                "occ.merge.new",
                "cut_piece",
                &FaceSplitDisc::Side { side: 9 },
            ),
        )
        .unwrap();
    let (res_mint, names_mint) = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_mint,
        b_mint.build(),
    )
    .expect("Merge(MintNew) succeeds")
    .into_parts();
    assert!(res_mint
        .envelope()
        .lineage_events()
        .iter()
        .any(|e| e.kind() == LineageEventKind::Merge));
    let (f3_id, f3_name) = names_mint
        .created_entity_name::<CadNamingSchema, Face>(&res_mint, &f3_ref)
        .unwrap();
    assert_ne!(f3_name, f1a_name);
    let mut expected_parents = vec![f1a_rel, f2_rel];
    expected_parents.sort_unstable();
    assert_eq!(
        names_mint
            .entity_birth(f3_id)
            .unwrap()
            .parts()
            .parent_identities(),
        expected_parents.as_slice()
    );
    release_commit_snapshot(&runtime, &res_mint);
}
