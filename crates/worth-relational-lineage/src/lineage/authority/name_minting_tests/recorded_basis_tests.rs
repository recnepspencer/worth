//! A discriminator may reference any name recorded on the branch at the
//! basis: live, retired by an earlier commit (a retired ancestor) or retired
//! by this commit (a consumed cutter). A name the branch never recorded still
//! refuses, and a structural-identity birth's discriminator stays bounded.
use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::events::LineageSubject;
use crate::contracts::lineage::data::naming::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry,
    PersistentNameBirthRecord, PersistentNameBirthRequest, PersistentNameDiscriminatorReference,
    PersistentNameHolder, PersistentNameIdentity, PersistentNameIdentityOwnerPort as _,
    PersistentNameReferenceBasis, PersistentNameSubjectKind,
};
use crate::contracts::lineage::data::naming_denial::{
    DiscriminatorBasisDenialReason, LineageNameDenial,
};
use crate::contracts::lineage::data::postures::LineageBasisView;
use crate::lineage::authority::{
    PersistentNameMintingAuthority, MAX_STRUCTURAL_DISCRIMINATOR_BYTES,
};

use super::basis_tests::encoded_name_ref_discriminator;
use super::{entity_holder, face_registry};

/// Live holders and the names retired by earlier commits on one branch.
struct Branch {
    live: BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
    retired: BTreeSet<PersistentNameIdentity>,
}

impl LineageBasisView for Branch {
    fn live_holder(&self, id: &PersistentNameIdentity) -> Option<PersistentNameHolder> {
        self.live.get(id).cloned()
    }
    fn live_lineage(&self, _: &PersistentNameHolder) -> Option<LineageId> {
        None
    }
    fn retired_lineage(&self, id: &PersistentNameIdentity) -> Option<(LineageSubject, LineageId)> {
        self.retired
            .contains(id)
            .then_some((LineageSubject::Entity, LineageId(9)))
    }
    fn holder_name(&self, _: &PersistentNameHolder) -> Option<PersistentNameIdentity> {
        None
    }
}

fn name(byte: u8) -> PersistentNameIdentity {
    PersistentNameIdentity::from_minted_digest([byte; 32])
}

/// A cut piece told apart by the cutter it references at the basis.
fn piece_cut_by(cutter: PersistentNameIdentity, slot: u32) -> PersistentNameBirthRequest {
    PersistentNameBirthRequest {
        scheme_id: "worth.cad.face.v1".to_owned(),
        occurrence_key: "occ.cut".to_owned(),
        role_path: "cut_piece".to_owned(),
        parent_identities: Vec::new(),
        referenced_names: vec![PersistentNameDiscriminatorReference {
            identity: cutter,
            basis: PersistentNameReferenceBasis::Basis,
        }],
        canonical_discriminator: encoded_name_ref_discriminator(&cutter),
        holder: entity_holder(slot, 1),
    }
}

fn mint(
    registry: &InstalledPersistentNameSchemeRegistry,
    branch: &Branch,
    retired_in_commit: &BTreeSet<PersistentNameIdentity>,
    request: PersistentNameBirthRequest,
) -> Result<PersistentNameBirthRecord, LineageNameDenial> {
    PersistentNameMintingAuthority::mint_commit_births_with_basis(
        registry,
        branch,
        retired_in_commit,
        &[request],
    )
    .map(|mut records| records.remove(0))
}

#[test]
fn a_retired_ancestor_or_consumed_cutter_is_referenced_and_an_unrecorded_name_refuses() {
    let (live, ancestor, consumed, unrecorded) = (name(1), name(2), name(3), name(4));
    let branch = Branch {
        live: BTreeMap::from([(live, entity_holder(1, 1)), (consumed, entity_holder(3, 1))]),
        retired: BTreeSet::from([ancestor]),
    };
    let registry = face_registry();
    let none = BTreeSet::new();
    let by_live = mint(&registry, &branch, &none, piece_cut_by(live, 10)).expect("live");
    let by_ancestor = mint(&registry, &branch, &none, piece_cut_by(ancestor, 11))
        .expect("a name retired by an earlier commit is recorded at the basis");
    let consuming = BTreeSet::from([consumed]);
    let by_consumed = mint(&registry, &branch, &consuming, piece_cut_by(consumed, 12))
        .expect("a cutter this commit consumes is recorded at the basis");
    // Each reference contributes its fixed 32-byte identity to the name.
    let minted = [&by_live, &by_ancestor, &by_consumed].map(|record| record.identity());
    assert_eq!(BTreeSet::from(minted).len(), 3);
    assert_eq!(
        by_ancestor.parts().canonical_discriminator(),
        encoded_name_ref_discriminator(&ancestor)
    );
    let refused = mint(&registry, &branch, &none, piece_cut_by(unrecorded, 13));
    assert_eq!(
        refused,
        Err(LineageNameDenial::InvalidDiscriminatorBasis {
            scheme_id: "worth.cad.face.v1".to_owned(),
            role_path: "cut_piece".to_owned(),
            reason: DiscriminatorBasisDenialReason::AbsentBasisName {
                identity: unrecorded
            },
        })
    );
}

#[test]
fn a_structural_identity_discriminator_is_bounded_by_the_wire_ceiling() {
    let mut registry = InstalledPersistentNameSchemeRegistry::new();
    let scheme = InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
        "worth.cad.face.v1",
        PersistentNameSubjectKind::Entity,
        "face",
        ["authored", "cut_piece", "seam_patch"],
        true,
        true,
        true,
    );
    registry
        .register(scheme.expect("structural face scheme"))
        .expect("registered");
    let branch = Branch {
        live: BTreeMap::new(),
        retired: BTreeSet::new(),
    };
    let birth = |len: usize| {
        // The domain tag, an empty identity, then one text label padding
        // the discriminator to exactly `len` bytes.
        let mut bytes = b"worth.query.name-discriminator.v1\0".to_vec();
        let label = "x".repeat(len - bytes.len() - 8 - 1 - 8);
        bytes.extend_from_slice(&0u64.to_be_bytes());
        bytes.push(0x19); // TAG_TEXT
        bytes.extend_from_slice(&(label.len() as u64).to_be_bytes());
        bytes.extend_from_slice(label.as_bytes());
        PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: format!("occ.{len}"),
            role_path: "authored".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: Vec::new(),
            canonical_discriminator: bytes,
            holder: entity_holder(1, 1),
        }
    };
    let at_ceiling = birth(MAX_STRUCTURAL_DISCRIMINATOR_BYTES);
    assert_eq!(
        at_ceiling.canonical_discriminator.len(),
        MAX_STRUCTURAL_DISCRIMINATOR_BYTES
    );
    mint(&registry, &branch, &BTreeSet::new(), at_ceiling).expect("at the ceiling");
    let over = MAX_STRUCTURAL_DISCRIMINATOR_BYTES + 1;
    assert_eq!(
        mint(&registry, &branch, &BTreeSet::new(), birth(over)),
        Err(LineageNameDenial::InvalidDiscriminatorBasis {
            scheme_id: "worth.cad.face.v1".to_owned(),
            role_path: "authored".to_owned(),
            reason: DiscriminatorBasisDenialReason::UnboundedStructuralDiscriminator { len: over },
        })
    );
    // The default scheme keeps no ceiling of its own.
    mint(&face_registry(), &branch, &BTreeSet::new(), birth(over)).expect("default");
}
