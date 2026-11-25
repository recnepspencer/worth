//! End-to-end R2-B1 cold-directory (external-member) checkpoint restore courts.
//!
//! Proves that restoring a native external-member checkpoint through
//! `cold_directory::restore`:
//! 1. Keeps record regions cold while eagerly materializing and rebuilding the
//!    self-derived naming basis and non-head relation lineage bindings.
//! 2. Preserves exact `branch_live_persistent_names` and non-head `relation_lineage`.
//! 3. Refuses `DuplicateBirth` for both a pre-checkpoint entity and a non-head
//!    pre-checkpoint relation with zero effects.
//! 4. Replays a journal-tail `Split` of a pre-checkpoint named entity accurately.

use std::collections::BTreeSet;

use worth_execution::ExecutionRequest;
use worth_query_execution_naming::facade::{
    ExecutionNameBirthPlanBuilder, ExecutionNamingCommitAuthority, ExecutionNamingDenial,
    ExecutionPendingEntityBirth, ExecutionPendingRelationBirth, ExecutionSplitChildSpec,
};
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalCanonicalCommitArtifact,
    RelationalCheckpointArtifactPolicy, RelationalDurableCheckpointSource,
    RelationalDurableRecoverySource,
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

use crate::naming_cold_source_fixture::ExternalColdSource;
use crate::naming_execution_fixture::{
    begin_staged_tx, create_face_intent, create_seam_intent, execution_lease_for_workers, face_ref,
    fresh_cad_runtime, lowered_cad_catalog, release_commit_snapshot, seam_ref, CadNamingSchema,
    Face, FaceSplitDisc, Seam, SeamSplitDisc, FACE_SCHEME_ID, SEAM_SCHEME_ID,
};

fn assert_dup_entity_refuses(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    occ: &str,
    role: &str,
    expected_id: PersistentNameIdentity,
    expected_entity: EntityId,
) {
    let catalog = lowered_cad_catalog();
    let tx = begin_staged_tx(runtime, "dup-e", vec![create_face_intent("dup-e")]);
    let mut b = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b.add_entity_birth(ExecutionPendingEntityBirth::new(face_ref("dup-e"), FACE_SCHEME_ID, occ, role))
        .unwrap();
    let err = ExecutionNamingCommitAuthority::commit_with_naming_plan(runtime, execution, tx, b.build())
        .expect_err("duplicate entity birth must refuse after cold restore");
    assert!(
        matches!(
            err,
            ExecutionNamingDenial::RelationalNaming(LineageNameDenial::DuplicateBirth {
                identity,
                first: PersistentNameHolder::Entity(first_id),
                ..
            }) if identity == expected_id && first_id == expected_entity
        ),
        "unexpected duplicate entity refusal: {err:?}"
    );
}

fn assert_dup_seam_refuses(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    src: EntityId,
    dst: EntityId,
    occ: &str,
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
    let mut b = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b.add_relation_birth(
        ExecutionPendingRelationBirth::new(dup_ref, SEAM_SCHEME_ID, occ, "seam")
            .with_discriminator(&SeamSplitDisc::Ordinal { index: 1 }),
    )
    .unwrap();
    let err = ExecutionNamingCommitAuthority::commit_with_naming_plan(runtime, execution, tx, b.build())
        .expect_err("duplicate non-head relation birth must refuse after cold restore");
    assert!(
        matches!(
            err,
            ExecutionNamingDenial::RelationalNaming(LineageNameDenial::DuplicateBirth {
                identity,
                first: PersistentNameHolder::Relation(first_id),
                ..
            }) if identity == expected_id && first_id == expected_rel
        ),
        "unexpected duplicate relation refusal: {err:?}"
    );
}

#[test]
fn external_cold_directory_checkpoint_preserves_basis_nonhead_relation_and_tail_split() {
    let catalog = lowered_cad_catalog();
    let source = fresh_cad_runtime();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let main = BranchId("main".to_owned());

    // Commit 1 (NON-HEAD pre-checkpoint): F0, F1, F2, and named relations S1 (F1 -> F2), S2 (F0 -> F2).
    let f0_ref = face_ref("f0");
    let f1_ref = face_ref("f1");
    let f2_ref = face_ref("f2");
    let s1_ref = seam_ref("s1", EntityReference::Created(f1_ref.clone()), EntityReference::Created(f2_ref.clone()));
    let s2_ref = seam_ref("s2", EntityReference::Created(f0_ref.clone()), EntityReference::Created(f2_ref.clone()));
    let tx1 = begin_staged_tx(
        &source,
        "c1-nonhead",
        vec![
            create_face_intent("f0"),
            create_face_intent("f1"),
            create_face_intent("f2"),
            create_seam_intent("s1", EntityReference::Created(f1_ref.clone()), EntityReference::Created(f2_ref.clone())),
            create_seam_intent("s2", EntityReference::Created(f0_ref.clone()), EntityReference::Created(f2_ref.clone())),
        ],
    );
    let mut b1 = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b1.add_entity_birth(ExecutionPendingEntityBirth::new(f0_ref.clone(), FACE_SCHEME_ID, "occ.cold.f0", "authored"))
        .unwrap()
        .add_entity_birth(ExecutionPendingEntityBirth::new(f1_ref.clone(), FACE_SCHEME_ID, "occ.cold.f1", "authored"))
        .unwrap()
        .add_entity_birth(ExecutionPendingEntityBirth::new(f2_ref.clone(), FACE_SCHEME_ID, "occ.cold.f2", "authored"))
        .unwrap()
        .add_relation_birth(
            ExecutionPendingRelationBirth::new(s1_ref.clone(), SEAM_SCHEME_ID, "occ.cold.s1", "seam")
                .with_discriminator(&SeamSplitDisc::Ordinal { index: 1 }),
        )
        .unwrap()
        .add_relation_birth(
            ExecutionPendingRelationBirth::new(s2_ref.clone(), SEAM_SCHEME_ID, "occ.cold.s2", "seam")
                .with_discriminator(&SeamSplitDisc::Ordinal { index: 1 }),
        )
        .unwrap();
    let (res1, names1) = ExecutionNamingCommitAuthority::commit_with_naming_plan(&source, &execution, tx1, b1.build())
        .expect("c1 succeeds")
        .into_parts();
    let (f1_id, _) = names1.created_entity_name::<CadNamingSchema, Face>(&res1, &f1_ref).unwrap();
    let (f2_id, _) = names1.created_entity_name::<CadNamingSchema, Face>(&res1, &f2_ref).unwrap();
    let (s1_id, _) = names1.created_relation_name::<Seam>(&res1, &s1_ref).unwrap();
    let f1_rel = names1.entity_relational_identity(f1_id).unwrap();
    let s1_rel = names1.relation_relational_identity(s1_id).unwrap();
    let (s2_id, _) = names1.created_relation_name::<Seam>(&res1, &s2_ref).unwrap();
    let s2_rel = names1.relation_relational_identity(s2_id).unwrap();
    release_commit_snapshot(&source, &res1);

    // Commit 2 (HEAD pre-checkpoint): F3 only, making Commit 1's named relation S1 strictly non-head.
    let f3_ref = face_ref("f3");
    let tx2 = begin_staged_tx(&source, "c2-head", vec![create_face_intent("f3")]);
    let mut b2 = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b2.add_entity_birth(ExecutionPendingEntityBirth::new(f3_ref.clone(), FACE_SCHEME_ID, "occ.cold.f3", "authored"))
        .unwrap();
    let (res2, names2) = ExecutionNamingCommitAuthority::commit_with_naming_plan(&source, &execution, tx2, b2.build())
        .expect("c2 succeeds")
        .into_parts();
    let (f3_id, _) = names2.created_entity_name::<CadNamingSchema, Face>(&res2, &f3_ref).unwrap();
    release_commit_snapshot(&source, &res2);

    let expected_ckpt_basis = source.branch_live_persistent_names(&main);
    let expected_s1_lin = source.relation_lineage(s1_id).expect("non-head seam lineage exists");

    // Capture an external-member native checkpoint so reopen MUST go through cold_directory::restore.
    let native = source
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .expect("external-member native checkpoint captures");
    let covered_pos = native.covered_position().expect("covered position");
    let covered_trans = native.covered_branch_transitions().expect("covered transitions");
    let cold_source_ckpt = ExternalColdSource::from_native_capture(&native, &source, &execution);

    // Commit 3 (journal tail): split pre-checkpoint F1 into F1a and F1b.
    let f1a_ref = face_ref("f1a");
    let f1b_ref = face_ref("f1b");
    let tx3 = begin_staged_tx(
        &source,
        "c3-tail-split",
        vec![
            MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: f1_id })),
            create_face_intent("f1a"),
            create_face_intent("f1b"),
        ],
    );
    let mut b3 = ExecutionNameBirthPlanBuilder::new(&catalog, [f1_rel]);
    b3.add_split_posture(
        LineageRecordTarget::Entity(f1_id),
        f1_rel,
        vec![
            ExecutionSplitChildSpec::entity(f1a_ref.clone(), FACE_SCHEME_ID, "occ.cold.split", "cut_piece")
                .with_discriminator(&FaceSplitDisc::Side { side: 1 }),
            ExecutionSplitChildSpec::entity(f1b_ref.clone(), FACE_SCHEME_ID, "occ.cold.split", "cut_piece")
                .with_discriminator(&FaceSplitDisc::Side { side: 2 }),
        ],
    )
    .unwrap();
    let (res3, names3) = ExecutionNamingCommitAuthority::commit_with_naming_plan(&source, &execution, tx3, b3.build())
        .expect("tail split succeeds")
        .into_parts();
    let (f1a_id, _) = names3.created_entity_name::<CadNamingSchema, Face>(&res3, &f1a_ref).unwrap();
    let f1a_rel = names3.entity_relational_identity(f1a_id).unwrap();
    release_commit_snapshot(&source, &res3);

    let expected_tail_basis = source.branch_live_persistent_names(&main);
    assert!(!expected_tail_basis.contains_key(&f1_rel));
    assert!(expected_tail_basis.contains_key(&f1a_rel));
    // S1 ran from F1, so it retired with its endpoint in the tail split and its
    // name ended there; S2 never touched F1 and keeps its name live.
    assert_eq!(expected_tail_basis.get(&s1_rel), None);
    assert_eq!(expected_tail_basis.get(&s2_rel), Some(&PersistentNameHolder::Relation(s2_id)));

    // Part A: Cold checkpoint-only restore.
    let mut reopened_ckpt = fresh_cad_runtime();
    let recovery_ckpt = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(covered_pos, covered_trans, native.clone())),
        Vec::new(),
        Vec::new(),
        cold_source_ckpt.clone(),
    );
    let plan_ckpt = reopened_ckpt
        .durability()
        .recovery_plan_from_source(&execution, recovery_ckpt, RecoveryVerificationMode::NormalRecoveryVerification)
        .expect("cold checkpoint plan builds");
    reopened_ckpt
        .durability_recovery()
        .recover(&execution, plan_ckpt)
        .expect("cold checkpoint restore succeeds");
    assert_eq!(*cold_source_ckpt.region_reads.lock().unwrap(), 0, "record regions remain cold");
    assert!(*cold_source_ckpt.lineage_reads.lock().unwrap() > 0, "cold_directory eagerly materialized lineage pages");
    assert_eq!(reopened_ckpt.branch_live_persistent_names(&main), expected_ckpt_basis);
    assert_eq!(reopened_ckpt.relation_lineage(s1_id), Some(expected_s1_lin));

    let head_a = reopened_ckpt.admit_branch_basis(&reopened_ckpt.main_branch_identity()).unwrap().observation().commit_id();
    assert_dup_entity_refuses(&reopened_ckpt, &execution, "occ.cold.f1", "authored", f1_rel, f1_id);
    assert_dup_seam_refuses(&reopened_ckpt, &execution, f2_id, f3_id, "occ.cold.s1", s1_rel, s1_id);
    assert_eq!(reopened_ckpt.admit_branch_basis(&reopened_ckpt.main_branch_identity()).unwrap().observation().commit_id(), head_a);
    assert_eq!(reopened_ckpt.branch_live_persistent_names(&main), expected_ckpt_basis);

    // Part B: Cold checkpoint + journal-tail Split of pre-checkpoint F1.
    let tail_commits: Vec<RelationalCanonicalCommitArtifact> = source
        .durability()
        .canonical_commit_artifacts(&execution)
        .after_position(covered_pos)
        .map(|res| {
            let artifact = res.expect("export tail");
            let _verified = RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), &execution)
                .expect("readmit tail with verified persisted births");
            artifact
        })
        .collect();
    assert_eq!(tail_commits.len(), 1);

    let cold_source_tail = ExternalColdSource::from_native_capture(&native, &source, &execution);
    let mut reopened_tail = fresh_cad_runtime();
    let recovery_tail = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(covered_pos, covered_trans, native)),
        tail_commits,
        Vec::new(),
        cold_source_tail,
    );
    let plan_tail = reopened_tail
        .durability()
        .recovery_plan_from_source(&execution, recovery_tail, RecoveryVerificationMode::NormalRecoveryVerification)
        .expect("cold checkpoint + tail plan builds");
    let outcome = reopened_tail
        .durability_recovery()
        .recover(&execution, plan_tail)
        .expect("cold checkpoint + tail Split replay succeeds");
    assert_eq!(outcome.recovered_commits, 3);
    assert_eq!(reopened_tail.branch_live_persistent_names(&main), expected_tail_basis);
    assert_eq!(reopened_tail.relation_lineage(s1_id), Some(expected_s1_lin));
    let live_ids: BTreeSet<_> = reopened_tail.branch_live_persistent_names(&main).keys().copied().collect();
    assert!(live_ids.contains(&f1a_rel) && live_ids.contains(&s2_rel));
    assert!(!live_ids.contains(&s1_rel) && !live_ids.contains(&f1_rel));

    let head_b = reopened_tail.admit_branch_basis(&reopened_tail.main_branch_identity()).unwrap().observation().commit_id();
    assert_dup_seam_refuses(&reopened_tail, &execution, f2_id, f3_id, "occ.cold.s2", s2_rel, s2_id);
    assert_eq!(reopened_tail.admit_branch_basis(&reopened_tail.main_branch_identity()).unwrap().observation().commit_id(), head_b);
    assert_eq!(reopened_tail.branch_live_persistent_names(&main), expected_tail_basis);
}
