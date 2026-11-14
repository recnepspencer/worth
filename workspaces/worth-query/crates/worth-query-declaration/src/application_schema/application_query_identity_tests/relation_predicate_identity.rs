use super::*;

#[test]
fn relation_predicate_changes_definition_identity() {
    let relation = ApplicationQueryResultRelationRef::<
        Query,
        RelationSlot,
        Schema,
        Relation,
        Entity,
        Entity,
        ForwardResultTraversal,
        ManyResults,
    >::forward_many("related", relation_reference());
    let unfiltered = relation_definition(relation);
    let filtered = filtered_relation_definition(relation, false);
    let absent = filtered_relation_definition(relation, true);

    assert_ne!(unfiltered.canonical_basis(), filtered.canonical_basis());
    assert_ne!(unfiltered.canonical_basis(), absent.canonical_basis());
    assert_ne!(filtered.canonical_basis(), absent.canonical_basis());
    assert!(
        absent.parameters().is_empty(),
        "absence has no artificial parameter"
    );
}

fn filtered_relation_definition(
    relation: ApplicationQueryResultRelationRef<
        Query,
        RelationSlot,
        Schema,
        Relation,
        Entity,
        Entity,
        ForwardResultTraversal,
        ManyResults,
    >,
    absent: bool,
) -> ErasedApplicationQueryDefinition {
    let entity = ApplicationEntityRef::<Schema, Entity>::from_schema_identifier("Entity");
    let nested =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Entity, (), NestedResultBinding>::new(
            entity,
        );
    let parameter = ApplicationQueryParameterRef::<
        Query,
        PredicateParameter,
        crate::application_schema::U64ApplicationValueBinding,
    >::from_query_identifier("selected");
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Entity,
        QueryResult,
        QueryResultBinding,
    >::new(entity);
    let shape = if absent {
        shape.relation_where_absent(relation, nested, Field::at("Field"))
    } else {
        shape.relation_where_equal(relation, nested, Field::at("Field"), parameter)
    }
    .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(query_reference())
        .root(entity)
        .scope(entity)
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public();
    let definition = if absent {
        definition
    } else {
        definition.parameter(parameter)
    };
    definition.build().unwrap().into_erased()
}

#[test]
fn intrinsic_relation_identity_requires_its_parameter_and_changes_canonical_meaning() {
    let exact = identity_relation_definition(IdentityParameter::Exact).unwrap();
    let relation = ApplicationQueryResultRelationRef::<
        Query,
        RelationSlot,
        Schema,
        Relation,
        Entity,
        Entity,
        ForwardResultTraversal,
        ManyResults,
    >::forward_many("related", relation_reference());
    assert_ne!(
        exact.canonical_basis(),
        filtered_relation_definition(relation, false).canonical_basis()
    );
    assert_eq!(
        exact.result_shape().relations()[0]
            .predicate()
            .unwrap()
            .field(),
        None
    );
    assert_eq!(
        identity_relation_definition(IdentityParameter::Missing).unwrap_err(),
        ApplicationQueryDefinitionDenial::UnknownPredicateParameter
    );
    assert_eq!(
        identity_relation_definition(IdentityParameter::WrongFamily).unwrap_err(),
        ApplicationQueryDefinitionDenial::PredicateParameterMismatch
    );
}

enum IdentityParameter {
    Exact,
    Missing,
    WrongFamily,
}
fn identity_relation_definition(
    parameter: IdentityParameter,
) -> Result<ErasedApplicationQueryDefinition, ApplicationQueryDefinitionDenial> {
    use crate::application_schema::{EntityRefApplicationValueBinding, U64ApplicationValueBinding};
    let entity = ApplicationEntityRef::<Schema, Entity>::from_schema_identifier("Entity");
    let related = ApplicationQueryResultRelationRef::<
        Query,
        RelationSlot,
        Schema,
        Relation,
        Entity,
        Entity,
        ForwardResultTraversal,
        ManyResults,
    >::forward_many("related", relation_reference());
    let coordinate = ApplicationQueryParameterRef::<
        Query,
        PredicateParameter,
        EntityRefApplicationValueBinding,
    >::from_query_identifier("selected");
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
    .relation_where_entity(related, nested, coordinate)
    .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(query_reference())
        .root(entity)
        .scope(entity)
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public();
    let definition = match parameter {
        IdentityParameter::Exact => definition.parameter(coordinate),
        IdentityParameter::Missing => definition,
        IdentityParameter::WrongFamily => {
            definition.parameter(ApplicationQueryParameterRef::<
                Query,
                PredicateParameter,
                U64ApplicationValueBinding,
            >::from_query_identifier("selected"))
        }
    };
    definition
        .build()
        .map(|definition| definition.into_erased())
}
