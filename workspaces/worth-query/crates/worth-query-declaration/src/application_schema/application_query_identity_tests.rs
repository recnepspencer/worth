use crate::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDefinitionDenial,
    ApplicationQueryDependencyCeiling, ApplicationQueryDisclosureContract,
    ApplicationQueryLaneEligibility, ApplicationQueryOrderingDirection, ApplicationQueryReference,
    ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef,
    ApplicationQueryResultShapeBuilder, ApplicationQueryResultTraversalEndpoints,
    ErasedApplicationQueryDefinition, ForwardResultTraversal, ManyResults, ReverseResultTraversal,
};

use super::canonical_identity::{canonical_identity, ApplicationSchemaCanonicalHeader};
use super::{
    ApplicationAbilityRef, ApplicationEntityRef, ApplicationFieldRef, ApplicationRelationRef,
    ApplicationSchemaIdentity, ApplicationSchemaMember, EqualityPredicate, NoApplicationUnit,
    ReadOnly,
};

struct Schema;
struct Entity;
struct Aspect;
struct Field;
struct Parameters;
struct QueryResult;
struct FirstSlot;
struct SecondSlot;
struct Relation;
struct RelationSlot;
struct ViewEntity;

crate::worth_query_structured_value_binding!(QueryParametersBinding for Parameters { identity: "Parameters" });
crate::worth_query_structured_value_binding!(QueryResultBinding for QueryResult { identity: "worth.query.test.identity-query-result.v1" });
crate::worth_query_structured_value_binding!(
    NestedResultBinding for () {
        identity: "worth.rust.unit"
    }
);
crate::worth_query_application_query!(
    Query for Schema,
    identity "Query",
    parameters QueryParametersBinding,
    result QueryResultBinding,
    scope Entity => "Entity",
    name "query"
);
worth_query_portable_type!(QueryResult => "worth.query.test.identity-query-result.v1");
worth_query_portable_type!(FirstSlot => "worth.query.test.identity-first-slot.v1");
worth_query_portable_type!(SecondSlot => "worth.query.test.identity-second-slot.v1");
worth_query_portable_type!(RelationSlot => "worth.query.test.identity-relation-slot.v1");

impl crate::application_schema::DeclaredApplicationFieldValue for Field {
    type Value = u64;
    type Binding = crate::application_schema::U64ApplicationValueBinding;
    const PRESENCE: crate::application_schema::ApplicationFieldPresence =
        crate::application_schema::ApplicationFieldPresence::Required;
}

impl crate::application_schema::RequiredApplicationFieldValue for Field {}

#[test]
fn result_slot_type_changes_definition_and_schema_identity() {
    let first = definition::<FirstSlot>().unwrap().into_erased();
    let second = definition::<SecondSlot>().unwrap().into_erased();

    assert_ne!(first.canonical_basis(), second.canonical_basis());
    assert_ne!(schema_identity(first), schema_identity(second));
}

#[test]
fn duplicate_result_slot_denies_definition_authority() {
    let entity = ApplicationEntityRef::<Schema, Entity>::from_schema_identifier("Entity");
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Entity,
        QueryResult,
        QueryResultBinding,
    >::new(entity)
    .field(selector::<FirstSlot>("first"))
    .field(selector::<FirstSlot>("second"))
    .build();
    let denial = ApplicationQueryDefinitionBuilder::declare(query_reference())
        .root(entity)
        .scope(entity)
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 2))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .err()
        .expect("one slot cannot identify two result positions");

    assert_eq!(
        denial,
        ApplicationQueryDefinitionDenial::DuplicateResultSlot
    );
}

#[test]
fn scoped_authorization_requirement_changes_definition_and_schema_identity() {
    let public = definition::<FirstSlot>().unwrap().into_erased();
    let entity = ApplicationEntityRef::<Schema, Entity>::from_schema_identifier("Entity");
    let shape = ApplicationQueryResultShapeBuilder::new(entity)
        .field(selector::<FirstSlot>("value"))
        .build();
    let governed = ApplicationQueryDefinitionBuilder::declare(query_reference())
        .root(entity)
        .scope(entity)
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 1))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(
            ApplicationAbilityRef::<Schema, ViewEntity, Entity>::from_schema_identifiers(
                "ViewEntity",
                "Entity",
            ),
        )
        .build()
        .unwrap()
        .into_erased();

    assert_ne!(public.canonical_basis(), governed.canonical_basis());
    assert_ne!(schema_identity(public), schema_identity(governed));
}

#[test]
fn result_traversal_direction_changes_definition_identity() {
    let forward = relation_definition(ApplicationQueryResultRelationRef::<
        Query,
        RelationSlot,
        Schema,
        Relation,
        Entity,
        Entity,
        ForwardResultTraversal,
        ManyResults,
    >::forward_many("related", relation_reference()));
    let reverse = relation_definition(ApplicationQueryResultRelationRef::<
        Query,
        RelationSlot,
        Schema,
        Relation,
        Entity,
        Entity,
        ReverseResultTraversal,
        ManyResults,
    >::reverse_many("related", relation_reference()));

    assert_ne!(forward.canonical_basis(), reverse.canonical_basis());
}

#[test]
fn ordering_selector_must_name_a_projected_result_slot() {
    let denial = ordering_definition(selector::<SecondSlot>("value"))
        .err()
        .expect("an absent result slot cannot become ordering authority");

    assert_eq!(
        denial,
        ApplicationQueryDefinitionDenial::UnknownOrderingResultSlot
    );
}

#[test]
fn ordering_selector_contract_must_match_its_projected_slot() {
    let denial = ordering_definition(selector::<FirstSlot>("invented"))
        .err()
        .expect("a forged selector contract cannot become ordering authority");

    assert_eq!(
        denial,
        ApplicationQueryDefinitionDenial::OrderingResultFieldMismatch
    );
}

fn definition<Slot: crate::portable_identity::WorthQueryPortableType>() -> Result<
    ApplicationQueryDefinition<Schema, Query, Parameters, QueryResult, Entity>,
    ApplicationQueryDefinitionDenial,
> {
    let entity = ApplicationEntityRef::<Schema, Entity>::from_schema_identifier("Entity");
    let shape = ApplicationQueryResultShapeBuilder::new(entity)
        .field(selector::<Slot>("value"))
        .build();
    ApplicationQueryDefinitionBuilder::declare(query_reference())
        .root(entity)
        .scope(entity)
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 1))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
}

fn query_reference() -> ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Entity> {
    Query::reference()
}

fn selector<Slot: crate::portable_identity::WorthQueryPortableType>(
    output: &'static str,
) -> ApplicationQueryResultFieldRef<
    Query,
    Slot,
    Schema,
    Entity,
    Aspect,
    Field,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new(
        output,
        ApplicationFieldRef::from_schema_identifiers("Entity", "Aspect", "Field"),
    )
}

fn relation_definition<Direction>(
    relation: ApplicationQueryResultRelationRef<
        Query,
        RelationSlot,
        Schema,
        Relation,
        Entity,
        Entity,
        Direction,
        ManyResults,
    >,
) -> ErasedApplicationQueryDefinition
where
    Direction: ApplicationQueryResultTraversalEndpoints<Entity, Entity, Entity, Entity>,
{
    let entity = ApplicationEntityRef::<Schema, Entity>::from_schema_identifier("Entity");
    let nested =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Entity, (), NestedResultBinding>::new(
            entity,
        );
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Entity,
        QueryResult,
        QueryResultBinding,
    >::new(entity)
    .relation(relation, nested)
    .build();
    ApplicationQueryDefinitionBuilder::declare(query_reference())
        .root(entity)
        .scope(entity)
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 0))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .expect("direction-only identity fixture should be valid")
        .into_erased()
}

fn ordering_definition<Slot: crate::portable_identity::WorthQueryPortableType>(
    ordering: ApplicationQueryResultFieldRef<
        Query,
        Slot,
        Schema,
        Entity,
        Aspect,
        Field,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    >,
) -> Result<
    ApplicationQueryDefinition<Schema, Query, Parameters, QueryResult, Entity>,
    ApplicationQueryDefinitionDenial,
> {
    let entity = ApplicationEntityRef::<Schema, Entity>::from_schema_identifier("Entity");
    let shape = ApplicationQueryResultShapeBuilder::new(entity)
        .field(selector::<FirstSlot>("value"))
        .build();
    ApplicationQueryDefinitionBuilder::declare(query_reference())
        .root(entity)
        .scope(entity)
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 1))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .order_by(ordering, ApplicationQueryOrderingDirection::Ascending)
        .build()
}

fn relation_reference() -> ApplicationRelationRef<Schema, Relation, Entity, Entity> {
    ApplicationRelationRef::from_schema_identifiers(
        "Relation", "Entity", "Entity",
        crate::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    )
}

fn schema_identity(definition: ErasedApplicationQueryDefinition) -> ApplicationSchemaIdentity {
    canonical_identity(
        ApplicationSchemaCanonicalHeader {
            owner: "owner",
            name: "Schema",
            major: 1,
            minor: 0,
        },
        &[ApplicationSchemaMember::ApplicationQuery { definition }],
        &[],
    )
}
