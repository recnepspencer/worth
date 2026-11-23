//! Slice 1B courts for reorder invariance vs parent-changing reorder,
//! `DuplicateBirth` & `InvalidDiscriminatorBasis` commit refusals with zero
//! committed effects (head AND naming basis unchanged), and Relational
//! self-derived live/retired basis enforcement.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_query_execution_naming::facade::{
    ExecutionNameBirthPlanBuilder, ExecutionNamingCommitAuthority, ExecutionNamingDenial,
    ExecutionPendingEntityBirth, ExecutionPendingRelationBirth,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::{
    CommitNamingPlan, DiscriminatorBasisDenialReason, LineageNameDenial, PendingBirthTarget,
    PendingCommitBirth, PersistentNameHolder,
};
use worth_relational::facade::transactions::{
    DeleteEntityIntent, EntityMutationIntent, EntityReference, MutationIntent,
};

use crate::naming_execution_fixture::{
    begin_staged_tx, create_face_intent, create_seam_intent, execution_lease_for_workers, face_ref,
    fresh_cad_runtime, lowered_cad_catalog, release_commit_snapshot, seam_ref, CadNamingSchema,
    Face, FaceSplitDisc, Seam, SeamSplitDisc, FACE_SCHEME_ID, SEAM_SCHEME_ID,
};

#[test]
fn independent_occurrence_reorder_invariance_vs_parent_changing_reorder() {
    let catalog = lowered_cad_catalog();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);

    let commit_two = |first_occ: &str, second_occ: &str| {
        let rt = fresh_cad_runtime();
        let commit_one = |occ: &str, key: &str| {
            let r = face_ref(key);
            let tx = begin_staged_tx(&rt, occ, vec![create_face_intent(key)]);
            let mut b = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
            b.add_entity_birth(ExecutionPendingEntityBirth::new(
                r.clone(),
                FACE_SCHEME_ID,
                occ,
                "authored",
            ))
            .unwrap();
            let c = ExecutionNamingCommitAuthority::commit_with_naming_plan(
                &rt,
                &execution,
                tx,
                b.build(),
            )
            .expect("commit succeeds");
            let (res, names) = c.into_parts();
            let (eid, name) = names
                .created_entity_name::<CadNamingSchema, Face>(&res, &r)
                .unwrap();
            let rel_id = names.entity_relational_identity(eid).unwrap();
            release_commit_snapshot(&rt, &res);
            (eid, name, rel_id)
        };
        let a = commit_one(first_occ, "k1");
        let b = commit_one(second_occ, "k2");
        (rt, a, b)
    };

    let (rt_ab, (e_f1_a, name_f1_a, id_f1_a), (_, name_f2_a, _)) = commit_two("occ.F1", "occ.F2");
    let (rt_ba, (e_f2_b, name_f2_b, id_f2_b), (_, name_f1_b, _)) = commit_two("occ.F2", "occ.F1");
    assert_eq!(name_f1_a, name_f1_b);
    assert_eq!(name_f2_a, name_f2_b);
    assert_ne!(name_f1_a, name_f2_a);

    let split_face = |rt: &worth_relational::facade::runtime::RelationalRuntime,
                      parent_e,
                      parent_name: WorthQueryName<Face>,
                      parent_rel_id| {
        let child_ref = face_ref("split-child");
        let tx = begin_staged_tx(
            rt,
            "split",
            vec![
                MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                    entity_id: parent_e,
                })),
                create_face_intent("split-child"),
            ],
        );
        let mut b = ExecutionNameBirthPlanBuilder::new(&catalog, [parent_rel_id]);
        b.add_retire_entity(parent_e, parent_rel_id);
        b.add_entity_birth(
            ExecutionPendingEntityBirth::new(
                child_ref.clone(),
                FACE_SCHEME_ID,
                "occ.cut.shared",
                "cut_piece",
            )
            .with_parent(parent_name)
            .with_discriminator(&FaceSplitDisc::Side { side: 1 }),
        )
        .unwrap();
        let c = ExecutionNamingCommitAuthority::commit_with_naming_plan(
            rt,
            &execution,
            tx,
            b.build(),
        )
        .expect("split commit succeeds");
        let (res, names) = c.into_parts();
        let (_, child_name) = names
            .created_entity_name::<CadNamingSchema, Face>(&res, &child_ref)
            .unwrap();
        release_commit_snapshot(rt, &res);
        child_name
    };

    let child_from_f1 = split_face(&rt_ab, e_f1_a, name_f1_a, id_f1_a);
    let child_from_f2 = split_face(&rt_ba, e_f2_b, name_f2_b, id_f2_b);
    assert_ne!(
        child_from_f1, child_from_f2,
        "parent-changing reorder must mint a distinct child name"
    );
}

#[test]
fn duplicate_birth_and_invalid_discriminator_refuse_with_zero_effects_using_self_derived_basis() {
    let catalog = lowered_cad_catalog();
    let runtime = fresh_cad_runtime();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let main = BranchId("main".to_owned());

    let head_commit = |rt: &worth_relational::facade::runtime::RelationalRuntime| {
        rt.admit_branch_basis(&rt.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id()
    };

    // 1. Same-commit sibling collision: two created entities claim identical birth.
    let head_before = head_commit(&runtime);
    let basis_before = runtime.branch_live_persistent_names(&main);
    let dup_a = face_ref("sib-a");
    let dup_b = face_ref("sib-b");
    let tx_sib = begin_staged_tx(
        &runtime,
        "sib-dup",
        vec![create_face_intent("sib-a"), create_face_intent("sib-b")],
    );
    let mut b_sib = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b_sib
        .add_entity_birth(ExecutionPendingEntityBirth::new(
            dup_a,
            FACE_SCHEME_ID,
            "occ.same",
            "authored",
        ))
        .unwrap()
        .add_entity_birth(ExecutionPendingEntityBirth::new(
            dup_b,
            FACE_SCHEME_ID,
            "occ.same",
            "authored",
        ))
        .unwrap();
    let sib_err = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_sib,
        b_sib.build(),
    )
    .err()
    .expect("same-commit sibling collision must refuse");
    assert!(matches!(
        sib_err,
        ExecutionNamingDenial::RelationalNaming(LineageNameDenial::DuplicateBirth {
            first: PersistentNameHolder::Entity(e1),
            second: PersistentNameHolder::Entity(e2),
            ..
        }) if e1 != e2
    ));
    assert_eq!(head_commit(&runtime), head_before);
    assert_eq!(runtime.branch_live_persistent_names(&main), basis_before);

    // 2. Seed entity F1 + F2 + relation S1, then prove Relational self-derives
    //    live holders across commits with zero caller-supplied live maps.
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
        "occ.live.1",
        "authored",
    ))
    .unwrap()
    .add_entity_birth(
        ExecutionPendingEntityBirth::new(f2_ref.clone(), FACE_SCHEME_ID, "occ.live.2", "authored"),
    )
    .unwrap()
    .add_relation_birth(
        ExecutionPendingRelationBirth::new(s1_ref.clone(), SEAM_SCHEME_ID, "occ.live.1", "seam")
            .with_discriminator(&SeamSplitDisc::Ordinal { index: 1 }),
    )
    .unwrap();
    let c1 = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx1,
        b1.build(),
    )
    .expect("seed commit succeeds");
    let (c1_res, c1_names) = c1.into_parts();
    let (f1_id, f1_name) = c1_names
        .created_entity_name::<CadNamingSchema, Face>(&c1_res, &f1_ref)
        .unwrap();
    let (f2_id, _) = c1_names
        .created_entity_name::<CadNamingSchema, Face>(&c1_res, &f2_ref)
        .unwrap();
    let (s1_id, s1_name) = c1_names
        .created_relation_name::<Seam>(&c1_res, &s1_ref)
        .unwrap();
    release_commit_snapshot(&runtime, &c1_res);

    let head_after_seed = head_commit(&runtime);
    let basis_after_seed = runtime.branch_live_persistent_names(&main);
    assert!(runtime.relation_lineage(s1_id).is_some());

    // Cross-commit duplicate entity birth.
    let dup_f_ref = face_ref("dup-f1");
    let tx_dup_f = begin_staged_tx(&runtime, "dup-f", vec![create_face_intent("dup-f1")]);
    let mut b_dup_f = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b_dup_f
        .add_entity_birth(ExecutionPendingEntityBirth::new(
            dup_f_ref,
            FACE_SCHEME_ID,
            "occ.live.1",
            "authored",
        ))
        .unwrap();
    let dup_f_err = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_dup_f,
        b_dup_f.build(),
    )
    .err()
    .expect("cross-commit live entity DuplicateBirth must refuse");
    assert!(matches!(
        dup_f_err,
        ExecutionNamingDenial::RelationalNaming(LineageNameDenial::DuplicateBirth {
            identity,
            ref first,
            second: PersistentNameHolder::Entity(other),
        }) if *identity.as_bytes() == *f1_name.as_bytes()
            && *first == PersistentNameHolder::Entity(f1_id)
            && other != f1_id
    ));
    assert_eq!(head_commit(&runtime), head_after_seed);
    assert_eq!(runtime.branch_live_persistent_names(&main), basis_after_seed);

    // Cross-commit duplicate relation birth (distinct endpoints f2 -> f1).
    let dup_s_ref = seam_ref(
        "dup-s1",
        EntityReference::Existing(f2_id),
        EntityReference::Existing(f1_id),
    );
    let tx_dup_s = begin_staged_tx(
        &runtime,
        "dup-s",
        vec![create_seam_intent(
            "dup-s1",
            EntityReference::Existing(f2_id),
            EntityReference::Existing(f1_id),
        )],
    );
    let mut b_dup_s = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b_dup_s
        .add_relation_birth(
            ExecutionPendingRelationBirth::new(dup_s_ref, SEAM_SCHEME_ID, "occ.live.1", "seam")
                .with_discriminator(&SeamSplitDisc::Ordinal { index: 1 }),
        )
        .unwrap();
    let dup_s_err = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_dup_s,
        b_dup_s.build(),
    )
    .err()
    .expect("cross-commit live relation DuplicateBirth must refuse");
    assert!(matches!(
        dup_s_err,
        ExecutionNamingDenial::RelationalNaming(LineageNameDenial::DuplicateBirth {
            identity,
            ref first,
            second: PersistentNameHolder::Relation(other),
        }) if *identity.as_bytes() == *s1_name.as_bytes()
            && *first == PersistentNameHolder::Relation(s1_id)
            && other != s1_id
    ));
    assert_eq!(head_commit(&runtime), head_after_seed);
    assert_eq!(runtime.branch_live_persistent_names(&main), basis_after_seed);

    // 3. Smuggled/undeclared embedded name reference in discriminator bytes
    //    refuses via Relational's scanner (`InvalidDiscriminatorBasis`).
    let bad_ref = face_ref("smuggled");
    let mut tx_bad = begin_staged_tx(&runtime, "smuggled", vec![create_face_intent("smuggled")]);
    let (disc_bytes, _) =
        worth_query_declaration::facade::application_naming::ApplicationNameDiscriminator::canonical_bytes_with_references(
            &FaceSplitDisc::Cutter {
                cutter: f1_name,
                piece: 9,
            },
        );
    tx_bad.attach_naming_plan(CommitNamingPlan {
        pending_births: vec![PendingCommitBirth {
            target: PendingBirthTarget::CreatedEntity(bad_ref),
            scheme_id: FACE_SCHEME_ID.to_owned(),
            occurrence_key: "occ.smuggle".to_owned(),
            role_path: "cut_piece".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: Vec::new(),
            canonical_discriminator: disc_bytes,
        }],
        ..Default::default()
    });
    let smuggle_err = runtime
        .commit_branch_transaction(&execution, tx_bad)
        .err()
        .expect("undeclared embedded name reference must refuse");
    assert!(matches!(
        smuggle_err.lineage_name_denial_ref(),
        Some(LineageNameDenial::InvalidDiscriminatorBasis {
            reason: DiscriminatorBasisDenialReason::UndeclaredEncodedNameReference { raw_digest },
            ..
        }) if raw_digest == f1_name.as_bytes()
    ));
    assert_eq!(head_commit(&runtime), head_after_seed);
    assert_eq!(runtime.branch_live_persistent_names(&main), basis_after_seed);
}
