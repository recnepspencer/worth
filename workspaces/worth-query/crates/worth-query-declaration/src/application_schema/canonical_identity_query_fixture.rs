use crate::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
    ApplicationQueryDependencyCeiling, ApplicationQueryDisclosureContract,
    ApplicationQueryLaneEligibility, ApplicationQueryResultFieldRef,
    ApplicationQueryResultShapeBuilder,
};
use crate::application_schema::{
    ApplicationEntityRef, ApplicationFieldRef, ApplicationSchemaMember, EqualityPredicate,
    NoApplicationUnit, ReadOnly,
};

pub(super) struct QuerySchema;
pub(super) struct QueryEntity;
struct QueryAspect;
struct QueryField;
pub(super) struct QueryParameters;
pub(super) struct QueryResult;
struct QueryFieldSlot;

crate::worth_query_structured_value_binding!(pub(super) QueryMarkerParametersBinding for QueryParameters { identity: "QueryParameters" });
crate::worth_query_structured_value_binding!(pub(super) QueryMarkerResultBinding for QueryResult { identity: "worth.query.test.canonical-query-result.v1" });
crate::worth_query_application_query!(
    pub(super) QueryMarker for QuerySchema,
    identity "QueryMarker",
    parameters QueryMarkerParametersBinding,
    result QueryMarkerResultBinding,
    scope QueryEntity => "QueryEntity",
    name "query"
);
worth_query_portable_type!(QueryResult => "worth.query.test.canonical-query-result.v1");
worth_query_portable_type!(QueryFieldSlot => "worth.query.test.canonical-query-field-slot.v1");

impl crate::application_schema::DeclaredApplicationFieldValue for QueryField {
    type Value = u64;
    type Binding = crate::application_schema::U64ApplicationValueBinding;
    const PRESENCE: crate::application_schema::ApplicationFieldPresence =
        crate::application_schema::ApplicationFieldPresence::Required;
}

impl crate::application_schema::RequiredApplicationFieldValue for QueryField {}

pub(super) fn application_query(output_name: &'static str) -> ApplicationSchemaMember {
    let entity =
        ApplicationEntityRef::<QuerySchema, QueryEntity>::from_schema_identifier("QueryEntity");
    let field = ApplicationFieldRef::<
        QuerySchema,
        QueryEntity,
        QueryAspect,
        QueryField,
        u64,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_identifiers("QueryEntity", "QueryAspect", "QueryField");
    let result_field = ApplicationQueryResultFieldRef::<
        QueryMarker,
        QueryFieldSlot,
        QuerySchema,
        QueryEntity,
        QueryAspect,
        QueryField,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    >::new(output_name, field);
    let shape = ApplicationQueryResultShapeBuilder::<
        QuerySchema,
        QueryMarker,
        QueryEntity,
        QueryResult,
        QueryMarkerResultBinding,
    >::new(entity)
    .field(result_field)
    .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(QueryMarker::reference())
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
        .unwrap();
    ApplicationSchemaMember::ApplicationQuery {
        definition: definition.into_erased(),
    }
}
