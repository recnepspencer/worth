use std::mem::size_of;

use crate::application_naming::{
    ApplicationNameDiscriminator, ApplicationNameDiscriminatorEncoder,
    ApplicationNameSchemeBuilder, ApplicationNameSchemeDenial, WorthQueryName,
    NAME_DISCRIMINATOR_DOMAIN_TAG,
};
use crate::application_schema::fragment::{Entity, Fragment, NoBindings};
use crate::application_schema::{
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaDeclarationDenial, ApplicationSchemaMember,
};

struct Face;
struct Edge;

impl Entity for Face {
    const NAME: &'static str = "Face";
    type Aspects = ();
    type Relations = ();
}

impl Entity for Edge {
    const NAME: &'static str = "Edge";
    type Aspects = ();
    type Relations = ();
}

struct CadFragment;
impl Fragment for CadFragment {
    const NAME: &'static str = "Cad";
    type Bindings = NoBindings;
    type Members = (Face, Edge);
}

struct CadSchema;
impl ApplicationSchema for CadSchema {
    const OWNER: &'static str = "worth.tests.naming";
    const NAME: &'static str = "Cad";

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let scheme = ApplicationNameSchemeBuilder::<Self, Face>::new("worth.cad.face.v1")
            .role_order(&["authored", "cut_piece", "seam_patch"])
            .role_depends_on("seam_patch", "cut_piece")
            .split_discriminator::<FaceSplitDiscriminator>()
            .require_authored_occurrence()
            .build()
            .expect("valid scheme builds");
        ApplicationSchemaDeclarationBuilder::from_fragment(CadFragment)?
            .application_name_scheme::<Face>(scheme)?
            .build()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FaceSplitDiscriminator {
    SidedCut {
        cutter: WorthQueryName<Face>,
        side: u8,
        boundary_piece: Option<WorthQueryName<Edge>>,
    },
    TrimCap {
        index: u32,
    },
}

crate::worth_query_name_discriminator! {
    enum FaceSplitDiscriminator {
        identity: "worth.cad.face.split.v1",
        referenced_roles: ["cut_piece"],
        variants: {
            SidedCut { cutter, side, boundary_piece },
            TrimCap { index },
        }
    }
}

#[test]
fn worth_query_name_is_fixed_32_bytes_and_totally_ordered() {
    assert_eq!(size_of::<WorthQueryName<Face>>(), 32);
    assert_eq!(size_of::<WorthQueryName<Edge>>(), 32);

    let low = WorthQueryName::<Face>::from_admitted_digest([0x01; 32]);
    let high = WorthQueryName::<Face>::from_admitted_digest([0x02; 32]);
    assert!(low < high);
    assert_eq!(low.as_bytes(), &[0x01; 32]);
    assert_eq!(low.render_hex(), "01".repeat(32));
}

#[test]
fn discriminator_encoding_is_domain_tagged_and_disambiguates_variants_and_options() {
    let cutter = WorthQueryName::<Face>::from_admitted_digest([0xaa; 32]);
    let edge = WorthQueryName::<Edge>::from_admitted_digest([0xbb; 32]);

    let without_edge = FaceSplitDiscriminator::SidedCut {
        cutter,
        side: 1,
        boundary_piece: None,
    };
    let with_edge = FaceSplitDiscriminator::SidedCut {
        cutter,
        side: 1,
        boundary_piece: Some(edge),
    };
    let trim = FaceSplitDiscriminator::TrimCap { index: 1 };

    let bytes_without = without_edge.canonical_bytes();
    let (bytes_with, refs_with) = with_edge.canonical_bytes_with_references();
    let (bytes_trim, refs_trim) = trim.canonical_bytes_with_references();

    assert!(bytes_without.starts_with(NAME_DISCRIMINATOR_DOMAIN_TAG));
    assert_eq!(bytes_without, without_edge.canonical_bytes());
    assert_ne!(bytes_without, bytes_with);
    assert_ne!(bytes_without, bytes_trim);
    assert_eq!(refs_with, vec![[0xaa; 32], [0xbb; 32]]);
    assert!(refs_trim.is_empty());
    // Adding an Option<WorthQueryName<Edge>> grows the encoding by exactly the
    // 32-byte identity + 1-byte TAG_NAME_REF (33 bytes).
    assert_eq!(bytes_with.len() - bytes_without.len(), 33);
}

#[test]
fn discriminator_encoder_distinguishes_prefix_ambiguous_strings() {
    let mut left = ApplicationNameDiscriminatorEncoder::new("worth.test.disc");
    left.field("a", &"xy").field("b", &"z");

    let mut right = ApplicationNameDiscriminatorEncoder::new("worth.test.disc");
    right.field("a", &"x").field("b", &"yz");

    assert_ne!(left.finish(), right.finish());
}

#[test]
fn schema_registers_application_name_scheme_and_commits_canonical_identity() {
    let declaration = CadSchema::declaration().expect("valid schema with name scheme");
    let schemes: Vec<_> = declaration
        .erased()
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::ApplicationNameScheme { definition } => Some(definition),
            _ => None,
        })
        .collect();
    assert_eq!(schemes.len(), 1);
    assert_eq!(schemes[0].scheme_id(), "worth.cad.face.v1");
    assert_eq!(schemes[0].subject_name(), "Face");
    assert_eq!(
        schemes[0].role_order(),
        &["authored", "cut_piece", "seam_patch"]
    );
    assert!(schemes[0].split_capable());
    assert_eq!(
        schemes[0].split_discriminator_identity(),
        Some("worth.cad.face.split.v1")
    );
    assert!(schemes[0].require_authored_occurrence());

    let plain = ApplicationSchemaDeclarationBuilder::<CadSchema>::from_fragment(CadFragment)
        .unwrap()
        .build()
        .unwrap();
    assert_ne!(
        declaration.identity(),
        plain.identity(),
        "name scheme must enter canonical application schema identity"
    );
}

#[test]
fn scheme_builder_rejects_empty_identifier_and_empty_role_order() {
    let bad_id = ApplicationNameSchemeBuilder::<CadSchema, Face>::new("")
        .role_order(&["authored"])
        .build();
    assert_eq!(
        bad_id,
        Err(ApplicationNameSchemeDenial::InvalidSchemeIdentifier {
            scheme_id: String::new()
        })
    );

    let empty_roles =
        ApplicationNameSchemeBuilder::<CadSchema, Face>::new("worth.cad.face.v1").build();
    assert_eq!(
        empty_roles,
        Err(ApplicationNameSchemeDenial::EmptyRoleOrder {
            scheme_id: "worth.cad.face.v1".to_owned()
        })
    );
}
