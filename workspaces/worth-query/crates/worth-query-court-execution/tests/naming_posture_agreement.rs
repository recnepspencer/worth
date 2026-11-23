//! Cross-crate agreement court between `worth-query-declaration`'s Phase 2
//! lineage output postures (`WorthQueryLineageOutputPosture`,
//! `WorthQueryMergeKeeper`, `ErasedLineageOutputPosture`) and
//! `worth-relational`'s persistent naming & discriminator authority across both
//! entities and parallel same-role relations.

use std::collections::{BTreeMap, BTreeSet};

use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_query_declaration::facade::application_operation::{
    ErasedLineageOutputPosture, ErasedMergeKeeper, WorthQueryLineageOutputPosture,
    WorthQueryLineagePostureDenial, WorthQueryMergeKeeper,
};
use worth_query_execution_naming::facade::admit_minted_birth_record;
use worth_relational::facade::identity::{PartitionId, RelationId};
use worth_relational::facade::lineage::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry,
    PersistentNameBirthRequest, PersistentNameDiscriminatorReference, PersistentNameHolder,
    PersistentNameIdentity, PersistentNameMintingAuthority, PersistentNameReferenceBasis,
    PersistentNameSubjectKind,
};

struct Face;
struct Seam;

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

fn registry() -> InstalledPersistentNameSchemeRegistry {
    let mut reg = InstalledPersistentNameSchemeRegistry::new();
    reg.register(InstalledPersistentNameScheme::from_ordered_roles(
        "worth.cad.face.v1",
        PersistentNameSubjectKind::Entity,
        ["authored"],
        true,
        true,
    ))
    .unwrap();
    reg.register(InstalledPersistentNameScheme::from_ordered_roles(
        "worth.cad.seam.v1",
        PersistentNameSubjectKind::Relation,
        ["seam", "seam_piece", "merged_seam"],
        true,
        true,
    ))
    .unwrap();
    reg
}

fn rel_holder(slot: u64) -> PersistentNameHolder {
    PersistentNameHolder::Relation(RelationId::new(PartitionId::main(), slot, 1))
}

#[test]
fn declaration_split_and_merge_postures_erase_and_mint_with_relational_authority() {
    let reg = registry();

    // 1. Mint a basis cutter face and two parallel seam relations.
    let face_rec = PersistentNameMintingAuthority::mint_commit_births(
        &reg,
        &BTreeMap::new(),
        &BTreeSet::new(),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: "occ.face.1".to_owned(),
            role_path: "authored".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: Vec::new(),
            canonical_discriminator: Vec::new(),
            holder: PersistentNameHolder::PendingBirth {
                occurrence_key: "occ.face.1".to_owned(),
                role_path: "authored".to_owned(),
                ordinal: 0,
            },
        }],
    )
    .unwrap()[0]
        .clone();
    let cutter_id = face_rec.identity();
    let cutter_name: WorthQueryName<Face> =
        admit_minted_birth_record(&face_rec, PersistentNameSubjectKind::Entity).unwrap();

    let seam1_disc = SeamSplitDiscriminator::AtFace {
        cutter: cutter_name,
        piece: 1,
    };
    let seam2_disc = SeamSplitDiscriminator::AtFace {
        cutter: cutter_name,
        piece: 2,
    };
    let (s1_bytes, s1_refs) =
        worth_query_declaration::facade::application_naming::ApplicationNameDiscriminator::canonical_bytes_with_references(&seam1_disc);
    let (s2_bytes, s2_refs) =
        worth_query_declaration::facade::application_naming::ApplicationNameDiscriminator::canonical_bytes_with_references(&seam2_disc);
    assert_eq!(s1_refs, vec![*cutter_id.as_bytes()]);
    assert_eq!(s2_refs, vec![*cutter_id.as_bytes()]);

    let mut live_holders = BTreeMap::from([(
        cutter_id,
        PersistentNameHolder::PendingBirth {
            occurrence_key: "occ.face.1".to_owned(),
            role_path: "authored".to_owned(),
            ordinal: 0,
        },
    )]);

    let seams = PersistentNameMintingAuthority::mint_commit_births(
        &reg,
        &live_holders,
        &BTreeSet::new(),
        &[
            PersistentNameBirthRequest {
                scheme_id: "worth.cad.seam.v1".to_owned(),
                occurrence_key: "occ.seams".to_owned(),
                role_path: "seam".to_owned(),
                parent_identities: Vec::new(),
                referenced_names: vec![PersistentNameDiscriminatorReference {
                    identity: cutter_id,
                    basis: PersistentNameReferenceBasis::Basis,
                }],
                canonical_discriminator: s1_bytes,
                holder: rel_holder(1),
            },
            PersistentNameBirthRequest {
                scheme_id: "worth.cad.seam.v1".to_owned(),
                occurrence_key: "occ.seams".to_owned(),
                role_path: "seam".to_owned(),
                parent_identities: Vec::new(),
                referenced_names: vec![PersistentNameDiscriminatorReference {
                    identity: cutter_id,
                    basis: PersistentNameReferenceBasis::Basis,
                }],
                canonical_discriminator: s2_bytes,
                holder: rel_holder(2),
            },
        ],
    )
    .expect("parallel seam births mint distinct names");
    let seam1_id = seams[0].identity();
    let seam2_id = seams[1].identity();
    assert_ne!(seam1_id, seam2_id);
    live_holders.insert(seam1_id, rel_holder(1));
    live_holders.insert(seam2_id, rel_holder(2));

    let seam1_name: WorthQueryName<Seam> =
        admit_minted_birth_record(&seams[0], PersistentNameSubjectKind::Relation).unwrap();
    let seam2_name: WorthQueryName<Seam> =
        admit_minted_birth_record(&seams[1], PersistentNameSubjectKind::Relation).unwrap();

    // 2. Lower a typed Split posture on seam 1 and mint its successor through Relational.
    let split_posture = WorthQueryLineageOutputPosture::Split {
        parent: seam1_name,
        discriminator: SeamSplitDiscriminator::AtFace {
            cutter: cutter_name,
            piece: 10,
        },
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

    let split_child = PersistentNameMintingAuthority::mint_commit_births(
        &reg,
        &live_holders,
        &BTreeSet::from([seam1_id]),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.seam.v1".to_owned(),
            occurrence_key: "occ.split".to_owned(),
            role_path: "seam_piece".to_owned(),
            parent_identities: vec![seam1_id],
            referenced_names: vec![PersistentNameDiscriminatorReference {
                identity: cutter_id,
                basis: PersistentNameReferenceBasis::Basis,
            }],
            canonical_discriminator,
            holder: rel_holder(3),
        }],
    )
    .expect("lowered Split discriminator passes Relational scanner and mints");
    let split_child_id = split_child[0].identity();
    assert_ne!(split_child_id, seam1_id);
    assert_ne!(split_child_id, seam2_id);
    live_holders.remove(&seam1_id);
    live_holders.insert(split_child_id, rel_holder(3));

    // 3. Lower a typed Merge(MintNew) posture across (split_child, seam2) and mint via Relational.
    let split_child_name: WorthQueryName<Seam> =
        admit_minted_birth_record(&split_child[0], PersistentNameSubjectKind::Relation).unwrap();
    let merge_posture = WorthQueryLineageOutputPosture::Merge {
        parents: vec![split_child_name, seam2_name],
        keeper: WorthQueryMergeKeeper::MintNew {
            discriminator: SeamSplitDiscriminator::MergedSeam {
                label: "reunited".to_owned(),
            },
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

    let merged = PersistentNameMintingAuthority::mint_commit_births(
        &reg,
        &live_holders,
        &BTreeSet::from([split_child_id, seam2_id]),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.seam.v1".to_owned(),
            occurrence_key: "occ.merge".to_owned(),
            role_path: "merged_seam".to_owned(),
            parent_identities: expected_parents.clone(),
            referenced_names: Vec::new(),
            canonical_discriminator: merge_disc,
            holder: rel_holder(4),
        }],
    )
    .expect("lowered Merge(MintNew) mints cleanly");
    assert_eq!(
        merged[0].parts().parent_identities(),
        expected_parents.as_slice()
    );

    // 4. Lowering a KeepParent outside parents refuses at the declaration boundary.
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
