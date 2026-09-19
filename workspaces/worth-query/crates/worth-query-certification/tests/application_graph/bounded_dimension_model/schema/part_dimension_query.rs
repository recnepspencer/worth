//! The ordinary read that reports what dimension a part actually carries.
//!
//! Every court assertion about a performed or denied mutation is checked
//! against this read, so the evidence is the committed value rather than the
//! outcome variant alone.

use worth_query_host::facade::declaration;
use worth_query_host::facade::declaration::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryResultFieldRef, ApplicationQueryResultShapeBuilder,
};
use worth_query_host::facade::{
    worth_query_application_query, worth_query_portable_type, worth_query_structured_value_binding,
};

use super::{BoundedDimensionSchema, Part, PartDimensionField, PartFacts, PartIdentityField};

pub struct PartQueryParameters;
pub struct PartIdentitySlot;
pub struct PartDimensionSlot;
worth_query_portable_type!(PartIdentitySlot => "worth.query.certification.bounded-dimension.identity-slot.v1");
worth_query_portable_type!(PartDimensionSlot => "worth.query.certification.bounded-dimension.dimension-slot.v1");

/// One part as an ordinary read reports it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartDimensionRow {
    pub identity: String,
    pub dimension: u64,
}
worth_query_portable_type!(PartDimensionRow => "worth.query.certification.bounded-dimension.row.v1");

worth_query_structured_value_binding!(pub PartQueryParametersBinding for PartQueryParameters {
    identity: "PartQueryParameters"
});
worth_query_structured_value_binding!(pub PartDimensionRowBinding for PartDimensionRow {
    identity: "worth.query.certification.bounded-dimension.row.v1"
});
worth_query_application_query!(
    pub PartDimensionQuery for BoundedDimensionSchema,
    identity "PartDimensionQuery",
    parameters PartQueryParametersBinding,
    result PartDimensionRowBinding,
    scope Part => "Part",
    name "part_dimension_query"
);

pub fn part_dimension_query_definition() -> ApplicationQueryDefinition<
    BoundedDimensionSchema,
    PartDimensionQuery,
    PartQueryParameters,
    PartDimensionRow,
    Part,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        BoundedDimensionSchema,
        PartDimensionQuery,
        Part,
        PartDimensionRow,
        PartDimensionRowBinding,
    >::new(Part::reference())
    .field(identity_result())
    .field(dimension_result())
    .build();
    ApplicationQueryDefinitionBuilder::declare(PartDimensionQuery::reference())
        .root(Part::reference())
        .scope(Part::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 5))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .expect("the part dimension query is canonical")
}

type ResultField<Slot, Field, Value, Write> = ApplicationQueryResultFieldRef<
    PartDimensionQuery,
    Slot,
    BoundedDimensionSchema,
    Part,
    PartFacts,
    Field,
    Value,
    Write,
    declaration::application_schema::EqualityPredicate,
    declaration::application_schema::NoApplicationUnit,
>;

pub(super) fn identity_result() -> ResultField<
    PartIdentitySlot,
    PartIdentityField,
    String,
    declaration::application_schema::ReadOnly,
> {
    ApplicationQueryResultFieldRef::new("identity", PartIdentityField::reference())
}

pub(super) fn dimension_result() -> ResultField<
    PartDimensionSlot,
    PartDimensionField,
    u64,
    declaration::application_schema::ReadWrite,
> {
    ApplicationQueryResultFieldRef::new("dimension", PartDimensionField::reference())
}
