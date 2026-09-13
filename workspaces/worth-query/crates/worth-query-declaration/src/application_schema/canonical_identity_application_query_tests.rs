use crate::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
        ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
        ApplicationQueryResultRelationRef, ApplicationQueryResultShapeBuilder, ExactlyOneResult,
        ForwardResultTraversal, ManyResults, OptionalOneResult,
    },
    application_schema::{ApplicationEntityRef, ApplicationRelationRef, ApplicationSchemaMember},
};

use super::query_fixture::QueryMarkerResultBinding;
use super::{identity, QueryEntity, QueryMarker, QueryResult, QuerySchema};

struct QueryChild;
struct QueryRelation;
struct QueryChildSlot;

worth_query_portable_type!(QueryChildSlot => "worth.query.test.canonical-query-child-slot.v1");
crate::worth_query_structured_value_binding!(
    QueryChildResultBinding for () {
        identity: "worth.rust.unit"
    }
);

#[test]
fn nested_relation_cardinality_changes_schema_identity() {
    let optional = identity(&[application_query_with_relation(
        ApplicationQueryCardinality::OptionalOne,
    )]);
    let one = identity(&[application_query_with_relation(
        ApplicationQueryCardinality::ExactlyOne,
    )]);
    let many = identity(&[application_query_with_relation(
        ApplicationQueryCardinality::Many,
    )]);

    assert_ne!(optional, one);
    assert_ne!(one, many);
    assert_ne!(optional, many);
}

fn application_query_with_relation(
    cardinality: ApplicationQueryCardinality,
) -> ApplicationSchemaMember {
    let entity =
        ApplicationEntityRef::<QuerySchema, QueryEntity>::from_schema_identifier("QueryEntity");
    let child =
        ApplicationEntityRef::<QuerySchema, QueryChild>::from_schema_identifier("QueryChild");
    let relation = ApplicationRelationRef::<
        QuerySchema,
        QueryRelation,
        QueryEntity,
        QueryChild,
    >::from_schema_identifiers(
        "QueryRelation", "QueryEntity", "QueryChild",
        crate::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    );
    let nested = ApplicationQueryResultShapeBuilder::<
        QuerySchema,
        QueryMarker,
        QueryChild,
        (),
        QueryChildResultBinding,
    >::new(child);
    let shape = ApplicationQueryResultShapeBuilder::<
        QuerySchema,
        QueryMarker,
        QueryEntity,
        QueryResult,
        QueryMarkerResultBinding,
    >::new(entity);
    let shape = match cardinality {
        ApplicationQueryCardinality::OptionalOne => shape.relation(
            ApplicationQueryResultRelationRef::<
                QueryMarker,
                QueryChildSlot,
                QuerySchema,
                QueryRelation,
                QueryEntity,
                QueryChild,
                ForwardResultTraversal,
                OptionalOneResult,
            >::forward_optional("child", relation),
            nested,
        ),
        ApplicationQueryCardinality::ExactlyOne => shape.relation(
            ApplicationQueryResultRelationRef::<
                QueryMarker,
                QueryChildSlot,
                QuerySchema,
                QueryRelation,
                QueryEntity,
                QueryChild,
                ForwardResultTraversal,
                ExactlyOneResult,
            >::forward_one("child", relation),
            nested,
        ),
        ApplicationQueryCardinality::Many => shape.relation(
            ApplicationQueryResultRelationRef::<
                QueryMarker,
                QueryChildSlot,
                QuerySchema,
                QueryRelation,
                QueryEntity,
                QueryChild,
                ForwardResultTraversal,
                ManyResults,
            >::forward_many("child", relation),
            nested,
        ),
    }
    .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(QueryMarker::reference())
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
        .unwrap();
    assert_eq!(
        definition.result_shape().result_type(),
        "worth.query.test.canonical-query-result.v1"
    );
    assert_eq!(
        definition.result_shape().relations()[0]
            .nested_shape()
            .result_type(),
        "worth.rust.unit"
    );
    ApplicationSchemaMember::ApplicationQuery {
        definition: definition.into_erased(),
    }
}
