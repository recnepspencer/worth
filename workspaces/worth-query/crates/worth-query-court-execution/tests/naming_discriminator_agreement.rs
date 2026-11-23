//! Cross-crate agreement court between `worth-query-declaration`'s
//! `ApplicationNameDiscriminatorEncoder` and `worth-relational`'s commit-time
//! canonical discriminator scanner, using real Relational-admitted
//! `WorthQueryName` handles.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{
    ApplicationNameDiscriminator, ApplicationNameDiscriminatorEncoder,
    ApplicationNameDiscriminatorValue, NestedDiscriminator, WorthQueryName,
};
use worth_query_execution_naming::facade::{
    ExecutionNameBirthPlanBuilder, ExecutionNamingCommitAuthority, ExecutionPendingEntityBirth,
};
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::lineage::{
    CommitNamingPlan, DiscriminatorBasisDenialReason, LineageNameDenial, PendingBirthTarget,
    PendingCommitBirth, PersistentNameDiscriminatorReference, PersistentNameReferenceBasis,
};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, CreatedEntityRef, EntitySpec, MutationIntent,
};

use crate::naming_execution_fixture::{
    begin_staged_tx, create_face_intent, execution_lease_for_workers, face_ref, fresh_cad_runtime,
    lowered_cad_catalog, release_commit_snapshot, CadNamingSchema, Edge, Face, EDGE_KIND_ID,
    EDGE_SCHEME_ID, FACE_SCHEME_ID,
};

#[derive(Clone, Debug, Eq, PartialEq)]
enum InnerSplit {
    ByCutter {
        cutter: WorthQueryName<Face>,
        side: u8,
        boundary: Option<WorthQueryName<Edge>>,
    },
}

worth_query_declaration::worth_query_name_discriminator! {
    enum InnerSplit {
        identity: "worth.cad.face.inner_split.v1",
        variants: {
            ByCutter { cutter, side, boundary },
        }
    }
}

fn edge_ref(key: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: EDGE_KIND_ID,
        client_key: ClientKey::raw(key),
    }
}

fn create_edge_intent(key: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(EDGE_KIND_ID.0),
        client_key: ClientKey::raw(key),
        fields: AspectFieldPatch::default(),
    }))
}

#[test]
fn naming_encoder_and_relational_scanner_agree_across_nested_pairs_options_and_tampering() {
    let catalog = lowered_cad_catalog();
    let runtime = fresh_cad_runtime();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);

    // Seed real committed basis faces & edge so all WorthQueryName handles are genuinely admitted.
    let cutter_ref = face_ref("cutter");
    let edge_r = edge_ref("edge");
    let outer_ref = face_ref("outer");
    let tx_seed = begin_staged_tx(
        &runtime,
        "seed",
        vec![
            create_face_intent("cutter"),
            create_edge_intent("edge"),
            create_face_intent("outer"),
        ],
    );
    let mut b_seed = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b_seed
        .add_entity_birth(ExecutionPendingEntityBirth::new(
            cutter_ref.clone(),
            FACE_SCHEME_ID,
            "occ.cutter",
            "authored",
        ))
        .unwrap()
        .add_entity_birth(ExecutionPendingEntityBirth::new(
            edge_r.clone(),
            EDGE_SCHEME_ID,
            "occ.edge",
            "authored",
        ))
        .unwrap()
        .add_entity_birth(ExecutionPendingEntityBirth::new(
            outer_ref.clone(),
            FACE_SCHEME_ID,
            "occ.outer",
            "authored",
        ))
        .unwrap();
    let seeded = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx_seed,
        b_seed.build(),
    )
    .expect("basis seed succeeds");
    let (seed_res, seed_names) = seeded.into_parts();
    let (cutter_e, cutter_name) = seed_names
        .created_entity_name::<CadNamingSchema, Face>(&seed_res, &cutter_ref)
        .unwrap();
    let (edge_e, edge_name) = seed_names
        .created_entity_name::<CadNamingSchema, Edge>(&seed_res, &edge_r)
        .unwrap();
    let (outer_e, outer_name) = seed_names
        .created_entity_name::<CadNamingSchema, Face>(&seed_res, &outer_ref)
        .unwrap();
    let cutter_id = seed_names.entity_relational_identity(cutter_e).unwrap();
    let edge_id = seed_names.entity_relational_identity(edge_e).unwrap();
    let outer_ref_id = seed_names.entity_relational_identity(outer_e).unwrap();
    release_commit_snapshot(&runtime, &seed_res);

    let inner = InnerSplit::ByCutter {
        cutter: cutter_name,
        side: 2,
        boundary: Some(edge_name),
    };
    let mut encoder = ApplicationNameDiscriminatorEncoder::new("worth.cad.face.outer.v1");
    encoder
        .variant("NestedCut")
        .field("pair", &(NestedDiscriminator(inner), 42_u32))
        .field("absent_opt", &Option::<WorthQueryName<Edge>>::None)
        .field("outer_opt", &Some(outer_name))
        .field("flag", &true)
        .field("label", &"top-cap")
        .field("uuid", &[0x5a_u8; 16]);
    let (encoded_bytes, encoded_refs) = encoder.finish_with_references();

    assert_eq!(
        encoded_refs,
        vec![
            *cutter_id.as_bytes(),
            *edge_id.as_bytes(),
            *outer_ref_id.as_bytes(),
        ]
    );

    let declared_refs: Vec<PersistentNameDiscriminatorReference> =
        [cutter_id, edge_id, outer_ref_id]
            .into_iter()
            .map(|identity| PersistentNameDiscriminatorReference {
                identity,
                basis: PersistentNameReferenceBasis::Basis,
            })
            .collect();

    // 1. Exact agreement: real encoder output + matching declared references commits cleanly.
    let ok_ref = face_ref("nested-ok");
    let mut tx_ok = begin_staged_tx(&runtime, "nested-ok", vec![create_face_intent("nested-ok")]);
    tx_ok.attach_naming_plan(CommitNamingPlan {
        pending_births: vec![PendingCommitBirth {
            target: PendingBirthTarget::CreatedEntity(ok_ref.clone()),
            scheme_id: FACE_SCHEME_ID.to_owned(),
            occurrence_key: "occ.split.1".to_owned(),
            role_path: "cut_piece".to_owned(),
            parent_identities: vec![cutter_id],
            referenced_names: declared_refs.clone(),
            canonical_discriminator: encoded_bytes.clone(),
        }],
        ..Default::default()
    });
    let ok_res = runtime
        .commit_branch_transaction(&execution, tx_ok)
        .expect("real encoder output with declared references must commit cleanly");
    let ok_births = ok_res.committed_name_births();
    assert_eq!(ok_births.len(), 1);
    assert_eq!(
        ok_births[0].1.parts().canonical_discriminator(),
        encoded_bytes.as_slice()
    );
    release_commit_snapshot(&runtime, &ok_res);

    // 2. Omitting ANY embedded reference refuses with exact digest.
    for (omit_idx, omitted_id) in [cutter_id, edge_id, outer_ref_id].into_iter().enumerate() {
        let partial: Vec<_> = declared_refs
            .iter()
            .enumerate()
            .filter(|&(idx, _)| idx != omit_idx)
            .map(|(_, entry)| entry.clone())
            .collect();
        let key = format!("omit-{omit_idx}");
        let mut tx = begin_staged_tx(&runtime, &key, vec![create_face_intent(&key)]);
        tx.attach_naming_plan(CommitNamingPlan {
            pending_births: vec![PendingCommitBirth {
                target: PendingBirthTarget::CreatedEntity(face_ref(&key)),
                scheme_id: FACE_SCHEME_ID.to_owned(),
                occurrence_key: format!("occ.omit.{omit_idx}"),
                role_path: "cut_piece".to_owned(),
                parent_identities: vec![cutter_id],
                referenced_names: partial,
                canonical_discriminator: encoded_bytes.clone(),
            }],
            ..Default::default()
        });
        let denial = runtime
            .commit_branch_transaction(&execution, tx)
            .expect_err("omitting any embedded name reference must refuse");
        assert_eq!(
            denial.lineage_name_denial_ref(),
            Some(&LineageNameDenial::InvalidDiscriminatorBasis {
                scheme_id: FACE_SCHEME_ID.to_owned(),
                role_path: "cut_piece".to_owned(),
                reason: DiscriminatorBasisDenialReason::UndeclaredEncodedNameReference {
                    raw_digest: *omitted_id.as_bytes(),
                },
            })
        );
    }

    // 3. Truncation, trailing byte, and v2 header swap all refuse with MalformedDiscriminator.
    let mut v2_swapped = encoded_bytes.clone();
    let v1_pos = v2_swapped
        .windows(4)
        .position(|w| w == b".v1\0")
        .expect("v1 header present");
    v2_swapped[v1_pos + 2] = b'2';
    let mut trailing_byte = encoded_bytes.clone();
    trailing_byte.push(0x00);
    let truncated = encoded_bytes[..encoded_bytes.len() - 1].to_vec();

    for (label, tampered) in [
        ("v2_header", v2_swapped),
        ("trailing", trailing_byte),
        ("truncated", truncated),
    ] {
        let key = format!("tamper-{label}");
        let mut tx = begin_staged_tx(&runtime, &key, vec![create_face_intent(&key)]);
        tx.attach_naming_plan(CommitNamingPlan {
            pending_births: vec![PendingCommitBirth {
                target: PendingBirthTarget::CreatedEntity(face_ref(&key)),
                scheme_id: FACE_SCHEME_ID.to_owned(),
                occurrence_key: format!("occ.tamper.{label}"),
                role_path: "cut_piece".to_owned(),
                parent_identities: vec![cutter_id],
                referenced_names: Vec::new(),
                canonical_discriminator: tampered,
            }],
            ..Default::default()
        });
        let denial = runtime
            .commit_branch_transaction(&execution, tx)
            .expect_err("tampered discriminator must refuse");
        assert_eq!(
            denial.lineage_name_denial_ref(),
            Some(&LineageNameDenial::InvalidDiscriminatorBasis {
                scheme_id: FACE_SCHEME_ID.to_owned(),
                role_path: "cut_piece".to_owned(),
                reason: DiscriminatorBasisDenialReason::MalformedDiscriminator,
            })
        );
    }

    // 4. Deep Option nesting (Option^70<u32>) produced by the real encoder commits cleanly.
    type Opt10<T> =
        Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<T>>>>>>>>>>;
    type Opt70<T> = Opt10<Opt10<Opt10<Opt10<Opt10<Opt10<Opt10<T>>>>>>>;
    fn wrap10<T>(v: T) -> Opt10<T> {
        Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(v))))))))))
    }
    let deep_val: Opt70<u32> = wrap10(wrap10(wrap10(wrap10(wrap10(wrap10(wrap10(7_u32)))))));
    let mut deep_enc = ApplicationNameDiscriminatorEncoder::new("worth.cad.face.deep.v1");
    deep_enc.field("depth70", &deep_val);
    let (deep_bytes, deep_refs) = deep_enc.finish_with_references();
    assert!(deep_refs.is_empty());
    let mut tx_deep = begin_staged_tx(&runtime, "deep70", vec![create_face_intent("deep70")]);
    tx_deep.attach_naming_plan(CommitNamingPlan {
        pending_births: vec![PendingCommitBirth {
            target: PendingBirthTarget::CreatedEntity(face_ref("deep70")),
            scheme_id: FACE_SCHEME_ID.to_owned(),
            occurrence_key: "occ.deep70".to_owned(),
            role_path: "cut_piece".to_owned(),
            parent_identities: vec![cutter_id],
            referenced_names: Vec::new(),
            canonical_discriminator: deep_bytes,
        }],
        ..Default::default()
    });
    let deep_res = runtime
        .commit_branch_transaction(&execution, tx_deep)
        .expect("real encoder Option^70<u32> must commit cleanly");
    assert_eq!(deep_res.committed_name_births().len(), 1);
    release_commit_snapshot(&runtime, &deep_res);

    let _check_trait: fn(&u32, &mut ApplicationNameDiscriminatorEncoder) =
        <u32 as ApplicationNameDiscriminatorValue>::encode_discriminator_value;
    let _check_inner: fn(&InnerSplit) -> Vec<u8> = InnerSplit::canonical_bytes;
}
