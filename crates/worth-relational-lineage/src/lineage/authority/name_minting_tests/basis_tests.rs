use crate::contracts::lineage::data::naming::{
    PersistentNameBirthRequest, PersistentNameDiscriminatorReference, PersistentNameIdentity,
    PersistentNameIdentityOwnerPort as _, PersistentNameReferenceBasis,
};
use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::lineage::data::naming_denial::{
    DiscriminatorBasisDenialReason, LineageNameDenial,
};
use crate::lineage::authority::PersistentNameMintingAuthority;

use super::{canonical_label_discriminator, entity_holder, face_registry};

/// Builds a domain-tagged canonical discriminator payload embedding one
/// `TAG_NAME_REF` (0x20) 32-byte digest matching `ApplicationNameDiscriminatorEncoder`.
pub(super) fn encoded_name_ref_discriminator(identity: &PersistentNameIdentity) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"worth.query.name-discriminator.v1\0");
    let disc_id = b"worth.cad.face.split.v1";
    bytes.extend_from_slice(&(disc_id.len() as u64).to_be_bytes());
    bytes.extend_from_slice(disc_id);
    // TAG_FIELD (0x02) + len("cutter") + "cutter" + TAG_NAME_REF (0x20) + 32 bytes
    bytes.push(0x02);
    bytes.extend_from_slice(&(6u64).to_be_bytes());
    bytes.extend_from_slice(b"cutter");
    bytes.push(0x20);
    bytes.extend_from_slice(identity.as_bytes());
    bytes
}

#[test]
fn discriminator_basis_refuses_absent_basis_name_unborn_parent_and_smuggled_ref() {
    let registry = face_registry();
    let unborn = PersistentNameIdentity::from_minted_digest([0xde; 32]);

    // 1. Referencing an absent/unborn name as Basis refuses AbsentBasisName.
    let absent_basis = PersistentNameMintingAuthority::mint_commit_births_for_test(
        &registry,
        &BTreeMap::new(),
        &BTreeSet::new(),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: "occ.absent".to_owned(),
            role_path: "cut_piece".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: vec![PersistentNameDiscriminatorReference {
                identity: unborn,
                basis: PersistentNameReferenceBasis::Basis,
            }],
            canonical_discriminator: encoded_name_ref_discriminator(&unborn),
            holder: entity_holder(1, 1),
        }],
    )
    .expect_err("absent basis name must refuse");
    assert_eq!(
        absent_basis,
        LineageNameDenial::InvalidDiscriminatorBasis {
            scheme_id: "worth.cad.face.v1".to_owned(),
            role_path: "cut_piece".to_owned(),
            reason: DiscriminatorBasisDenialReason::AbsentBasisName { identity: unborn },
        }
    );

    // 2. Smuggling a TAG_NAME_REF inside canonical_discriminator bytes with empty
    //    referenced_names refuses UndeclaredEncodedNameReference.
    let smuggled = PersistentNameMintingAuthority::mint_commit_births_for_test(
        &registry,
        &BTreeMap::new(),
        &BTreeSet::new(),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: "occ.smuggle".to_owned(),
            role_path: "cut_piece".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: Vec::new(),
            canonical_discriminator: encoded_name_ref_discriminator(&unborn),
            holder: entity_holder(2, 1),
        }],
    )
    .expect_err("smuggled TAG_NAME_REF must refuse");
    assert_eq!(
        smuggled,
        LineageNameDenial::InvalidDiscriminatorBasis {
            scheme_id: "worth.cad.face.v1".to_owned(),
            role_path: "cut_piece".to_owned(),
            reason: DiscriminatorBasisDenialReason::UndeclaredEncodedNameReference {
                raw_digest: *unborn.as_bytes(),
            },
        }
    );

    // 3. Referencing an absent/unborn parent identity refuses UnknownParentIdentity.
    let bad_parent = PersistentNameMintingAuthority::mint_commit_births_for_test(
        &registry,
        &BTreeMap::new(),
        &BTreeSet::new(),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: "occ.bad_parent".to_owned(),
            role_path: "cut_piece".to_owned(),
            parent_identities: vec![unborn],
            referenced_names: Vec::new(),
            canonical_discriminator: canonical_label_discriminator("left"),
            holder: entity_holder(3, 1),
        }],
    )
    .expect_err("unborn parent identity must refuse");
    assert_eq!(
        bad_parent,
        LineageNameDenial::UnknownParentIdentity {
            scheme_id: "worth.cad.face.v1".to_owned(),
            role_path: "cut_piece".to_owned(),
            parent: unborn,
        }
    );
}

#[test]
fn same_occurrence_basis_refuses_later_or_unborn_sibling_and_accepts_earlier_role() {
    let registry = face_registry();

    // Mint an existing live basis face first.
    let basis_birth = PersistentNameMintingAuthority::mint_commit_births_for_test(
        &registry,
        &BTreeMap::new(),
        &BTreeSet::new(),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: "occ.base".to_owned(),
            role_path: "authored".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: Vec::new(),
            canonical_discriminator: Vec::new(),
            holder: entity_holder(1, 1),
        }],
    )
    .unwrap();
    let basis_id = basis_birth[0].identity();
    let mut live_holders = BTreeMap::new();
    live_holders.insert(basis_id, entity_holder(1, 1));

    // 1. Referencing a same-or-later role in SameOccurrence refuses NonPrecedingRole.
    let later_role = PersistentNameMintingAuthority::mint_commit_births_for_test(
        &registry,
        &live_holders,
        &BTreeSet::new(),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: "occ.cut".to_owned(),
            role_path: "cut_piece".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: vec![PersistentNameDiscriminatorReference {
                identity: basis_id,
                basis: PersistentNameReferenceBasis::SameOccurrence {
                    role: "seam_patch".to_owned(),
                },
            }],
            canonical_discriminator: encoded_name_ref_discriminator(&basis_id),
            holder: entity_holder(2, 1),
        }],
    )
    .expect_err("later-role same-occurrence reference must refuse");
    assert_eq!(
        later_role,
        LineageNameDenial::InvalidDiscriminatorBasis {
            scheme_id: "worth.cad.face.v1".to_owned(),
            role_path: "cut_piece".to_owned(),
            reason: DiscriminatorBasisDenialReason::NonPrecedingRole {
                referenced_role: "seam_patch".to_owned(),
            },
        }
    );

    // 2. Referencing an earlier role whose identity was NOT minted in this occurrence refuses.
    let unborn_sibling = PersistentNameMintingAuthority::mint_commit_births_for_test(
        &registry,
        &live_holders,
        &BTreeSet::new(),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: "occ.cut".to_owned(),
            role_path: "cut_piece".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: vec![PersistentNameDiscriminatorReference {
                identity: basis_id,
                basis: PersistentNameReferenceBasis::SameOccurrence {
                    role: "authored".to_owned(),
                },
            }],
            canonical_discriminator: encoded_name_ref_discriminator(&basis_id),
            holder: entity_holder(2, 1),
        }],
    )
    .expect_err("unborn earlier-role sibling must refuse");
    assert_eq!(
        unborn_sibling,
        LineageNameDenial::InvalidDiscriminatorBasis {
            scheme_id: "worth.cad.face.v1".to_owned(),
            role_path: "cut_piece".to_owned(),
            reason: DiscriminatorBasisDenialReason::UnbornSameOccurrenceName {
                referenced_role: "authored".to_owned(),
                identity: basis_id,
            },
        }
    );

    // 3. Minting an earlier-role birth AND referencing it in a later role of the
    //    same occurrence + referencing a live Basis face succeeds!
    let authored_req = PersistentNameBirthRequest {
        scheme_id: "worth.cad.face.v1".to_owned(),
        occurrence_key: "occ.chain".to_owned(),
        role_path: "authored".to_owned(),
        parent_identities: Vec::new(),
        referenced_names: vec![PersistentNameDiscriminatorReference {
            identity: basis_id,
            basis: PersistentNameReferenceBasis::Basis,
        }],
        canonical_discriminator: encoded_name_ref_discriminator(&basis_id),
        holder: entity_holder(10, 1),
    };
    let preview = PersistentNameMintingAuthority::mint_commit_births_for_test(
        &registry,
        &live_holders,
        &BTreeSet::new(),
        std::slice::from_ref(&authored_req),
    )
    .expect("authored role referencing live basis succeeds");
    let authored_id = preview[0].identity();

    let seam_req = PersistentNameBirthRequest {
        scheme_id: "worth.cad.face.v1".to_owned(),
        occurrence_key: "occ.chain".to_owned(),
        role_path: "seam_patch".to_owned(),
        parent_identities: vec![authored_id],
        referenced_names: vec![PersistentNameDiscriminatorReference {
            identity: authored_id,
            basis: PersistentNameReferenceBasis::SameOccurrence {
                role: "authored".to_owned(),
            },
        }],
        canonical_discriminator: encoded_name_ref_discriminator(&authored_id),
        holder: entity_holder(11, 1),
    };
    let combined = PersistentNameMintingAuthority::mint_commit_births_for_test(
        &registry,
        &live_holders,
        &BTreeSet::new(),
        &[authored_req, seam_req],
    )
    .expect("earlier-role same-occurrence reference must succeed");
    assert_eq!(combined.len(), 2);
    assert_eq!(combined[0].identity(), authored_id);
    assert_eq!(combined[1].parts().parent_identities(), &[authored_id]);
}
