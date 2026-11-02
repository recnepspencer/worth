//! Actual typed paths preserve namespace until the complete physical entrance.
use super::*;
use crate::application_query::*;
use crate::application_schema::*;
struct Schema;
struct Node;
struct Edge;
struct Live;
impl ApplicationEntityMarkerIdentity<Schema> for Node {
    const IDENTIFIER: &'static str = "Node";
}
fn node() -> ApplicationEntityRef<Schema, Node> {
    ApplicationEntityRef::from_schema_identifier("Node")
}
fn edge() -> ApplicationRelationRef<Schema, Edge, Node, Node> {
    ApplicationRelationRef::from_schema_identifiers(
        "Edges",
        "Node",
        "Node",
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    )
}
fn live() -> ApplicationDerivedRelationRef<Schema, Live, Node, Node> {
    ApplicationDerivedRelationRef::from_schema_identifiers("Live", "Node", "Node")
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "root-path.namespace";
    const NAME: &'static str = "Nodes";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::for_schema()
            .entity(node())
            .relation(edge(), node(), node())
            .derived_relation(
                live(),
                ApplicationDerivedRelation::compose(edge(), edge()),
                ApplicationRelationGraphLaws::unbounded(),
            )
            .build()
    }
}
fn members() -> Vec<ApplicationSchemaMember> {
    Schema::declaration().unwrap().erased().members().to_vec()
}
#[test]
fn typed_derived_paths_are_closed_only_in_their_exact_entity_namespace() {
    let members = members();
    for path in [
        ApplicationQueryRootPath::from(node())
            .forward_derived(live())
            .into_meaning(),
        ApplicationQueryRootPath::from(node())
            .reverse_derived(live())
            .into_meaning(),
    ] {
        assert!(is_closed(&path, &members));
        assert_eq!(
            path.steps()[0].source_kind(),
            ApplicationQueryRootPathSourceKind::Derived
        );
        let mut tokens = Vec::new();
        path.visit_canonical_parts(|part| tokens.push(part.to_owned()));
        assert!(tokens
            .iter()
            .any(|t| t == "forward-derived" || t == "reverse-derived"));
        let mut missing = members.clone();
        missing.retain(|m| !matches!(m, ApplicationSchemaMember::DerivedRelation { .. }));
        // A physical relation with exactly the same name still is not this source.
        let physical = missing
            .iter_mut()
            .find_map(|m| match m {
                ApplicationSchemaMember::Relation { relation, .. } => Some(relation),
                _ => None,
            })
            .unwrap();
        *physical = "Live".into();
        assert!(!is_closed(&path, &missing));
    }
    for (from, to) in [
        (
            ApplicationGraphMemberSelector::Projected("Node"),
            ApplicationGraphMemberSelector::Entity("Node"),
        ),
        (
            ApplicationGraphMemberSelector::Entity("Node"),
            ApplicationGraphMemberSelector::Projected("Node"),
        ),
    ] {
        let alias =
            ApplicationDerivedRelationRef::<Schema, Live, Node, Node>::from_member_identifiers(
                "Live", from, to,
            );
        let path = ApplicationQueryRootPath::from(node())
            .forward_derived(alias)
            .into_meaning();
        assert!(!path.is_well_formed());
        assert!(!is_closed(&path, &members));
        assert_eq!(path.steps()[0].from_member(), &from.into_meaning());
        assert_eq!(path.steps()[0].to_member(), &to.into_meaning());
    }
}
#[test]
fn entity_final_recipe_with_projected_intermediate_is_explicitly_refused() {
    let mut members = members();
    members.push(ApplicationSchemaMember::ProjectedMemberDomain {
        definition: ErasedApplicationProjectedMemberDomainDefinition::from_untrusted_parts(
            WorthQueryPortableProjectedMemberDomainParts {
                semantic_version: 1,
                domain: "Ports".into(),
                carrier: "Node".into(),
                cases: vec!["port".into()],
            },
        ),
    });
    let parts = members
        .iter_mut()
        .find_map(|m| match m {
            ApplicationSchemaMember::DerivedRelation { definition } => Some(definition),
            _ => None,
        })
        .unwrap();
    *parts = ErasedApplicationDerivedRelationDefinition::from_untrusted_parts(
        WorthQueryPortableDerivedRelationParts {
            relation: "Live".into(),
            from: "Node".into(),
            to: "Node".into(),
            laws: ApplicationRelationGraphLaws::unbounded(),
            expression: ApplicationDerivedRelationExpression {
                nodes: vec![
                    ApplicationDerivedRelationNode::CarrierLift {
                        domain: "Ports".into(),
                        carrier: "Node".into(),
                        case: "port".into(),
                    },
                    ApplicationDerivedRelationNode::Reverse { input: 0 },
                    ApplicationDerivedRelationNode::Compose { left: 0, right: 1 },
                ],
            },
        },
    );
    parts.validate().unwrap();
    assert_eq!(
        application_derived_relation_source_closure(&members, "Live")
            .unwrap()
            .projected_domains
            .len(),
        1
    );
    let path = ApplicationQueryRootPath::from(node())
        .forward_derived(live())
        .into_meaning();
    assert!(path.is_well_formed());
    assert!(!is_closed(&path, &members));
}
