//! Value courts for lowering declared persistent-name schemes onto the exact
//! kinds an installed layout lowered.
use std::collections::BTreeMap;

use worth_query_declaration::facade::application_naming::ApplicationNameSchemeBuilder;
use worth_query_declaration::facade::application_schema::fragment::{
    self, Entity, Fragment, NoBindings, Relation,
};
use worth_query_declaration::facade::application_schema::*;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::lineage::PersistentNameSubjectKind;
use worth_relational::facade::schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId};

use super::super::{register_entity, register_relation};

const FACE_SCHEME: &str = "worth.tests.layout.face.v1";
const EDGE_SCHEME: &str = "worth.tests.layout.edge.v1";

struct Face;
impl Entity for Face {
    const NAME: &'static str = "Face";
    type Aspects = ();
    type Relations = (Edge,);
}
struct Edge;
impl Relation for Edge {
    const NAME: &'static str = "Edge";
    type To = Face;
}
struct Faces;
impl Fragment for Faces {
    const NAME: &'static str = "LayoutNaming";
    type Bindings = NoBindings;
    type Members = (Face,);
}
struct NamingSchema;
impl ApplicationSchema for NamingSchema {
    const OWNER: &'static str = "worth.tests.layout.naming";
    const NAME: &'static str = "LayoutNaming";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let scheme = ApplicationNameSchemeBuilder::<Self, Face>::new(FACE_SCHEME)
            .role_order(&["authored"])
            .require_authored_occurrence()
            .build()
            .expect("valid face scheme");
        let edge = ApplicationNameSchemeBuilder::<Self, Edge>::for_relation(
            EDGE_SCHEME,
            fragment::relation(Faces, Face, Edge),
        )
        .role_order(&["seam"])
        .build()
        .expect("valid edge scheme");
        ApplicationSchemaDeclarationBuilder::from_fragment(Faces)?
            .application_name_scheme::<Face>(scheme)?
            .application_name_scheme::<Edge>(edge)?
            .build()
    }
}
struct PlainSchema;
impl ApplicationSchema for PlainSchema {
    const OWNER: &'static str = "worth.tests.layout.naming";
    const NAME: &'static str = "LayoutNaming";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::from_fragment(Faces)?.build()
    }
}

fn face_registry(kind: KindId) -> RelationalSchemaRegistry {
    register_entity(
        RelationalSchemaRegistry::new(),
        &SchemaId("worth.tests.layout.naming".to_owned()),
        SchemaVersionId(1),
        "Face",
        kind,
        Vec::new(),
    )
    .unwrap()
}

fn kinds(kind: KindId) -> BTreeMap<String, KindId> {
    BTreeMap::from([("Face".to_owned(), kind)])
}

/// `face_registry` plus the declared `Edge` relation at `edge`.
fn edge_registry(
    declaration: &ErasedApplicationSchemaDeclaration,
    face: KindId,
    edge: KindId,
) -> RelationalSchemaRegistry {
    let integrity = declaration
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::Relation { integrity, .. } => Some(integrity),
            _ => None,
        })
        .expect("the fragment declares Edge");
    register_relation(
        face_registry(face),
        &SchemaId("worth.tests.layout.naming".to_owned()),
        SchemaVersionId(1),
        "Edge",
        edge,
        face,
        face,
        integrity,
        None,
    )
    .unwrap()
}

#[test]
fn declared_schemes_bind_to_the_lowered_kind_identity() {
    let declaration = NamingSchema::declaration().unwrap().into_erased();
    for (kind, edge) in [(KindId(7), KindId(8)), (KindId(41), KindId(42))] {
        let edges = BTreeMap::from([("Edge".to_owned(), edge)]);
        let (registry, names) = super::lower(
            edge_registry(&declaration, kind, edge),
            &declaration,
            &kinds(kind),
            &edges,
        )
        .unwrap();
        assert_eq!(
            registry.scheme_id_for_kind(PersistentNameSubjectKind::Entity, kind),
            Some(FACE_SCHEME)
        );
        assert_eq!(
            registry.scheme_id_for_kind(PersistentNameSubjectKind::Relation, edge),
            Some(EDGE_SCHEME)
        );
        let names = names.expect("a schemed declaration keeps its naming layout");
        assert_eq!(
            names.scheme_for_kind(PersistentNameSubjectKind::Entity, kind),
            Some(FACE_SCHEME)
        );
        assert_eq!(
            names.scheme_for_kind(PersistentNameSubjectKind::Relation, edge),
            Some(EDGE_SCHEME)
        );
        assert!(names.catalog().scheme_by_id(FACE_SCHEME).is_some());
    }
}

#[test]
fn a_schemaless_declaration_leaves_the_registry_bit_identical() {
    let declaration = PlainSchema::declaration().unwrap().into_erased();
    let before = face_registry(KindId(7));
    let (after, names) = super::lower(
        before.clone(),
        &declaration,
        &kinds(KindId(7)),
        &BTreeMap::new(),
    )
    .unwrap();
    assert!(names.is_none());
    assert_eq!(after, before);
    assert_eq!(after.authority_snapshot(), before.authority_snapshot());
}

#[test]
fn a_scheme_whose_subject_was_not_lowered_refuses() {
    let declaration = NamingSchema::declaration().unwrap().into_erased();
    assert!(super::lower(
        face_registry(KindId(7)),
        &declaration,
        &BTreeMap::new(),
        &BTreeMap::new()
    )
    .is_err());
    assert!(
        super::lower(
            face_registry(KindId(7)),
            &declaration,
            &kinds(KindId(7)),
            &BTreeMap::new()
        )
        .is_err(),
        "an unlowered relation subject refuses too"
    );
}
