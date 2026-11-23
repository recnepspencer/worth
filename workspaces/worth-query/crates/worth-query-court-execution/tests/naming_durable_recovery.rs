//! End-to-end Slice 1B durable recovery courts proving persistent naming
//! survives BOTH checkpoint restore and checkpoint + journal tail replay across
//! wire-serialized canonical commit artifacts (entities, relations, and `Split`).

use worth_execution::ExecutionRequest;
use worth_query_execution_naming::facade::{
    ExecutionNameBirthPlanBuilder, ExecutionNamingCommitAuthority, ExecutionNamingDenial,
    ExecutionPendingEntityBirth, ExecutionPendingRelationBirth, ExecutionSplitChildSpec,
};
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalCanonicalCommitArtifact,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::lineage::{
    LineageNameDenial, LineageRecordTarget, PersistentNameHolder, PersistentNameIdentity,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::{
    DeleteEntityIntent, EntityMutationIntent, EntityReference, MutationIntent,
};

use crate::naming_execution_fixture::{
    begin_staged_tx, create_face_intent, create_seam_intent, execution_lease_for_workers, face_ref,
    fresh_cad_runtime, lowered_cad_catalog, release_commit_snapshot, seam_ref, CadNamingSchema,
    Face, FaceSplitDisc, Seam, SeamSplitDisc, FACE_SCHEME_ID, SEAM_SCHEME_ID,
};

fn assert_dup_relation_refuses(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    src: EntityId,
    dst: EntityId,
    occ: &str,
    disc: &SeamSplitDisc,
    basis_ids: &[PersistentNameIdentity],
    expected_id: PersistentNameIdentity,
    expected_rel: RelationId,
) {
    let catalog = lowered_cad_catalog();
    let dup_ref = seam_ref("dup-s", EntityReference::Existing(src), EntityReference::Existing(dst));
    let tx = begin_staged_tx(
        runtime,
        "dup-s",
        vec![create_seam_intent("dup-s", EntityReference::Existing(src), EntityReference::Existing(dst))],
    );
    let mut b = ExecutionNameBirthPlanBuilder::new(&catalog, basis_ids.iter().copied());
    b.add_relation_birth(
        ExecutionPendingRelationBirth::new(dup_ref, SEAM_SCHEME_ID, occ, "seam")
            .with_discriminator(disc),
    )
    .unwrap();
    let err = ExecutionNamingCommitAuthority::commit_with_naming_plan(runtime, execution, tx, b.build())
        .expect_err("post-recovery relation DuplicateBirth must refuse");
    assert!(
        matches!(
            err,
            ExecutionNamingDenial::RelationalNaming(LineageNameDenial::DuplicateBirth {
                identity,
                first: PersistentNameHolder::Relation(first_id),
                ..
            }) if identity == expected_id && first_id == expected_rel
        ),
        "unexpected post-recovery refusal: {err:?}"
    );
}

#[test]
fn durable_checkpoint_restore_preserves_entity_and_relation_naming_basis() {
    let catalog = lowered_cad_catalog();
    let source = fresh_cad_runtime();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let main = BranchId("main".to_owned());

    let f1_ref = face_ref("f1");
    let f2_ref = face_ref("f2");
    let s1_ref = seam_ref("s1", EntityReference::Created(f1_ref.clone()), EntityReference::Created(f2_ref.clone()));
    let tx1 = begin_staged_tx(
        &source,
        "seed",
        vec![
            create_face_intent("f1"),
            create_face_intent("f2"),
            create_seam_intent("s1", EntityReference::Created(f1_ref.clone()), EntityReference::Created(f2_ref.clone())),
        ],
    );
    let mut b1 = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b1.add_entity_birth(ExecutionPendingEntityBirth::new(f1_ref.clone(), FACE_SCHEME_ID, "occ.ckpt.f1", "authored"))
        .unwrap()
        .add_entity_birth(ExecutionPendingEntityBirth::new(f2_ref.clone(), FACE_SCHEME_ID, "occ.ckpt.f2", "authored"))
        .unwrap()
        .add_relation_birth(
            ExecutionPendingRelationBirth::new(s1_ref.clone(), SEAM_SCHEME_ID, "occ.ckpt.s1", "seam")
                .with_discriminator(&SeamSplitDisc::Ordinal { index: 1 }),
        )
        .unwrap();
    let (res1, names1) = ExecutionNamingCommitAuthority::commit_with_naming_plan(&source, &execution, tx1, b1.build())
        .expect("seed commit succeeds")
        .into_parts();
    let (f1_id, _) = names1.created_entity_name::<CadNamingSchema, Face>(&res1, &f1_ref).unwrap();
    let (f2_id, _) = names1.created_entity_name::<CadNamingSchema, Face>(&res1, &f2_ref).unwrap();
    let (s1_id, _) = names1.created_relation_name::<Seam>(&res1, &s1_ref).unwrap();
    let f1_rel = names1.entity_relational_identity(f1_id).unwrap();
    let s1_rel = names1.relation_relational_identity(s1_id).unwrap();
    release_commit_snapshot(&source, &res1);

    let expected_basis = source.branch_live_persistent_names(&main);
    let expected_s1_lin = source.relation_lineage(s1_id).expect("seam lineage exists");

    source.durability_authority().checkpoint(&execution).expect("checkpoint succeeds");
    let plan = source
        .durability()
        .recovery_plan(&execution, RecoveryVerificationMode::NormalRecoveryVerification)
        .expect("recovery plan builds");
    assert!(plan.checkpoint.is_some());
    assert!(plan.checkpoint_covered_position().is_some());

    let mut reopened = fresh_cad_runtime();
    let outcome = reopened.durability_recovery().recover(&execution, plan).expect("checkpoint restore succeeds");
    assert_eq!(outcome.recovered_commits, 1);
    assert_eq!(reopened.branch_live_persistent_names(&main), expected_basis);
    assert_eq!(reopened.relation_lineage(s1_id), Some(expected_s1_lin));

    let head0 = reopened.admit_branch_basis(&reopened.main_branch_identity()).unwrap().observation().commit_id();

    let tx_dup_f = begin_staged_tx(&reopened, "dup-f", vec![create_face_intent("dup-f")]);
    let mut b_dup_f = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b_dup_f.add_entity_birth(ExecutionPendingEntityBirth::new(face_ref("dup-f"), FACE_SCHEME_ID, "occ.ckpt.f1", "authored")).unwrap();
    let err_f = ExecutionNamingCommitAuthority::commit_with_naming_plan(&reopened, &execution, tx_dup_f, b_dup_f.build())
        .expect_err("post-checkpoint entity DuplicateBirth must refuse");
    assert!(matches!(
        err_f,
        ExecutionNamingDenial::RelationalNaming(LineageNameDenial::DuplicateBirth {
            identity,
            first: PersistentNameHolder::Entity(first_id),
            ..
        }) if identity == f1_rel && first_id == f1_id
    ));
    assert_dup_relation_refuses(
        &reopened,
        &execution,
        f2_id,
        f1_id,
        "occ.ckpt.s1",
        &SeamSplitDisc::Ordinal { index: 1 },
        &[],
        s1_rel,
        s1_id,
    );
    assert_eq!(reopened.admit_branch_basis(&reopened.main_branch_identity()).unwrap().observation().commit_id(), head0);
    assert_eq!(reopened.branch_live_persistent_names(&main), expected_basis);
}

#[test]
fn durable_journal_tail_replay_reconstructs_entity_relation_and_split_naming_basis() {
    let catalog = lowered_cad_catalog();
    let source = fresh_cad_runtime();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let main = BranchId("main".to_owned());

    // Commit 1 (covered by checkpoint): seed F1.
    let f1_ref = face_ref("f1");
    let tx1 = begin_staged_tx(&source, "c1", vec![create_face_intent("f1")]);
    let mut b1 = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b1.add_entity_birth(ExecutionPendingEntityBirth::new(f1_ref.clone(), FACE_SCHEME_ID, "occ.tail.f1", "authored")).unwrap();
    let (res1, names1) = ExecutionNamingCommitAuthority::commit_with_naming_plan(&source, &execution, tx1, b1.build())
        .expect("c1 succeeds")
        .into_parts();
    let (f1_id, _) = names1.created_entity_name::<CadNamingSchema, Face>(&res1, &f1_ref).unwrap();
    let f1_rel = names1.entity_relational_identity(f1_id).unwrap();
    release_commit_snapshot(&source, &res1);

    // Checkpoint AFTER commit 1 so commits 2 and 3 live strictly in the journal tail.
    source.durability_authority().checkpoint(&execution).expect("checkpoint at c1 succeeds");

    // Commit 2 (journal tail): split F1 into F1a and F1b.
    let f1a_ref = face_ref("f1a");
    let f1b_ref = face_ref("f1b");
    let tx2 = begin_staged_tx(
        &source,
        "c2-split",
        vec![
            MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: f1_id })),
            create_face_intent("f1a"),
            create_face_intent("f1b"),
        ],
    );
    let mut b2 = ExecutionNameBirthPlanBuilder::new(&catalog, [f1_rel]);
    b2.add_split_posture(
        LineageRecordTarget::Entity(f1_id),
        f1_rel,
        vec![
            ExecutionSplitChildSpec::entity(f1a_ref.clone(), FACE_SCHEME_ID, "occ.tail.split", "cut_piece")
                .with_discriminator(&FaceSplitDisc::Side { side: 1 }),
            ExecutionSplitChildSpec::entity(f1b_ref.clone(), FACE_SCHEME_ID, "occ.tail.split", "cut_piece")
                .with_discriminator(&FaceSplitDisc::Side { side: 2 }),
        ],
    )
    .unwrap();
    let (res2, names2) = ExecutionNamingCommitAuthority::commit_with_naming_plan(&source, &execution, tx2, b2.build())
        .expect("tail split commit succeeds")
        .into_parts();
    let (f1a_id, f1a_name) = names2.created_entity_name::<CadNamingSchema, Face>(&res2, &f1a_ref).unwrap();
    let (f1b_id, _) = names2.created_entity_name::<CadNamingSchema, Face>(&res2, &f1b_ref).unwrap();
    let f1a_rel = names2.entity_relational_identity(f1a_id).unwrap();
    release_commit_snapshot(&source, &res2);

    // Commit 3 (journal tail): create named relation S1 (F1a -> F1b) referencing f1a_name.
    let s1_ref = seam_ref("s1", EntityReference::Existing(f1a_id), EntityReference::Existing(f1b_id));
    let tx3 = begin_staged_tx(
        &source,
        "c3-seam",
        vec![create_seam_intent("s1", EntityReference::Existing(f1a_id), EntityReference::Existing(f1b_id))],
    );
    let mut b3 = ExecutionNameBirthPlanBuilder::new(&catalog, [f1a_rel]);
    b3.add_relation_birth(
        ExecutionPendingRelationBirth::new(s1_ref.clone(), SEAM_SCHEME_ID, "occ.tail.seam", "seam")
            .with_discriminator(&SeamSplitDisc::ByFace { face: f1a_name, index: 7 }),
    )
    .unwrap();
    let (res3, names3) = ExecutionNamingCommitAuthority::commit_with_naming_plan(&source, &execution, tx3, b3.build())
        .expect("tail relation commit succeeds")
        .into_parts();
    let (s1_id, _) = names3.created_relation_name::<Seam>(&res3, &s1_ref).unwrap();
    let s1_rel = names3.relation_relational_identity(s1_id).unwrap();
    release_commit_snapshot(&source, &res3);

    let expected_basis = source.branch_live_persistent_names(&main);
    let expected_s1_lin = source.relation_lineage(s1_id).expect("seam lineage exists");

    // Round-trip every tail entry through wire serialization + readmit (verifies persisted digests).
    let plan = source
        .durability()
        .recovery_plan(&execution, RecoveryVerificationMode::NormalRecoveryVerification)
        .expect("tail recovery plan builds");
    assert!(plan.checkpoint.is_some());
    let covered = plan
        .checkpoint_covered_position()
        .expect("checkpoint covers commit 1");
    let wire_tail: Vec<Vec<u8>> = source
        .durability()
        .canonical_commit_artifacts(&execution)
        .after_position(covered)
        .map(|res| res.expect("export tail artifact").as_bytes().to_vec())
        .collect();
    assert_eq!(wire_tail.len(), 2);
    let readmitted: Vec<_> = wire_tail
        .iter()
        .map(|bytes| {
            RelationalCanonicalCommitArtifact::readmit(bytes, &execution)
                .expect("readmit tail artifact with verified persisted births")
        })
        .collect();
    assert!(readmitted[0]
        .envelope()
        .lineage_events()
        .iter()
        .any(|ev| ev.name_birth().is_some()));
    assert!(readmitted[1]
        .envelope()
        .lineage_events()
        .iter()
        .any(|ev| ev.name_birth().is_some()));

    let mut reopened = fresh_cad_runtime();
    let outcome = reopened.durability_recovery().recover(&execution, plan).expect("checkpoint + tail replay succeeds");
    assert_eq!(outcome.recovered_commits, 3);
    assert_eq!(reopened.branch_live_persistent_names(&main), expected_basis);
    assert_eq!(reopened.relation_lineage(s1_id), Some(expected_s1_lin));

    assert_dup_relation_refuses(
        &reopened,
        &execution,
        f1b_id,
        f1a_id,
        "occ.tail.seam",
        &SeamSplitDisc::ByFace { face: f1a_name, index: 7 },
        &[f1a_rel],
        s1_rel,
        s1_id,
    );
    assert_eq!(reopened.branch_live_persistent_names(&main), expected_basis);
}
