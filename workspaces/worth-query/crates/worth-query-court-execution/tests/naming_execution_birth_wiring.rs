//! End-to-end Slice 1B courts for `worth-query-execution-naming` ->
//! `worth-relational` commit-finalization wiring: fresh-build determinism,
//! dummy pre-allocation invariance, worker-count invariance (1 vs 2 vs 8),
//! kind-bound admission refusal (`KindIdentityMismatch`), post-retirement
//! commit-local admission immunity, fail-closed `MissingBirth`, and
//! discriminator reference resolution.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_query_execution_naming::facade::{
    ExecutionNameBirthPlanBuilder, ExecutionNamingCommitAuthority, ExecutionNamingDenial,
    ExecutionPendingEntityBirth, ExecutionPendingRelationBirth,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::lineage::{LineageNameDenial, LineageRecordTarget};
use worth_relational::facade::transactions::{
    DeleteEntityIntent, EntityMutationIntent, EntityReference, MutationIntent,
};

use crate::naming_execution_fixture::{
    begin_staged_tx, commit_dummy_preallocations, create_face_intent, create_seam_intent,
    execution_lease_for_workers, face_ref, fresh_cad_runtime, lowered_cad_catalog,
    release_commit_snapshot, seam_ref, CadNamingSchema, Edge, Face, FaceSplitDisc, Seam,
    SeamSplitDisc, FACE_SCHEME_ID, ROOM_GROUP_SCHEME_ID, SEAM_SCHEME_ID,
};

struct ScenarioResult {
    face1_id: EntityId,
    seam1_id: RelationId,
    face1_name: WorthQueryName<Face>,
    face2_name: WorthQueryName<Face>,
    seam1_name: WorthQueryName<Seam>,
    seam2_name: WorthQueryName<Seam>,
}

fn run_cad_birth_scenario(
    workers: usize,
    dummy_entities: usize,
    dummy_relations: usize,
    reverse_submission: bool,
) -> ScenarioResult {
    let catalog = lowered_cad_catalog();
    let runtime = fresh_cad_runtime();
    let lease = execution_lease_for_workers(workers);
    let execution = ExecutionRequest::leased(&lease);

    commit_dummy_preallocations(&runtime, &execution, dummy_entities, dummy_relations);

    let f1_ref = face_ref("face-left");
    let f2_ref = face_ref("face-right");
    let s1_ref = seam_ref("seam-top", EntityReference::Created(f1_ref.clone()), EntityReference::Created(f2_ref.clone()));
    let s2_ref = seam_ref("seam-bottom", EntityReference::Created(f2_ref.clone()), EntityReference::Created(f1_ref.clone()));

    let mut intents = vec![
        create_face_intent("face-left"),
        create_face_intent("face-right"),
        create_seam_intent("seam-top", EntityReference::Created(f1_ref.clone()), EntityReference::Created(f2_ref.clone())),
        create_seam_intent("seam-bottom", EntityReference::Created(f2_ref.clone()), EntityReference::Created(f1_ref.clone())),
    ];
    if reverse_submission {
        intents.swap(0, 1);
        intents.swap(2, 3);
    }
    let tx = begin_staged_tx(&runtime, "cad-birth-scenario", intents);

    let f1_birth = ExecutionPendingEntityBirth::new(f1_ref.clone(), FACE_SCHEME_ID, "occ.extrude.1", "authored")
        .with_discriminator(&FaceSplitDisc::Side { side: 1 });
    let f2_birth = ExecutionPendingEntityBirth::new(f2_ref.clone(), FACE_SCHEME_ID, "occ.extrude.1", "authored")
        .with_discriminator(&FaceSplitDisc::Side { side: 2 });
    let s1_birth = ExecutionPendingRelationBirth::new(s1_ref.clone(), SEAM_SCHEME_ID, "occ.extrude.1", "seam")
        .with_discriminator(&SeamSplitDisc::Ordinal { index: 1 });
    let s2_birth = ExecutionPendingRelationBirth::new(s2_ref.clone(), SEAM_SCHEME_ID, "occ.extrude.1", "seam")
        .with_discriminator(&SeamSplitDisc::Ordinal { index: 2 });

    let mut builder = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    if reverse_submission {
        builder.add_relation_birth(s2_birth).unwrap();
        builder.add_entity_birth(f2_birth).unwrap();
        builder.add_relation_birth(s1_birth).unwrap();
        builder.add_entity_birth(f1_birth).unwrap();
    } else {
        builder.add_entity_birth(f1_birth).unwrap();
        builder.add_entity_birth(f2_birth).unwrap();
        builder.add_relation_birth(s1_birth).unwrap();
        builder.add_relation_birth(s2_birth).unwrap();
    }

    let (commit_result, admitted) = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx,
        builder.build(),
    )
    .expect("commit with lowered naming plan succeeds")
    .into_parts();

    let (face1_id, face1_name) = admitted.created_entity_name::<CadNamingSchema, Face>(&commit_result, &f1_ref).unwrap();
    let (_, face2_name) = admitted.created_entity_name::<CadNamingSchema, Face>(&commit_result, &f2_ref).unwrap();
    let (seam1_id, seam1_name) = admitted.created_relation_name::<Seam>(&commit_result, &s1_ref).unwrap();
    let (seam2_id, seam2_name) = admitted.created_relation_name::<Seam>(&commit_result, &s2_ref).unwrap();

    // Cross-kind mismatch refusal: Face birth cannot be admitted as Edge.
    assert!(matches!(
        admitted.entity_name::<CadNamingSchema, Edge>(face1_id),
        Err(ExecutionNamingDenial::KindIdentityMismatch {
            ref scheme_id,
            ref expected_subject,
            ref requested_subject,
        }) if scheme_id == FACE_SCHEME_ID
            && expected_subject == "Face"
            && requested_subject == "Edge"
    ));

    // Group scheme lowers cleanly without exposing any unauthenticated admission door.
    let room_scheme = catalog.group_scheme_by_id(ROOM_GROUP_SCHEME_ID).expect("room group lowered");
    assert_eq!(room_scheme.scheme_id(), ROOM_GROUP_SCHEME_ID);
    assert!(room_scheme.relational_member_role("wall").is_some());
    assert!(room_scheme.relational_member_role("producer").is_some());

    assert!(runtime.relation_lineage(seam1_id).is_some());
    assert!(runtime.relation_lineage(seam2_id).is_some());
    let f1_rel_id = admitted.entity_relational_identity(face1_id).unwrap();
    release_commit_snapshot(&runtime, &commit_result);

    // Commit-local admission immunity: retiring face1 in a later commit leaves `admitted` intact.
    let tx_retire = begin_staged_tx(
        &runtime,
        "retire-f1",
        vec![MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: face1_id }))],
    );
    let mut b_retire = ExecutionNameBirthPlanBuilder::new(&catalog, [f1_rel_id]);
    b_retire.add_retire_entity(face1_id, f1_rel_id);
    let retired = ExecutionNamingCommitAuthority::commit_with_naming_plan(&runtime, &execution, tx_retire, b_retire.build())
        .expect("retiring face 1 succeeds");
    release_commit_snapshot(&runtime, retired.commit_result());
    assert_eq!(admitted.entity_name::<CadNamingSchema, Face>(face1_id).unwrap(), face1_name);

    ScenarioResult { face1_id, seam1_id, face1_name, face2_name, seam1_name, seam2_name }
}

#[test]
fn end_to_end_birth_wiring_is_deterministic_across_fresh_builds_dummy_allocations_and_workers() {
    let run_1w = run_cad_birth_scenario(1, 0, 0, false);
    let run_2w_shifted = run_cad_birth_scenario(2, 37, 15, false);
    let run_8w_shifted_rev = run_cad_birth_scenario(8, 113, 29, true);

    assert_ne!(run_1w.face1_id, run_2w_shifted.face1_id);
    assert_ne!(run_2w_shifted.face1_id, run_8w_shifted_rev.face1_id);
    assert_ne!(run_1w.seam1_id, run_2w_shifted.seam1_id);
    assert_ne!(run_2w_shifted.seam1_id, run_8w_shifted_rev.seam1_id);

    for other in [&run_2w_shifted, &run_8w_shifted_rev] {
        assert_eq!(run_1w.face1_name.as_bytes(), other.face1_name.as_bytes());
        assert_eq!(run_1w.face2_name.as_bytes(), other.face2_name.as_bytes());
        assert_eq!(run_1w.seam1_name.as_bytes(), other.seam1_name.as_bytes());
        assert_eq!(run_1w.seam2_name.as_bytes(), other.seam2_name.as_bytes());
    }
    assert_ne!(run_1w.face1_name, run_1w.face2_name);
    assert_ne!(run_1w.seam1_name, run_1w.seam2_name);
}

#[test]
fn schemed_entity_or_relation_without_birth_plan_refuses_fail_closed_with_missing_birth() {
    let catalog = lowered_cad_catalog();
    let runtime = fresh_cad_runtime();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let main = BranchId("main".to_owned());
    let head = || runtime.admit_branch_basis(&runtime.main_branch_identity()).unwrap().observation().commit_id();

    let (head0, basis0) = (head(), runtime.branch_live_persistent_names(&main).unwrap());
    let tx_no_plan = begin_staged_tx(&runtime, "no-plan-face", vec![create_face_intent("f")]);
    let err = runtime.commit_branch_transaction(&execution, tx_no_plan).expect_err("schemed entity must refuse");
    assert!(matches!(
        err.lineage_name_denial_ref(),
        Some(LineageNameDenial::MissingBirth { target: LineageRecordTarget::Entity(_), scheme_id }) if scheme_id == FACE_SCHEME_ID
    ));
    assert_eq!(head(), head0);
    assert_eq!(runtime.branch_live_persistent_names(&main).unwrap(), basis0);

    let (f1_ref, f2_ref) = (face_ref("f1"), face_ref("f2"));
    let tx_seed = begin_staged_tx(&runtime, "seed", vec![create_face_intent("f1"), create_face_intent("f2")]);
    let mut b_seed = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b_seed
        .add_entity_birth(ExecutionPendingEntityBirth::new(f1_ref.clone(), FACE_SCHEME_ID, "occ.f1", "authored")).unwrap()
        .add_entity_birth(ExecutionPendingEntityBirth::new(f2_ref.clone(), FACE_SCHEME_ID, "occ.f2", "authored")).unwrap();
    let (seed_res, seed_names) = ExecutionNamingCommitAuthority::commit_with_naming_plan(&runtime, &execution, tx_seed, b_seed.build())
        .expect("seed faces succeed")
        .into_parts();
    let (f1_id, _) = seed_names.created_entity_name::<CadNamingSchema, Face>(&seed_res, &f1_ref).unwrap();
    let (f2_id, _) = seed_names.created_entity_name::<CadNamingSchema, Face>(&seed_res, &f2_ref).unwrap();
    release_commit_snapshot(&runtime, &seed_res);

    let (head1, basis1) = (head(), runtime.branch_live_persistent_names(&main).unwrap());
    let tx_no_seam = begin_staged_tx(
        &runtime,
        "no-plan-seam",
        vec![create_seam_intent("s", EntityReference::Existing(f1_id), EntityReference::Existing(f2_id))],
    );
    let seam_err = runtime.commit_branch_transaction(&execution, tx_no_seam).expect_err("schemed relation must refuse");
    assert!(matches!(
        seam_err.lineage_name_denial_ref(),
        Some(LineageNameDenial::MissingBirth { target: LineageRecordTarget::Relation(_), scheme_id }) if scheme_id == SEAM_SCHEME_ID
    ));
    assert_eq!(head(), head1);
    assert_eq!(runtime.branch_live_persistent_names(&main).unwrap(), basis1);
}

#[test]
fn basis_and_same_occurrence_discriminator_reference_resolution_and_forward_refusal() {
    let catalog = lowered_cad_catalog();
    let runtime = fresh_cad_runtime();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);

    let f1_ref = face_ref("cutter");
    let tx1 = begin_staged_tx(&runtime, "c1", vec![create_face_intent("cutter")]);
    let mut b1 = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b1.add_entity_birth(ExecutionPendingEntityBirth::new(f1_ref.clone(), FACE_SCHEME_ID, "occ.cutter", "authored")).unwrap();
    let (c1_res, c1_names) = ExecutionNamingCommitAuthority::commit_with_naming_plan(&runtime, &execution, tx1, b1.build())
        .expect("commit 1 succeeds")
        .into_parts();
    let (cutter_entity, cutter_name) = c1_names.created_entity_name::<CadNamingSchema, Face>(&c1_res, &f1_ref).unwrap();
    let cutter_rel_id = c1_names.entity_relational_identity(cutter_entity).unwrap();
    release_commit_snapshot(&runtime, &c1_res);

    let f2_ref = face_ref("cut-child");
    let s1_ref = seam_ref("seam-ref", EntityReference::Existing(cutter_entity), EntityReference::Created(f2_ref.clone()));
    let tx2 = begin_staged_tx(
        &runtime,
        "c2",
        vec![
            create_face_intent("cut-child"),
            create_seam_intent("seam-ref", EntityReference::Existing(cutter_entity), EntityReference::Created(f2_ref.clone())),
        ],
    );
    let mut b2 = ExecutionNameBirthPlanBuilder::new(&catalog, [cutter_rel_id]);
    b2.add_entity_birth(
        ExecutionPendingEntityBirth::new(f2_ref.clone(), FACE_SCHEME_ID, "occ.cut.1", "cut_piece")
            .with_discriminator(&FaceSplitDisc::Cutter { cutter: cutter_name, piece: 1 }),
    )
    .unwrap();
    b2.add_relation_birth(
        ExecutionPendingRelationBirth::new(s1_ref.clone(), SEAM_SCHEME_ID, "occ.cut.1", "seam")
            .with_discriminator(&SeamSplitDisc::ByFace { face: cutter_name, index: 1 }),
    )
    .unwrap();
    let (c2_res, c2_names) = ExecutionNamingCommitAuthority::commit_with_naming_plan(&runtime, &execution, tx2, b2.build())
        .expect("basis-referenced discriminator commit succeeds")
        .into_parts();
    let (_, cut_name) = c2_names.created_entity_name::<CadNamingSchema, Face>(&c2_res, &f2_ref).unwrap();
    let (_, seam_name) = c2_names.created_relation_name::<Seam>(&c2_res, &s1_ref).unwrap();
    assert_ne!(cut_name, cutter_name);
    assert_eq!(seam_name.as_bytes().len(), 32);
    release_commit_snapshot(&runtime, &c2_res);

    let mut b_bad = ExecutionNameBirthPlanBuilder::new(&catalog, [cutter_rel_id]);
    let err = b_bad
        .add_entity_birth(
            ExecutionPendingEntityBirth::new(face_ref("bad-forward"), FACE_SCHEME_ID, "occ.bad", "cut_piece")
                .with_discriminator(&FaceSplitDisc::Cutter { cutter: cutter_name, piece: 2 })
                .with_same_occurrence_reference(cutter_name, "seam_patch"),
        )
        .err()
        .expect("forward same-occurrence reference must refuse");
    assert!(matches!(
        err,
        ExecutionNamingDenial::NonPrecedingSameOccurrenceRole {
            ref scheme_id,
            ref role_path,
            ref referenced_role,
        } if scheme_id == FACE_SCHEME_ID && role_path == "cut_piece" && referenced_role == "seam_patch"
    ));

    let mut b_unborn = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    let unborn_err = b_unborn
        .add_entity_birth(
            ExecutionPendingEntityBirth::new(face_ref("bad-unborn"), FACE_SCHEME_ID, "occ.unborn", "cut_piece")
                .with_discriminator(&FaceSplitDisc::Cutter { cutter: cutter_name, piece: 3 }),
        )
        .err()
        .expect("unborn referenced name must refuse");
    assert!(matches!(
        unborn_err,
        ExecutionNamingDenial::UnresolvedReferencedName {
            ref scheme_id,
            ref role_path,
            digest,
        } if scheme_id == FACE_SCHEME_ID && role_path == "cut_piece" && digest == *cutter_name.as_bytes()
    ));
}
