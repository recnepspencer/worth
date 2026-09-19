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
    let unfiltered = relation_definition(relation.clone());
    let filtered = filtered_relation_definition(relation);

    assert_ne!(unfiltered.canonical_basis(), filtered.canonical_basis());
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
    >::new(entity)
    .relation_where_equal(
        relation,
        nested,
        ApplicationFieldRef::<
            Schema,
            Entity,
            Aspect,
            Field,
            u64,
            ReadOnly,
            EqualityPredicate,
            NoApplicationUnit,
        >::from_schema_identifiers("Entity", "Aspect", "Field"),
        parameter,
    )
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
        .parameter(parameter)
        .build()
        .unwrap()
        .into_erased()
}
