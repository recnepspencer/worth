//! End-to-end Slice 1B court proving fork ancestry inheritance in Relational's
//! self-derived naming basis (`DuplicateBirth` across entity & relation names on
//! a forked branch, and independent retirement on the fork without disturbing
//! `main`).

use worth_execution::ExecutionRequest;
use worth_query_execution_naming::facade::{
    ExecutionNameBirthPlanBuilder, ExecutionNamingCommitAuthority, ExecutionNamingDenial,
    ExecutionPendingEntityBirth, ExecutionPendingRelationBirth,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::{LineageNameDenial, PersistentNameHolder};
use worth_relational::facade::transactions::{
    DeleteEntityIntent, EntityMutationIntent, EntityReference, MutationIntent,
};

use crate::naming_execution_fixture::{
    begin_branch_staged_tx, begin_staged_tx, create_face_intent, create_seam_intent,
    execution_lease_for_workers, face_ref, fresh_cad_runtime, lowered_cad_catalog,
    release_commit_snapshot, seam_ref, CadNamingSchema, Face, Seam, SeamSplitDisc, FACE_SCHEME_ID,
    SEAM_SCHEME_ID,
};

#[test]
fn forked_branch_inherits_entity_and_relation_naming_basis_and_retires_independently() {
    let catalog = lowered_cad_catalog();
    let runtime = fresh_cad_runtime();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let main = BranchId("main".to_owned());

    // 1. Seed F1, F2, and S1 (F1 -> F2) on main.
    let f1_ref = face_ref("f1");
    let f2_ref = face_ref("f2");
    let s1_ref = seam_ref(
        "s1",
        EntityReference::Created(f1_ref.clone()),
        EntityReference::Created(f2_ref.clone()),
    );
    let tx1 = begin_staged_tx(
        &runtime,
        "seed",
        vec![
            create_face_intent("f1"),
            create_face_intent("f2"),
            create_seam_intent(
                "s1",
                EntityReference::Created(f1_ref.clone()),
                EntityReference::Created(f2_ref.clone()),
            ),
        ],
    );
    let mut b1 = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b1.add_entity_birth(ExecutionPendingEntityBirth::new(
        f1_ref.clone(),
        FACE_SCHEME_ID,
        "occ.main.f1",
        "authored",
    ))
    .unwrap()
    .add_entity_birth(ExecutionPendingEntityBirth::new(
        f2_ref.clone(),
        FACE_SCHEME_ID,
        "occ.main.f2",
        "authored",
    ))
    .unwrap()
    .add_relation_birth(
        ExecutionPendingRelationBirth::new(s1_ref.clone(), SEAM_SCHEME_ID, "occ.main.s1", "seam")
            .with_discriminator(&SeamSplitDisc::Ordinal { index: 1 }),
    )
    .unwrap();
    let (res1, names1) = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx1,
        b1.build(),
    )
    .expect("seed on main succeeds")
    .into_parts();
    let (f1_id, _) = names1
        .created_entity_name::<CadNamingSchema, Face>(&res1, &f1_ref)
        .unwrap();
    let (f2_id, _) = names1
        .created_entity_name::<CadNamingSchema, Face>(&res1, &f2_ref)
        .unwrap();
    let (s1_id, _) = names1
        .created_relation_name::<Seam>(&res1, &s1_ref)
        .unwrap();
    let f1_rel = names1.entity_relational_identity(f1_id).unwrap();
    let s1_rel = names1.relation_relational_identity(s1_id).unwrap();
    release_commit_snapshot(&runtime, &res1);

    // 2. Fork "fork-a" from main.
    let fork_branch = BranchId("fork-a".to_owned());
    let (_, source_basis) = runtime
        .observe_fork_source(&main)
        .expect("main is forkable");
    let forked = runtime
        .fork_branch(&execution, fork_branch.clone(), source_basis)
        .expect("fork succeeds");
    let fork_identity = forked.target_identity().clone();

    let fork_head = || {
        runtime
            .admit_branch_basis(&fork_identity)
            .unwrap()
            .observation()
            .commit_id()
    };
    let head0 = fork_head();
    let basis0 = runtime.branch_live_persistent_names(&fork_branch).unwrap();
    assert_eq!(basis0, runtime.branch_live_persistent_names(&main).unwrap());

    // 3. Re-birthing inherited F1 on fork-a refuses DuplicateBirth naming Entity(f1_id).
    let tx_dup_f = begin_branch_staged_tx(
        &runtime,
        &fork_identity,
        "dup-f1-fork",
        vec![create_face_intent("dup-f1")],
    );
    let mut b_dup_f = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b_dup_f
        .add_entity_birth(ExecutionPendingEntityBirth::new(
            face_ref("dup-f1"),
            FACE_SCHEME_ID,
            "occ.main.f1",
            "authored",
        ))
        .unwrap();
    let dup_f_err = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_dup_f,
        b_dup_f.build(),
    )
    .expect_err("inherited entity DuplicateBirth on fork must refuse");
    assert!(matches!(
        dup_f_err,
        ExecutionNamingDenial::RelationalNaming(LineageNameDenial::DuplicateBirth {
            identity,
            first: PersistentNameHolder::Entity(first_id),
            ..
        }) if identity == f1_rel && first_id == f1_id
    ));
    assert_eq!(fork_head(), head0);
    assert_eq!(runtime.branch_live_persistent_names(&fork_branch).unwrap(), basis0);

    // 4. Re-birthing inherited S1 on fork-a (distinct endpoints f2 -> f1) refuses DuplicateBirth.
    let dup_s_ref = seam_ref(
        "dup-s1",
        EntityReference::Existing(f2_id),
        EntityReference::Existing(f1_id),
    );
    let tx_dup_s = begin_branch_staged_tx(
        &runtime,
        &fork_identity,
        "dup-s1-fork",
        vec![create_seam_intent(
            "dup-s1",
            EntityReference::Existing(f2_id),
            EntityReference::Existing(f1_id),
        )],
    );
    let mut b_dup_s = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b_dup_s
        .add_relation_birth(
            ExecutionPendingRelationBirth::new(dup_s_ref, SEAM_SCHEME_ID, "occ.main.s1", "seam")
                .with_discriminator(&SeamSplitDisc::Ordinal { index: 1 }),
        )
        .unwrap();
    let dup_s_err = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_dup_s,
        b_dup_s.build(),
    )
    .expect_err("inherited relation DuplicateBirth on fork must refuse");
    assert!(matches!(
        dup_s_err,
        ExecutionNamingDenial::RelationalNaming(LineageNameDenial::DuplicateBirth {
            identity,
            first: PersistentNameHolder::Relation(first_id),
            ..
        }) if identity == s1_rel && first_id == s1_id
    ));
    assert_eq!(fork_head(), head0);
    assert_eq!(runtime.branch_live_persistent_names(&fork_branch).unwrap(), basis0);

    // 5. Retiring inherited F1 on fork-a succeeds without disturbing main's live names.
    let tx_ret = begin_branch_staged_tx(
        &runtime,
        &fork_identity,
        "retire-f1-fork",
        vec![MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: f1_id },
        ))],
    );
    let mut b_ret = ExecutionNameBirthPlanBuilder::new(&catalog, [f1_rel]);
    b_ret.add_retire_entity(f1_id, f1_rel);
    let c_ret = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_ret,
        b_ret.build(),
    )
    .expect("retiring inherited entity on fork succeeds");
    release_commit_snapshot(&runtime, c_ret.commit_result());

    assert!(!runtime
        .branch_live_persistent_names(&fork_branch).unwrap()
        .contains_key(&f1_rel));
    assert_eq!(
        runtime.branch_live_persistent_names(&main).unwrap().get(&f1_rel),
        Some(&PersistentNameHolder::Entity(f1_id))
    );
}
