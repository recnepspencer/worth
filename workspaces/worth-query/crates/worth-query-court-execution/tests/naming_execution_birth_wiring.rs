//! End-to-end Slice 1B courts for `worth-query-execution-naming` ->
//! `worth-relational` commit-finalization wiring across entities, relations,
//! and groups: fresh-build determinism, dummy pre-allocation invariance,
//! worker-count invariance (1 vs 2 vs 8), and discriminator reference resolution.

use std::collections::BTreeSet;

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_query_execution_naming::facade::{
    ExecutionGroupMemberSpec, ExecutionNameBirthPlanBuilder, ExecutionNamingCommitAuthority,
    ExecutionNamingDenial, ExecutionPendingEntityBirth, ExecutionPendingRelationBirth,
};
use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::schema::NativeMemberKey;
use worth_relational::facade::transactions::EntityReference;

use crate::naming_execution_fixture::{
    begin_staged_tx, commit_dummy_preallocations, create_face_intent, create_seam_intent,
    execution_lease_for_workers, face_ref, fresh_cad_runtime, lowered_cad_catalog,
    release_commit_snapshot, seam_ref, Face, FaceSplitDisc, RoomGroup, Seam, SeamSplitDisc,
    FACE_SCHEME_ID, ROOM_GROUP_SCHEME_ID, SEAM_SCHEME_ID,
};

struct ScenarioResult {
    face1_id: EntityId,
    seam1_id: RelationId,
    face1_name: WorthQueryName<Face>,
    face2_name: WorthQueryName<Face>,
    seam1_name: WorthQueryName<Seam>,
    seam2_name: WorthQueryName<Seam>,
    room_name: WorthQueryName<RoomGroup>,
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

    // Commit 1: two faces and two parallel seam relations between them.
    let f1_ref = face_ref("face-left");
    let f2_ref = face_ref("face-right");
    let s1_ref = seam_ref(
        "seam-top",
        EntityReference::Created(f1_ref.clone()),
        EntityReference::Created(f2_ref.clone()),
    );
    let s2_ref = seam_ref(
        "seam-bottom",
        EntityReference::Created(f2_ref.clone()),
        EntityReference::Created(f1_ref.clone()),
    );

    let mut intents = vec![
        create_face_intent("face-left"),
        create_face_intent("face-right"),
        create_seam_intent(
            "seam-top",
            EntityReference::Created(f1_ref.clone()),
            EntityReference::Created(f2_ref.clone()),
        ),
        create_seam_intent(
            "seam-bottom",
            EntityReference::Created(f2_ref.clone()),
            EntityReference::Created(f1_ref.clone()),
        ),
    ];
    if reverse_submission {
        intents.swap(0, 1);
        intents.swap(2, 3);
    }
    let tx = begin_staged_tx(&runtime, "cad-birth-scenario", intents);

    let f1_birth = ExecutionPendingEntityBirth::new(
        f1_ref.clone(),
        FACE_SCHEME_ID,
        "occ.extrude.1",
        "authored",
    )
    .with_discriminator(&FaceSplitDisc::Side { side: 1 });
    let f2_birth = ExecutionPendingEntityBirth::new(
        f2_ref.clone(),
        FACE_SCHEME_ID,
        "occ.extrude.1",
        "authored",
    )
    .with_discriminator(&FaceSplitDisc::Side { side: 2 });
    let s1_birth =
        ExecutionPendingRelationBirth::new(s1_ref.clone(), SEAM_SCHEME_ID, "occ.extrude.1", "seam")
            .with_discriminator(&SeamSplitDisc::Ordinal { index: 1 });
    let s2_birth =
        ExecutionPendingRelationBirth::new(s2_ref.clone(), SEAM_SCHEME_ID, "occ.extrude.1", "seam")
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
    let plan = builder.build();

    let committed =
        ExecutionNamingCommitAuthority::commit_with_naming_plan(&runtime, &execution, tx, plan)
            .expect("commit with lowered naming plan succeeds");
    let (commit_result, admitted) = committed.into_parts();

    let (face1_id, face1_name) = admitted
        .created_entity_name::<Face>(&commit_result, &f1_ref)
        .expect("face 1 admitted");
    let (face2_id, face2_name) = admitted
        .created_entity_name::<Face>(&commit_result, &f2_ref)
        .expect("face 2 admitted");
    let (seam1_id, seam1_name) = admitted
        .created_relation_name::<Seam>(&commit_result, &s1_ref)
        .expect("seam 1 admitted");
    let (seam2_id, seam2_name) = admitted
        .created_relation_name::<Seam>(&commit_result, &s2_ref)
        .expect("seam 2 admitted");

    // Verify Relational persisted both relation lineages across commits.
    assert!(
        runtime.relation_lineage(seam1_id).is_some(),
        "seam 1 lineage must be persisted in Relational"
    );
    assert!(
        runtime.relation_lineage(seam2_id).is_some(),
        "seam 2 lineage must be persisted in Relational"
    );

    // Derive a boundary-continuity group identity over the minted faces.
    let room_scheme = catalog
        .group_scheme_by_id(ROOM_GROUP_SCHEME_ID)
        .expect("room group lowered");
    let f1_rel_id = admitted.entity_relational_identity(face1_id).unwrap();
    let f2_rel_id = admitted.entity_relational_identity(face2_id).unwrap();
    let mut members = vec![
        ExecutionGroupMemberSpec {
            member_identity: f1_rel_id,
            native_key: NativeMemberKey::Entity(face1_id),
            declared_role: "wall".to_owned(),
        },
        ExecutionGroupMemberSpec {
            member_identity: f2_rel_id,
            native_key: NativeMemberKey::Entity(face2_id),
            declared_role: "producer".to_owned(),
        },
    ];
    if reverse_submission {
        members.reverse();
    }
    let group_outcome = room_scheme
        .execute_group_transition::<RoomGroup>(&[], &BTreeSet::new(), &[members])
        .expect("group birth transition succeeds");
    assert_eq!(group_outcome.admitted_group_names.len(), 1);
    let room_name = group_outcome.admitted_group_names[0];

    release_commit_snapshot(&runtime, &commit_result);
    ScenarioResult {
        face1_id,
        seam1_id,
        face1_name,
        face2_name,
        seam1_name,
        seam2_name,
        room_name,
    }
}

#[test]
fn end_to_end_birth_wiring_is_deterministic_across_fresh_builds_dummy_allocations_and_workers() {
    let run_1w = run_cad_birth_scenario(1, 0, 0, false);
    let run_2w_shifted = run_cad_birth_scenario(2, 37, 15, false);
    let run_8w_shifted_rev = run_cad_birth_scenario(8, 113, 29, true);

    // Prove allocation coordinates genuinely shifted under dummy pre-allocations.
    assert_ne!(run_1w.face1_id, run_2w_shifted.face1_id);
    assert_ne!(run_2w_shifted.face1_id, run_8w_shifted_rev.face1_id);
    assert_ne!(run_1w.seam1_id, run_2w_shifted.seam1_id);
    assert_ne!(run_2w_shifted.seam1_id, run_8w_shifted_rev.seam1_id);

    // Every entity, relation, and group persistent name must be byte-identical.
    for other in [&run_2w_shifted, &run_8w_shifted_rev] {
        assert_eq!(run_1w.face1_name.as_bytes(), other.face1_name.as_bytes());
        assert_eq!(run_1w.face2_name.as_bytes(), other.face2_name.as_bytes());
        assert_eq!(run_1w.seam1_name.as_bytes(), other.seam1_name.as_bytes());
        assert_eq!(run_1w.seam2_name.as_bytes(), other.seam2_name.as_bytes());
        assert_eq!(run_1w.room_name.as_bytes(), other.room_name.as_bytes());
    }
    assert_ne!(run_1w.face1_name, run_1w.face2_name);
    assert_ne!(run_1w.seam1_name, run_1w.seam2_name);
}

#[test]
fn basis_and_same_occurrence_discriminator_reference_resolution_and_forward_refusal() {
    let catalog = lowered_cad_catalog();
    let runtime = fresh_cad_runtime();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);

    // Commit 1: mint root cutter face.
    let f1_ref = face_ref("cutter");
    let tx1 = begin_staged_tx(&runtime, "c1", vec![create_face_intent("cutter")]);
    let mut b1 = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b1.add_entity_birth(ExecutionPendingEntityBirth::new(
        f1_ref.clone(),
        FACE_SCHEME_ID,
        "occ.cutter",
        "authored",
    ))
    .unwrap();
    let c1 = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx1,
        b1.build(),
    )
    .expect("commit 1 succeeds");
    let (c1_res, c1_names) = c1.into_parts();
    let (cutter_entity, cutter_name) = c1_names
        .created_entity_name::<Face>(&c1_res, &f1_ref)
        .unwrap();
    let cutter_rel_id = c1_names.entity_relational_identity(cutter_entity).unwrap();
    release_commit_snapshot(&runtime, &c1_res);

    // Commit 2: cut_piece embeds cutter_name resolved from basis; seam embeds cutter_name too.
    let f2_ref = face_ref("cut-child");
    let s1_ref = seam_ref(
        "seam-ref",
        EntityReference::Existing(cutter_entity),
        EntityReference::Created(f2_ref.clone()),
    );
    let tx2 = begin_staged_tx(
        &runtime,
        "c2",
        vec![
            create_face_intent("cut-child"),
            create_seam_intent(
                "seam-ref",
                EntityReference::Existing(cutter_entity),
                EntityReference::Created(f2_ref.clone()),
            ),
        ],
    );
    let mut b2 = ExecutionNameBirthPlanBuilder::new(&catalog, [cutter_rel_id]);
    b2.add_entity_birth(
        ExecutionPendingEntityBirth::new(f2_ref.clone(), FACE_SCHEME_ID, "occ.cut.1", "cut_piece")
            .with_discriminator(&FaceSplitDisc::Cutter {
                cutter: cutter_name,
                piece: 1,
            }),
    )
    .unwrap();
    b2.add_relation_birth(
        ExecutionPendingRelationBirth::new(s1_ref.clone(), SEAM_SCHEME_ID, "occ.cut.1", "seam")
            .with_discriminator(&SeamSplitDisc::ByFace {
                face: cutter_name,
                index: 1,
            }),
    )
    .unwrap();
    let c2 = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx2,
        b2.build(),
    )
    .expect("basis-referenced discriminator commit succeeds");
    let (c2_res, c2_names) = c2.into_parts();
    let (_, cut_name) = c2_names
        .created_entity_name::<Face>(&c2_res, &f2_ref)
        .unwrap();
    let (_, seam_name) = c2_names
        .created_relation_name::<Seam>(&c2_res, &s1_ref)
        .unwrap();
    assert_ne!(cut_name, cutter_name);
    assert_eq!(seam_name.as_bytes().len(), 32);
    release_commit_snapshot(&runtime, &c2_res);

    // Forward/non-preceding same-occurrence reference (cut_piece -> seam_patch)
    // must be refused fail-closed before commit.
    let mut b_bad = ExecutionNameBirthPlanBuilder::new(&catalog, [cutter_rel_id]);
    let err = b_bad
        .add_entity_birth(
            ExecutionPendingEntityBirth::new(
                face_ref("bad-forward"),
                FACE_SCHEME_ID,
                "occ.bad",
                "cut_piece",
            )
            .with_discriminator(&FaceSplitDisc::Cutter {
                cutter: cutter_name,
                piece: 2,
            })
            .with_same_occurrence_reference(cutter_name, "seam_patch"),
        )
        .err()
        .expect("forward same-occurrence reference must refuse");
    assert!(
        matches!(
            err,
            ExecutionNamingDenial::NonPrecedingSameOccurrenceRole {
                ref scheme_id,
                ref role_path,
                ref referenced_role,
            } if scheme_id == FACE_SCHEME_ID
                && role_path == "cut_piece"
                && referenced_role == "seam_patch"
        ),
        "unexpected denial: {err:?}"
    );

    // Unknown/unborn referenced name (absent from basis_identities) must refuse.
    let mut b_unborn = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    let unborn_err = b_unborn
        .add_entity_birth(
            ExecutionPendingEntityBirth::new(
                face_ref("bad-unborn"),
                FACE_SCHEME_ID,
                "occ.unborn",
                "cut_piece",
            )
            .with_discriminator(&FaceSplitDisc::Cutter {
                cutter: cutter_name,
                piece: 3,
            }),
        )
        .err()
        .expect("unborn referenced name must refuse");
    assert!(
        matches!(
            unborn_err,
            ExecutionNamingDenial::UnresolvedReferencedName {
                ref scheme_id,
                ref role_path,
                digest,
            } if scheme_id == FACE_SCHEME_ID
                && role_path == "cut_piece"
                && digest == *cutter_name.as_bytes()
        ),
        "unexpected denial: {unborn_err:?}"
    );
}
