//! Typed aggregate slots and fresh declaration/schema closure, without runtime authority.
use crate::{application_query::*, application_schema::*, *};
struct Schema;
struct Node;
struct Set;
struct Slot;
struct Row;
worth_query_portable_type!(Slot => "worth.tests.components.slot.v1");
worth_query_portable_type!(Row => "worth.tests.components.row.v1");
worth_query_structured_value_binding!(ParametersBinding for () {identity:"worth.rust.unit"});
worth_query_structured_value_binding!(RowsBinding for Row {identity:"worth.tests.components.row.v1"});
worth_query_application_query!(Query for Schema,identity "worth.tests.components.query.v1",parameters ParametersBinding,result RowsBinding,scope Node => "Node", name "components");
fn node() -> ApplicationEntityRef<Schema, Node> {
    ApplicationEntityRef::from_schema_identifier("Node")
}
fn set() -> ApplicationComponentSetRef<Schema, Set, Node> {
    ApplicationComponentSetRef::new("Shells", node())
}
fn selection(
    mode: ApplicationQueryComponentSelection,
) -> ApplicationQueryComponentRef<Schema, Query, Slot, Set, Node> {
    match mode {
        ApplicationQueryComponentSelection::OfScopeMember => {
            ApplicationQueryComponentRef::of_scope_member("shells", set())
        }
        ApplicationQueryComponentSelection::Census => {
            ApplicationQueryComponentRef::census("shells", set())
        }
    }
}
fn query(mode: ApplicationQueryComponentSelection) -> ErasedApplicationQueryDefinition {
    ApplicationQueryDefinitionBuilder::declare(Query::reference()).root(node()).scope(node())
        .result_shape(ApplicationQueryResultShapeBuilder::<Schema,Query,Node,Row,RowsBinding>::new(node()).build())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public().components_by(selection(mode), ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(0).unwrap(), ApplicationQueryInfluenceContract::permit_all()))
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned()).lanes(ApplicationQueryLaneEligibility::one_shot()).public()
        .root_path(ApplicationQueryRootPath::from(node())).component_selection(selection(mode)).build().unwrap().into_erased()
}
fn members(query: ErasedApplicationQueryDefinition) -> Vec<ApplicationSchemaMember> {
    vec![
        ApplicationSchemaMember::Entity {
            entity: "Node".into(),
        },
        ApplicationSchemaMember::Relation {
            relation: "Next".into(),
            from: "Node".into(),
            to: "Node".into(),
            integrity: ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
                .into(),
        },
        ApplicationSchemaMember::ComponentSet {
            definition: ApplicationComponentSetMeaning::from_untrusted_parts(
                WorthQueryPortableApplicationComponentSetParts {
                    name: "Shells".into(),
                    member: "Node".into(),
                    relations: vec!["Next".into()],
                },
            ),
        },
        ApplicationSchemaMember::ApplicationQuery { definition: query },
    ]
}
#[test]
fn typed_metadata_only_of_and_census_have_distinct_complete_slot_meaning() {
    let of = query(ApplicationQueryComponentSelection::OfScopeMember);
    let census = query(ApplicationQueryComponentSelection::Census);
    for query in [&of, &census] {
        assert!(query.result_shape().fields().is_empty());
        assert_eq!(
            super::super::validate_application_query_members(&members(query.clone())),
            Ok(())
        );
    }
    assert_ne!(
        of.component_selection().unwrap().slot_key(),
        census.component_selection().unwrap().slot_key()
    );
    assert_ne!(of.canonical_basis(), census.canonical_basis());
}
#[test]
fn fresh_component_shapes_refuse_missing_paths_scope_and_mismatched_disclosure() {
    let base = query(ApplicationQueryComponentSelection::OfScopeMember).into_parts();
    let mut missing = base.clone();
    missing.root_paths.clear();
    assert_eq!(
        validate_portable_application_query_freshly(&missing),
        Err(ApplicationQueryDefinitionDenial::InvalidComponentShape)
    );
    let mut scope = base.clone();
    scope.scope_entity = "Other".into();
    scope.root_paths = vec![ApplicationQueryRootPathMeaning::from_untrusted_parts(
        WorthQueryPortableApplicationQueryRootPathParts {
            start_entity: "Other".into(),
            terminal_entity: "Node".into(),
            steps: vec![ApplicationQueryRootPathStep::from_untrusted_fields(
                "Owns".into(),
                "Other".into(),
                "Node".into(),
                RelationDirection::Forward,
                ApplicationQueryRootPathTraversal::OneHop,
            )],
            guards: vec![],
        },
    )];
    assert_eq!(
        validate_portable_application_query_freshly(&scope),
        Err(ApplicationQueryDefinitionDenial::InvalidComponentShape)
    );
    let mut both = base.clone();
    both.orbit = Some(ApplicationQueryOrbitMeaning::from_untrusted_parts(
        WorthQueryPortableApplicationQueryOrbitParts {
            query: base.query_type.clone(),
            slot: base.component_selection.as_ref().unwrap().slot_identity(),
            relation: "Next".into(),
            from: "Node".into(),
            to: "Node".into(),
            output_name: "orbit".into(),
        },
    ));
    assert!(validate_portable_application_query_freshly(&both).is_err());
    let mut disclosure = base.clone();
    disclosure.disclosure = query(ApplicationQueryComponentSelection::Census)
        .into_parts()
        .disclosure;
    assert_eq!(
        validate_portable_application_query_freshly(&disclosure),
        Err(ApplicationQueryDefinitionDenial::DisclosureSelectorMismatch)
    );
    let mut ordinary = base;
    ordinary.component_selection = None;
    ordinary.disclosure = ApplicationQueryDisclosureContract::public();
    assert_eq!(
        validate_portable_application_query_freshly(&ordinary),
        Err(ApplicationQueryDefinitionDenial::InvalidRootPath)
    );
}
#[test]
fn actual_component_set_and_path_dependencies_must_close() {
    let base = query(ApplicationQueryComponentSelection::Census);
    let mut missing = members(base.clone());
    missing.retain(|m| !matches!(m, ApplicationSchemaMember::ComponentSet { .. }));
    assert_eq!(
        super::super::validate_application_query_members(&missing),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery)
    );
    let mut wrong = members(base.clone());
    let ApplicationSchemaMember::ComponentSet { definition } = &mut wrong[2] else {
        unreachable!()
    };
    *definition = ApplicationComponentSetMeaning::from_untrusted_parts(
        WorthQueryPortableApplicationComponentSetParts {
            name: "Shells".into(),
            member: "Other".into(),
            relations: vec!["Next".into()],
        },
    );
    assert_eq!(
        super::super::validate_application_query_members(&wrong),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery)
    );
    let mut parts = base.into_parts();
    parts.root_paths = vec![ApplicationQueryRootPathMeaning::from_untrusted_parts(
        WorthQueryPortableApplicationQueryRootPathParts {
            start_entity: "Node".into(),
            terminal_entity: "Node".into(),
            steps: vec![ApplicationQueryRootPathStep::from_untrusted_fields(
                "Missing".into(),
                "Node".into(),
                "Node".into(),
                RelationDirection::Forward,
                ApplicationQueryRootPathTraversal::OneHop,
            )],
            guards: vec![],
        },
    )];
    let bad = ErasedApplicationQueryDefinition::from_untrusted_parts(parts);
    assert_eq!(
        super::super::validate_application_query_members(&members(bad)),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery)
    );
}
