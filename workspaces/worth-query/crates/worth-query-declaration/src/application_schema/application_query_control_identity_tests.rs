use crate::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryOrderingDirection, ApplicationQueryParameterRef,
    ApplicationQueryResultFieldRef, ApplicationQueryResultShapeBuilder,
    ErasedApplicationQueryDefinition,
};

use super::ApplicationEntityRef;

struct Schema;
struct Entity;
struct Aspect;
struct Parameters;
struct QueryResult;
struct Parameter;
struct ResultSlot;

crate::worth_query_structured_value_binding!(QueryParametersBinding for Parameters { identity: "Parameters" });
crate::worth_query_structured_value_binding!(QueryResultBinding for QueryResult { identity: "worth.query.test.control-query-result.v1" });
crate::worth_query_application_query!(
    Query for Schema,
    identity "Query",
    parameters QueryParametersBinding,
    result QueryResultBinding,
    scope Entity => "Entity",
    name "query"
);
worth_query_portable_type!(QueryResult => "worth.query.test.control-query-result.v1");
worth_query_portable_type!(ResultSlot => "worth.query.test.control-result-slot.v1");

#[derive(Clone)]
struct QueryControlFixture {
    root: &'static str,
    scope: &'static str,
    output: &'static str,
    parameter: &'static str,
    cardinality: ApplicationQueryCardinality,
    disclosure: ApplicationQueryDisclosureContract,
    basis: ApplicationQueryBasisSupport,
    lanes: ApplicationQueryLaneEligibility,
    predicate: bool,
    ordering: ApplicationQueryOrderingDirection,
}

impl QueryControlFixture {
    fn baseline() -> Self {
        Self {
            root: "Entity",
            scope: "Entity",
            output: "value",
            parameter: "parameter",
            cardinality: ApplicationQueryCardinality::ExactlyOne,
            disclosure: ApplicationQueryDisclosureContract::public(),
            basis: ApplicationQueryBasisSupport::current_and_pinned(),
            lanes: ApplicationQueryLaneEligibility::one_shot(),
            predicate: true,
            ordering: ApplicationQueryOrderingDirection::Ascending,
        }
    }
}

#[test]
fn every_scalar_query_control_is_identity_bearing() {
    let baseline = QueryControlFixture::baseline();
    let baseline_basis = definition(&baseline).canonical_basis().clone();
    let mut variants = Vec::new();

    let mut changed = baseline.clone();
    changed.root = "OtherRoot";
    variants.push(("root", changed));
    let mut changed = baseline.clone();
    changed.scope = "OtherScope";
    variants.push(("scope", changed));
    let mut changed = baseline.clone();
    changed.output = "other_value";
    variants.push(("output", changed));
    let mut changed = baseline.clone();
    changed.parameter = "other_parameter";
    variants.push(("parameter", changed));
    let mut changed = baseline.clone();
    changed.cardinality = ApplicationQueryCardinality::OptionalOne;
    variants.push(("cardinality", changed));
    let mut changed = baseline.clone();
    changed.disclosure = ApplicationQueryDisclosureContract::installed_policy("private");
    variants.push(("disclosure posture", changed));
    let mut changed = baseline.clone();
    changed.disclosure = ApplicationQueryDisclosureContract::installed_policy("secret");
    variants.push(("disclosure classification", changed));
    let mut changed = baseline.clone();
    changed.basis = ApplicationQueryBasisSupport::current_and_pinned().with_preview();
    variants.push(("basis support", changed));
    let mut changed = baseline.clone();
    changed.lanes = ApplicationQueryLaneEligibility::one_shot().with_historical();
    variants.push(("lane eligibility", changed));
    let mut changed = baseline.clone();
    changed.predicate = false;
    variants.push(("predicate", changed));
    let mut changed = baseline;
    changed.ordering = ApplicationQueryOrderingDirection::Descending;
    variants.push(("ordering", changed));

    for (dimension, variant) in variants {
        assert_ne!(
            baseline_basis,
            definition(&variant).canonical_basis().clone(),
            "{dimension} must change the canonical query basis"
        );
    }
}

fn definition(fixture: &QueryControlFixture) -> ErasedApplicationQueryDefinition {
    let root = ApplicationEntityRef::<Schema, Entity>::from_schema_identifier("Entity");
    let scope = ApplicationEntityRef::<Schema, Entity>::from_schema_identifier(fixture.scope);
    let field = Field::at("Field");
    let result = ApplicationQueryResultFieldRef::<Query, ResultSlot, Schema, Field>::new(
        fixture.output,
        field,
    );
    let parameter = ApplicationQueryParameterRef::<
        Query,
        Parameter,
        crate::application_schema::U64ApplicationValueBinding,
    >::from_query_identifier(fixture.parameter);
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Entity,
        QueryResult,
        QueryResultBinding,
    >::new(root)
    .field(result)
    .build();
    let builder = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(root)
        .scope(scope)
        .result_shape(shape)
        .cardinality(fixture.cardinality)
        .disclosure(fixture.disclosure.clone())
        .basis_support(fixture.basis)
        .lanes(fixture.lanes)
        .public()
        .parameter(parameter)
        .order_by(result, fixture.ordering);
    let builder = if fixture.predicate {
        builder.where_equal(field, parameter)
    } else {
        builder
    };
    let mut definition = builder
        .build()
        .expect("one-axis identity fixture should remain valid")
        .into_erased()
        .into_parts();
    definition.root_entity = fixture.root.into();
    ErasedApplicationQueryDefinition::from_untrusted_parts(definition)
}

use super::fragment;
struct QueryModel;
impl fragment::Fragment for QueryModel {
    const NAME: &'static str = "query_model";
    type Bindings = QueryBindings;
    type Members = (Entity,);
}
struct QueryBindings;
impl fragment::BindingSet for QueryBindings {
    type Work<'a> = super::ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut super::ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for QueryBindings {
    type Binding = super::U64ApplicationValueBinding;
    const WRITABLE: bool = false;
    const EQUALITY_QUERYABLE: bool = true;
}
impl fragment::Entity for Entity {
    const NAME: &'static str = "Entity";
    type Aspects = (Aspect,);
    type Relations = ();
}
impl fragment::Aspect for Aspect {
    const NAME: &'static str = "Aspect";
    type Row = QueryRow0;
}
struct QueryRow0(u64);
impl fragment::Record<QueryRow0> for QueryBindings {
    fn record() -> impl fragment::Shape<QueryRow0, Self> {
        fragment::field("Field", |row: &QueryRow0| row.0).build(
            |value, _: &mut super::ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(QueryRow0(value))
            },
        )
    }
}
type Field = fragment::FieldSelection<QueryModel, Entity, Aspect, u64>;
