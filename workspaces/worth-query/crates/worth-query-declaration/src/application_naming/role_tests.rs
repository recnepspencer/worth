use std::cmp::Ordering;

use super::{
    ApplicationNameDiscriminator, ApplicationNameDiscriminatorEncoder,
    ApplicationNameRoleDependency, ApplicationNameSchemeBuilder, IndexedRoleDiscriminator,
    NameSelection, WorthQueryNameRole, NAME_ROLE_INDEX_IDENTITY,
};
use crate::application_schema::fragment::Entity;
use crate::application_schema::{ApplicationRelationIntegrity, ApplicationRelationRef};

struct Cad;
struct Face;
impl Entity for Face {
    const NAME: &'static str = "Face";
    type Aspects = ();
    type Relations = ();
}
struct Seam;

/// The impl `#[derive(WorthQueryNameRole)]` writes, by hand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FaceRole {
    Authored,
    CutPiece,
    Instance(u32),
}

impl FaceRole {
    const fn canonical(self) -> (u32, u32) {
        match self {
            Self::Authored => (0, 0),
            Self::CutPiece => (1, 0),
            Self::Instance(index) => (2, index),
        }
    }
}

impl Ord for FaceRole {
    fn cmp(&self, other: &Self) -> Ordering {
        self.canonical().cmp(&other.canonical())
    }
}

impl PartialOrd for FaceRole {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl WorthQueryNameRole for FaceRole {
    const ROLE_ORDER: &'static [&'static str] = &["Authored", "CutPiece", "Instance"];
    const ROLE_DEPENDENCIES: &'static [(&'static str, &'static str)] = &[
        ("CutPiece", "Authored"),
        ("Instance", "Authored"),
        ("Instance", "CutPiece"),
    ];

    fn role_name(&self) -> &'static str {
        match self {
            Self::Authored => "Authored",
            Self::CutPiece => "CutPiece",
            Self::Instance(_) => "Instance",
        }
    }

    fn role_index(&self) -> Option<u32> {
        match self {
            Self::Instance(index) => Some(*index),
            _ => None,
        }
    }
}

#[test]
fn a_scheme_from_roles_is_the_scheme_its_role_order_and_dependencies_declare() {
    let from_roles =
        ApplicationNameSchemeBuilder::<Cad, Face>::from_roles::<FaceRole>("worth.cad.face.v1")
            .require_authored_occurrence()
            .build()
            .expect("the typed scheme builds");
    let by_hand = ApplicationNameSchemeBuilder::<Cad, Face>::new("worth.cad.face.v1")
        .role_order(&["Authored", "CutPiece", "Instance"])
        .role_depends_on("CutPiece", "Authored")
        .role_depends_on("Instance", "Authored")
        .role_depends_on("Instance", "CutPiece")
        .require_authored_occurrence()
        .build()
        .expect("the hand-written scheme builds");
    assert_eq!(from_roles, by_hand);
    assert_eq!(
        from_roles.erased().role_dependencies()[2],
        ApplicationNameRoleDependency::new("Instance", "CutPiece")
    );
    // Every role a value can be born under is a declared role, whatever its
    // index: the name is never an undeclared `Instance[7]`.
    for role in [
        FaceRole::Authored,
        FaceRole::CutPiece,
        FaceRole::Instance(7),
    ] {
        assert!(from_roles
            .erased()
            .role_order()
            .iter()
            .any(|declared| declared == role.role_name()));
    }
}

#[test]
fn a_relation_scheme_takes_its_roles_from_the_same_type() {
    let relation = || {
        ApplicationRelationRef::<Cad, Seam, Face, Face>::from_schema_identifiers(
            "Seam",
            "Face",
            "Face",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    };
    let from_roles = ApplicationNameSchemeBuilder::<Cad, Seam>::for_relation_from_roles::<
        FaceRole,
        _,
        _,
    >("worth.cad.seam.v1", relation())
    .build()
    .expect("the typed relation scheme builds");
    let by_hand =
        ApplicationNameSchemeBuilder::<Cad, Seam>::for_relation("worth.cad.seam.v1", relation())
            .role_order(FaceRole::ROLE_ORDER)
            .role_depends_on("CutPiece", "Authored")
            .role_depends_on("Instance", "Authored")
            .role_depends_on("Instance", "CutPiece")
            .build()
            .expect("the hand-written relation scheme builds");
    assert_eq!(from_roles, by_hand);
    assert_eq!(from_roles.erased().subject_name(), "Seam");
}

struct Side(u8);
impl ApplicationNameDiscriminator for Side {
    const DISCRIMINATOR_IDENTITY: &'static str = "worth.tests.side.v1";
    fn encode_canonical(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        encoder.field("side", &self.0);
    }
}

fn indexed(index: u32) -> Vec<u8> {
    IndexedRoleDiscriminator::index_alone(index).canonical_bytes()
}

#[test]
fn a_role_index_leads_the_canonical_discriminator_and_compares_as_a_number() {
    assert!(FaceRole::Instance(10) > FaceRole::Instance(9));
    assert!(FaceRole::Instance(9) > FaceRole::Instance(2));
    assert!(FaceRole::Instance(0) > FaceRole::CutPiece);
    // The canonical bytes Relational orders sibling births by agree: the
    // index is a big-endian `u32`, never decimal text.
    assert!(indexed(10) > indexed(9) && indexed(9) > indexed(2));
    assert!(indexed(256) > indexed(255) && indexed(u32::MAX) > indexed(1 << 24));
    assert_eq!(indexed(2).len(), indexed(u32::MAX).len());
    assert!(indexed(7).ends_with(&7_u32.to_be_bytes()));

    let mut by_hand = ApplicationNameDiscriminatorEncoder::new(NAME_ROLE_INDEX_IDENTITY);
    by_hand.field("worth.role.index", &7_u32);
    assert_eq!(indexed(7), by_hand.finish());
}

#[test]
fn an_indexed_birth_keeps_its_typed_discriminator_after_the_index() {
    let composed = IndexedRoleDiscriminator::new(3, Side(1)).canonical_bytes();
    let mut by_hand = ApplicationNameDiscriminatorEncoder::new(Side::DISCRIMINATOR_IDENTITY);
    by_hand
        .field("worth.role.index", &3_u32)
        .field("side", &1_u8);
    assert_eq!(composed, by_hand.finish());
    // What a birth writes through the encoder is the same discriminator.
    let mut written = ApplicationNameDiscriminatorEncoder::new(Side::DISCRIMINATOR_IDENTITY);
    Side(1).encode_canonical(written.role_index(3));
    assert_eq!(composed, written.finish());

    // The index tells siblings of one side apart; the side tells siblings of
    // one index apart; and neither is the unindexed discriminator.
    assert_ne!(
        composed,
        IndexedRoleDiscriminator::new(4, Side(1)).canonical_bytes()
    );
    assert_ne!(
        composed,
        IndexedRoleDiscriminator::new(3, Side(2)).canonical_bytes()
    );
    assert_ne!(composed, Side(1).canonical_bytes());
    assert_ne!(composed, indexed(3));
}

#[test]
fn a_side_selection_of_an_indexed_piece_carries_its_role_index() {
    let indexed_side = NameSelection::keep_side(&IndexedRoleDiscriminator::new(3, Side(1)));
    let NameSelection::KeepSide { discriminator } = indexed_side.expect("a well-formed side")
    else {
        unreachable!("keep_side builds a side selection")
    };
    assert_eq!(
        discriminator.as_bytes(),
        IndexedRoleDiscriminator::new(3, Side(1)).canonical_bytes()
    );
    assert_ne!(
        NameSelection::keep_side(&IndexedRoleDiscriminator::new(3, Side(1))),
        NameSelection::keep_side(&Side(1))
    );
    assert!(NameSelection::keep_side(&IndexedRoleDiscriminator::index_alone(3)).is_ok());
}
