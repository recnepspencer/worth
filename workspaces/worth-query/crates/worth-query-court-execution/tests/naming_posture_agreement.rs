//! Cross-crate agreement court between `worth-query-declaration`'s Phase 2
//! lineage output postures (`WorthQueryLineageOutputPosture`,
//! `WorthQueryMergeKeeper`, `ErasedLineageOutputPosture`) and
//! `worth-relational`'s commit-time persistent naming & posture authority
//! across entities and parallel same-role relations.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_query_declaration::facade::application_operation::{
    ErasedLineageOutputPosture, ErasedMergeKeeper, WorthQueryLineageOutputPosture,
    WorthQueryLineagePostureDenial, WorthQueryMergeKeeper,
};
use worth_query_execution_naming::facade::{
    ExecutionNameBirthPlanBuilder, ExecutionNamedMergeKeeper, ExecutionNamingCommitAuthority,
    ExecutionPendingEntityBirth, ExecutionPendingRelationBirth, ExecutionSplitChildSpec,
};
use worth_relational::facade::lineage::{
    LineageEventKind, LineageRecordTarget, PendingBirthTarget, PendingMergeTarget,
    PersistentNameIdentity,
};
use worth_relational::facade::transactions::{
    DeleteRelationIntent, EntityReference, MutationIntent, RelationMutationIntent,
};

use crate::naming_execution_fixture::{
    begin_staged_tx, create_face_intent, create_seam_intent, execution_lease_for_workers, face_ref,
    fresh_cad_runtime, lowered_cad_catalog, release_commit_snapshot, seam_ref, CadNamingSchema,
    Face, Seam, FACE_SCHEME_ID, SEAM_SCHEME_ID,
};

#[derive(Clone, Debug, Eq, PartialEq)]
enum SeamSplitDiscriminator {
    AtFace {
        cutter: WorthQueryName<Face>,
        piece: u8,
    },
    MergedSeam {
        label: String,
    },
}

worth_query_declaration::worth_query_name_discriminator! {
    enum SeamSplitDiscriminator {
        identity: "worth.cad.seam.split_disc.v1",
        variants: {
            AtFace { cutter, piece },
            MergedSeam { label },
        }
    }
}

#[test]
fn declaration_split_and_merge_postures_erase_and_commit_with_relational_authority() {
    let catalog = lowered_cad_catalog();
    let runtime = fresh_cad_runtime();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);

    // 1. Commit a basis cutter face and a partner face.
    let f1_ref = face_ref("cutter");
    let f2_ref = face_ref("other");
    let tx_faces = begin_staged_tx(
        &runtime,
        "faces",
        vec![create_face_intent("cutter"), create_face_intent("other")],
    );
    let mut b_faces = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b_faces
        .add_entity_birth(ExecutionPendingEntityBirth::new(
            f1_ref.clone(),
            FACE_SCHEME_ID,
            "occ.face.1",
            "authored",
        ))
        .unwrap()
        .add_entity_birth(ExecutionPendingEntityBirth::new(
            f2_ref.clone(),
            FACE_SCHEME_ID,
            "occ.face.2",
            "authored",
        ))
        .unwrap();
    let (faces_res, faces_names) = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_faces,
        b_faces.build(),
    )
    .expect("face seed succeeds")
    .into_parts();
    let (cutter_e, cutter_name) = faces_names
        .created_entity_name::<CadNamingSchema, Face>(&faces_res, &f1_ref)
        .unwrap();
    let (other_e, _) = faces_names
        .created_entity_name::<CadNamingSchema, Face>(&faces_res, &f2_ref)
        .unwrap();
    let cutter_id = faces_names.entity_relational_identity(cutter_e).unwrap();
    release_commit_snapshot(&runtime, &faces_res);

    // 2. Commit two parallel seam relations referencing cutter_name.
    let s1_ref = seam_ref(
        "s1",
        EntityReference::Existing(cutter_e),
        EntityReference::Existing(other_e),
    );
    let s2_ref = seam_ref(
        "s2",
        EntityReference::Existing(other_e),
        EntityReference::Existing(cutter_e),
    );
    let tx_seams = begin_staged_tx(
        &runtime,
        "seams",
        vec![
            create_seam_intent(
                "s1",
                EntityReference::Existing(cutter_e),
                EntityReference::Existing(other_e),
            ),
            create_seam_intent(
                "s2",
                EntityReference::Existing(other_e),
                EntityReference::Existing(cutter_e),
            ),
        ],
    );
    let mut b_seams = ExecutionNameBirthPlanBuilder::new(&catalog, [cutter_id]);
    b_seams
        .add_relation_birth(
            ExecutionPendingRelationBirth::new(s1_ref.clone(), SEAM_SCHEME_ID, "occ.seams", "seam")
                .with_discriminator(&SeamSplitDiscriminator::AtFace {
                    cutter: cutter_name,
                    piece: 1,
                }),
        )
        .unwrap()
        .add_relation_birth(
            ExecutionPendingRelationBirth::new(s2_ref.clone(), SEAM_SCHEME_ID, "occ.seams", "seam")
                .with_discriminator(&SeamSplitDiscriminator::AtFace {
                    cutter: cutter_name,
                    piece: 2,
                }),
        )
        .unwrap();
    let (seams_res, seams_names) = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_seams,
        b_seams.build(),
    )
    .expect("parallel seam births commit cleanly")
    .into_parts();
    let (s1_rel, seam1_name) = seams_names
        .created_relation_name::<Seam>(&seams_res, &s1_ref)
        .unwrap();
    let (s2_rel, seam2_name) = seams_names
        .created_relation_name::<Seam>(&seams_res, &s2_ref)
        .unwrap();
    let seam1_id = seams_names.relation_relational_identity(s1_rel).unwrap();
    let seam2_id = seams_names.relation_relational_identity(s2_rel).unwrap();
    assert_ne!(seam1_id, seam2_id);
    release_commit_snapshot(&runtime, &seams_res);

    // 3. Lower a typed Split posture on seam 1 and commit through Relational.
    let split_disc = SeamSplitDiscriminator::AtFace {
        cutter: cutter_name,
        piece: 10,
    };
    let split_posture = WorthQueryLineageOutputPosture::Split {
        parent: seam1_name,
        discriminator: split_disc.clone(),
    };
    let ErasedLineageOutputPosture::Split {
        parent_digest,
        canonical_discriminator,
        referenced_name_digests,
    } = split_posture
        .to_erased_posture()
        .expect("split lowers cleanly")
    else {
        panic!("expected Split");
    };
    assert_eq!(parent_digest, *seam1_id.as_bytes());
    assert_eq!(referenced_name_digests, vec![*cutter_id.as_bytes()]);

    let split_ref = seam_ref(
        "s1-split",
        EntityReference::Existing(cutter_e),
        EntityReference::Existing(other_e),
    );
    let tx_split = begin_staged_tx(
        &runtime,
        "split",
        vec![
            MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: s1_rel,
            })),
            create_seam_intent(
                "s1-split",
                EntityReference::Existing(cutter_e),
                EntityReference::Existing(other_e),
            ),
        ],
    );
    let mut b_split = ExecutionNameBirthPlanBuilder::new(&catalog, [cutter_id, seam1_id]);
    b_split
        .add_split_posture(
            LineageRecordTarget::Relation(s1_rel),
            seam1_id,
            vec![
                ExecutionSplitChildSpec::relation(
                    split_ref.clone(),
                    SEAM_SCHEME_ID,
                    "occ.split",
                    "seam_piece",
                )
                .with_discriminator(&split_disc),
            ],
        )
        .unwrap();
    let (split_res, split_names) = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_split,
        b_split.build(),
    )
    .expect("lowered Split discriminator passes Relational scanner and commits")
    .into_parts();
    assert!(split_res
        .envelope()
        .lineage_events()
        .iter()
        .any(|e| e.kind() == LineageEventKind::Split));
    let (split_rel, split_child_name) = split_names
        .created_relation_name::<Seam>(&split_res, &split_ref)
        .unwrap();
    assert_eq!(
        split_names
            .relation_birth(split_rel)
            .unwrap()
            .parts()
            .canonical_discriminator(),
        canonical_discriminator.as_slice()
    );
    let split_child_id = split_names.relation_relational_identity(split_rel).unwrap();
    assert_ne!(split_child_id, seam1_id);
    assert_ne!(split_child_id, seam2_id);
    release_commit_snapshot(&runtime, &split_res);

    // 4. Lower a typed Merge(MintNew) posture across (split_child, seam2) and commit via Relational.
    let merge_disc_val = SeamSplitDiscriminator::MergedSeam {
        label: "reunited".to_owned(),
    };
    let merge_posture = WorthQueryLineageOutputPosture::Merge {
        parents: vec![split_child_name, seam2_name],
        keeper: WorthQueryMergeKeeper::MintNew {
            discriminator: merge_disc_val.clone(),
        },
    };
    let ErasedLineageOutputPosture::Merge {
        parent_digests,
        keeper:
            ErasedMergeKeeper::MintNew {
                canonical_discriminator: merge_disc,
                referenced_name_digests: merge_refs,
            },
    } = merge_posture
        .to_erased_posture()
        .expect("merge lowers cleanly")
    else {
        panic!("expected Merge(MintNew)");
    };
    assert!(merge_refs.is_empty());
    let mut expected_parents: Vec<PersistentNameIdentity> = vec![split_child_id, seam2_id];
    expected_parents.sort_unstable();
    let expected_digests: Vec<[u8; 32]> = expected_parents.iter().map(|p| *p.as_bytes()).collect();
    assert_eq!(parent_digests, expected_digests);

    let merged_ref = seam_ref(
        "s-merged",
        EntityReference::Existing(cutter_e),
        EntityReference::Existing(other_e),
    );
    let tx_merge = begin_staged_tx(
        &runtime,
        "merge",
        vec![
            MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: split_rel,
            })),
            MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: s2_rel,
            })),
            create_seam_intent(
                "s-merged",
                EntityReference::Existing(cutter_e),
                EntityReference::Existing(other_e),
            ),
        ],
    );
    let mut b_merge = ExecutionNameBirthPlanBuilder::new(&catalog, [split_child_id, seam2_id]);
    b_merge
        .add_merge_posture(
            vec![
                (LineageRecordTarget::Relation(split_rel), split_child_id),
                (LineageRecordTarget::Relation(s2_rel), seam2_id),
            ],
            PendingMergeTarget::Created(PendingBirthTarget::CreatedRelation(merged_ref.clone())),
            ExecutionNamedMergeKeeper::mint_new(
                SEAM_SCHEME_ID,
                "occ.merge",
                "seam_piece",
                &merge_disc_val,
            ),
        )
        .unwrap();
    let (merge_res, merge_names) = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_merge,
        b_merge.build(),
    )
    .expect("lowered Merge(MintNew) commits cleanly")
    .into_parts();
    let (merged_rel, _) = merge_names
        .created_relation_name::<Seam>(&merge_res, &merged_ref)
        .unwrap();
    let merged_birth = merge_names.relation_birth(merged_rel).unwrap();
    assert_eq!(merged_birth.parts().parent_identities(), expected_parents.as_slice());
    assert_eq!(merged_birth.parts().canonical_discriminator(), merge_disc.as_slice());
    release_commit_snapshot(&runtime, &merge_res);

    // 5. Lowering a KeepParent outside parents refuses at the declaration boundary.
    let invalid_merge = WorthQueryLineageOutputPosture::<Seam, SeamSplitDiscriminator>::Merge {
        parents: vec![split_child_name, seam2_name],
        keeper: WorthQueryMergeKeeper::KeepParent(seam1_name),
    };
    assert_eq!(
        invalid_merge.to_erased_posture(),
        Err(WorthQueryLineagePostureDenial::InvalidMergeKeeper {
            keeper_hex: seam1_name.render_hex(),
        })
    );
}
