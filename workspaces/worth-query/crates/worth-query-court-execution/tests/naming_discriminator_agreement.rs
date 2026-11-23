//! Cross-crate agreement court between `worth-query-declaration`'s
//! `ApplicationNameDiscriminatorEncoder` and `worth-relational`'s
//! `PersistentNameMintingAuthority` canonical discriminator scanner.

use std::collections::{BTreeMap, BTreeSet};

use worth_query_declaration::facade::application_naming::{
    ApplicationNameDiscriminator, ApplicationNameDiscriminatorEncoder,
    ApplicationNameDiscriminatorValue, NestedDiscriminator, WorthQueryName,
};
use worth_query_execution_naming::facade::admit_minted_birth_record;
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::lineage::{
    DiscriminatorBasisDenialReason, InstalledPersistentNameScheme,
    InstalledPersistentNameSchemeRegistry, LineageNameDenial, PersistentNameBirthRecord,
    PersistentNameBirthRequest, PersistentNameDiscriminatorReference, PersistentNameHolder,
    PersistentNameMintingAuthority, PersistentNameReferenceBasis, PersistentNameSubjectKind,
};

struct Face;
struct Edge;

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

fn face_registry() -> InstalledPersistentNameSchemeRegistry {
    let mut registry = InstalledPersistentNameSchemeRegistry::new();
    registry
        .register(InstalledPersistentNameScheme::from_ordered_roles(
            "worth.cad.face.v1",
            PersistentNameSubjectKind::Entity,
            ["authored", "cut_piece"],
            true,
            true,
        ))
        .expect("register face scheme");
    registry
}

fn entity_holder(slot: u64) -> PersistentNameHolder {
    PersistentNameHolder::Entity(EntityId::new(PartitionId::main(), slot, 1))
}

fn mint_basis_identity(
    registry: &InstalledPersistentNameSchemeRegistry,
    occ: &str,
    slot: u64,
) -> (PersistentNameBirthRecord, PersistentNameHolder) {
    let holder = entity_holder(slot);
    let records = PersistentNameMintingAuthority::mint_commit_births(
        registry,
        &BTreeMap::new(),
        &BTreeSet::new(),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: occ.to_owned(),
            role_path: "authored".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: Vec::new(),
            canonical_discriminator: Vec::new(),
            holder: holder.clone(),
        }],
    )
    .expect("basis birth succeeds");
    (records.into_iter().next().unwrap(), holder)
}

#[test]
fn naming_encoder_and_relational_scanner_agree_across_nested_pairs_options_and_tampering() {
    let registry = face_registry();
    let (cutter_rec, cutter_holder) = mint_basis_identity(&registry, "occ.cutter", 1);
    let (edge_rec, edge_holder) = mint_basis_identity(&registry, "occ.edge", 2);
    let (outer_rec, outer_ref_holder) = mint_basis_identity(&registry, "occ.outer", 3);
    let (cutter_id, edge_id, outer_ref_id) = (
        cutter_rec.identity(),
        edge_rec.identity(),
        outer_rec.identity(),
    );

    let mut live_holders = BTreeMap::new();
    live_holders.insert(cutter_id, cutter_holder);
    live_holders.insert(edge_id, edge_holder);
    live_holders.insert(outer_ref_id, outer_ref_holder);

    let cutter_name: WorthQueryName<Face> =
        admit_minted_birth_record(&cutter_rec, PersistentNameSubjectKind::Entity).unwrap();
    let edge_name: WorthQueryName<Edge> =
        admit_minted_birth_record(&edge_rec, PersistentNameSubjectKind::Entity).unwrap();
    let outer_name: WorthQueryName<Face> =
        admit_minted_birth_record(&outer_rec, PersistentNameSubjectKind::Entity).unwrap();

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

    // 1. Exact agreement: real encoder output + matching declared references mints cleanly.
    let minted = PersistentNameMintingAuthority::mint_commit_births(
        &registry,
        &live_holders,
        &BTreeSet::new(),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: "occ.split.1".to_owned(),
            role_path: "cut_piece".to_owned(),
            parent_identities: vec![cutter_id],
            referenced_names: declared_refs.clone(),
            canonical_discriminator: encoded_bytes.clone(),
            holder: entity_holder(10),
        }],
    )
    .expect("real encoder output with declared references must mint cleanly");
    assert_eq!(minted.len(), 1);
    assert_eq!(
        minted[0].parts().canonical_discriminator(),
        encoded_bytes.as_slice()
    );

    // 2. Omitting ANY embedded reference (including inside NestedDiscriminator) refuses with exact digest.
    for (omit_idx, omitted_id) in [cutter_id, edge_id, outer_ref_id].into_iter().enumerate() {
        let partial: Vec<_> = declared_refs
            .iter()
            .enumerate()
            .filter(|&(idx, _)| idx != omit_idx)
            .map(|(_, entry)| entry.clone())
            .collect();
        let denial = PersistentNameMintingAuthority::mint_commit_births(
            &registry,
            &live_holders,
            &BTreeSet::new(),
            &[PersistentNameBirthRequest {
                scheme_id: "worth.cad.face.v1".to_owned(),
                occurrence_key: format!("occ.omit.{omit_idx}"),
                role_path: "cut_piece".to_owned(),
                parent_identities: vec![cutter_id],
                referenced_names: partial,
                canonical_discriminator: encoded_bytes.clone(),
                holder: entity_holder(20 + omit_idx as u64),
            }],
        )
        .expect_err("omitting any embedded name reference must refuse");
        assert_eq!(
            denial,
            LineageNameDenial::InvalidDiscriminatorBasis {
                scheme_id: "worth.cad.face.v1".to_owned(),
                role_path: "cut_piece".to_owned(),
                reason: DiscriminatorBasisDenialReason::UndeclaredEncodedNameReference {
                    raw_digest: *omitted_id.as_bytes(),
                },
            }
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
        let denial = PersistentNameMintingAuthority::mint_commit_births(
            &registry,
            &live_holders,
            &BTreeSet::new(),
            &[PersistentNameBirthRequest {
                scheme_id: "worth.cad.face.v1".to_owned(),
                occurrence_key: format!("occ.tamper.{label}"),
                role_path: "cut_piece".to_owned(),
                parent_identities: vec![cutter_id],
                referenced_names: Vec::new(),
                canonical_discriminator: tampered,
                holder: entity_holder(30),
            }],
        )
        .expect_err("tampered discriminator must refuse");
        assert_eq!(
            denial,
            LineageNameDenial::InvalidDiscriminatorBasis {
                scheme_id: "worth.cad.face.v1".to_owned(),
                role_path: "cut_piece".to_owned(),
                reason: DiscriminatorBasisDenialReason::MalformedDiscriminator,
            }
        );
    }

    // 4. Deep Option nesting (Option^70<u32>) produced by the real encoder parses cleanly.
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
    let deep_minted = PersistentNameMintingAuthority::mint_commit_births(
        &registry,
        &live_holders,
        &BTreeSet::new(),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: "occ.deep70".to_owned(),
            role_path: "cut_piece".to_owned(),
            parent_identities: vec![cutter_id],
            referenced_names: Vec::new(),
            canonical_discriminator: deep_bytes,
            holder: entity_holder(40),
        }],
    )
    .expect("real encoder Option^70<u32> must mint cleanly");
    assert_eq!(deep_minted.len(), 1);
    let _check_trait: fn(&u32, &mut ApplicationNameDiscriminatorEncoder) =
        <u32 as ApplicationNameDiscriminatorValue>::encode_discriminator_value;
    let _check_inner: fn(&InnerSplit) -> Vec<u8> = InnerSplit::canonical_bytes;
}
