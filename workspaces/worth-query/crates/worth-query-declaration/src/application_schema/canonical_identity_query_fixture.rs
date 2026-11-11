use crate::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryResultFieldRef, ApplicationQueryResultShapeBuilder,
};
use crate::application_schema::{ApplicationEntityRef, ApplicationSchemaMember};

pub(super) struct QuerySchema;
pub(super) struct QueryEntity;
pub(super) struct QueryAspect;
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

pub(super) fn application_query(output_name: &'static str) -> ApplicationSchemaMember {
    let entity =
        ApplicationEntityRef::<QuerySchema, QueryEntity>::from_schema_identifier("QueryEntity");
    let field = QueryField::at("QueryField");
    let result_field = ApplicationQueryResultFieldRef::<
        QueryMarker,
        QueryFieldSlot,
        QuerySchema,
        QueryField,
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

use crate::application_schema::fragment;
pub(super) struct QueryModel;
impl fragment::Fragment for QueryModel {
    const NAME: &'static str = "query_model";
    type Bindings = QueryBindings;
    type Members = (QueryEntity,);
}
pub(super) struct QueryBindings;
impl fragment::BindingSet for QueryBindings {
    type Work<'a> = crate::application_schema::ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut crate::application_schema::ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for QueryBindings {
    type Binding = crate::application_schema::U64ApplicationValueBinding;
    const WRITABLE: bool = false;
}
impl fragment::Entity for QueryEntity {
    const NAME: &'static str = "QueryEntity";
    type Aspects = (QueryAspect,);
    type Relations = ();
}
impl fragment::Aspect for QueryAspect {
    const NAME: &'static str = "QueryAspect";
    type Row = QueryRow0;
}
pub(super) struct QueryRow0(u64);
impl fragment::Record<QueryRow0> for QueryBindings {
    fn record() -> impl fragment::Shape<QueryRow0, Self> {
        fragment::field("QueryField", |row: &QueryRow0| row.0)
            .equality_queryable(true)
            .build(
                |value, _: &mut crate::application_schema::ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(QueryRow0(value))
                },
            )
    }
}
type QueryField = fragment::FieldSelection<QueryModel, QueryEntity, QueryAspect, u64>;
