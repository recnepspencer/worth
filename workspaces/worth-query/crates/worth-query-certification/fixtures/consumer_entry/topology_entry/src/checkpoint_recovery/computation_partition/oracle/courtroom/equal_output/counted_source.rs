use super::*;
use std::marker::PhantomData;
use worth_query_consumer_values::PositiveLength;
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_portable_type,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

#[derive(Clone, Debug)]
pub struct CountedRead {
    pub body_key: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CountedReadResult {
    pub body_key: String,
    pub y: PositiveLength,
}
pub struct CountedReadParameters;
worth_query_structured_value_binding!(pub CountedReadInputBinding for CountedRead {identity:"worth.query.certification.courtroom-counted-read-input.v1"});
worth_query_structured_value_binding!(pub CountedReadParametersBinding for CountedReadParameters {identity:"worth.query.certification.courtroom-counted-read-parameters.v1"});
worth_query_structured_value_binding!(pub CountedReadResultBinding for CountedReadResult {identity:"worth.query.certification.courtroom-counted-read-result.v1"});
pub struct CountedQuery;
pub struct PositionYSlot;
pub struct BodyKeySlot;
pub struct PlanarSuccessorSlot;
pub struct SuccessorPositionYSlot;
pub struct SecondSuccessorSlot;
pub struct SecondSuccessorPositionYSlot;
worth_query_portable_type!(PositionYSlot => "worth.query.certification.courtroom-counted-position-y-slot.v1");
worth_query_portable_type!(BodyKeySlot => "worth.query.certification.courtroom-counted-body-key-slot.v1");
worth_query_portable_type!(PlanarSuccessorSlot => "worth.query.certification.courtroom-counted-successor-slot.v1");
worth_query_portable_type!(SuccessorPositionYSlot => "worth.query.certification.successor-position-y-slot.v1");
worth_query_portable_type!(SecondSuccessorSlot => "worth.query.certification.second-successor-slot.v1");
worth_query_portable_type!(SecondSuccessorPositionYSlot => "worth.query.certification.second-successor-position-y-slot.v1");
impl<Schema: TopologySchemaBinding> ApplicationQueryMarkerIdentity<Schema> for CountedQuery {
    type ParameterBinding = CountedReadParametersBinding;
    type ResultBinding = CountedReadResultBinding;
    type Scope = Body;
    const IDENTIFIER: &'static str = "CountedQuery";
    const QUERY_TYPE_NAME: &'static str = "worth.query.certification.courtroom-counted-query.v1";
    const SCOPE_TYPE_NAME: &'static str = "Body";
}
pub fn counted_query_definition<Schema: TopologySchemaBinding>(
) -> ApplicationQueryDefinition<Schema, CountedQuery, CountedReadParameters, CountedReadResult, Body>
{
    let second_successor_shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        CountedQuery,
        Body,
        CountedReadResult,
        CountedReadResultBinding,
    >::new(Body::reference())
    .field(second_successor_position_y_result());
    let successor_shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        CountedQuery,
        Body,
        CountedReadResult,
        CountedReadResultBinding,
    >::new(Body::reference())
    .field(successor_position_y_result())
    .relation(second_successor_result(), second_successor_shape);
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        CountedQuery,
        Body,
        CountedReadResult,
        CountedReadResultBinding,
    >::new(Body::reference())
    .field(body_key_result())
    .field(position_y_result())
    .relation(planar_successor_result(), successor_shape)
    .build();
    ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::<
        Schema,
        CountedQuery,
        CountedReadParameters,
        CountedReadResult,
        Body,
    >::from_declaration())
    .root(Body::reference())
    .scope(Body::reference())
    .result_shape(shape)
    .cardinality(ApplicationQueryCardinality::ExactlyOne)
    .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(2, 2, 4))
    .disclosure(ApplicationQueryDisclosureContract::public())
    .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
    .lanes(ApplicationQueryLaneEligibility::one_shot())
    .public()
    .build()
    .expect("planar query is canonical")
}
fn successor_position_y_result<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    CountedQuery,
    SuccessorPositionYSlot,
    Schema,
    Body,
    PlanarPosition,
    PositionY,
    PositiveLength,
    ReadWrite,
    EqualityPredicate,
    DeclaredApplicationUnit<Metre, <TopologyLengthBinding as ApplicationScalarValueBinding>::Unit>,
> {
    ApplicationQueryResultFieldRef::new("successor_y", PositionY::reference())
}
fn planar_successor_result<Schema: TopologySchemaBinding>() -> ApplicationQueryResultRelationRef<
    CountedQuery,
    PlanarSuccessorSlot,
    Schema,
    PlanarSuccessor,
    Body,
    Body,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("successor", PlanarSuccessor::reference())
}
fn second_successor_result<Schema: TopologySchemaBinding>() -> ApplicationQueryResultRelationRef<
    CountedQuery,
    SecondSuccessorSlot,
    Schema,
    PlanarSuccessor,
    Body,
    Body,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("second_successor", PlanarSuccessor::reference())
}
fn second_successor_position_y_result<Schema: TopologySchemaBinding>(
) -> ApplicationQueryResultFieldRef<
    CountedQuery,
    SecondSuccessorPositionYSlot,
    Schema,
    Body,
    PlanarPosition,
    PositionY,
    PositiveLength,
    ReadWrite,
    EqualityPredicate,
    DeclaredApplicationUnit<Metre, <TopologyLengthBinding as ApplicationScalarValueBinding>::Unit>,
> {
    ApplicationQueryResultFieldRef::new("second_successor_y", PositionY::reference())
}
fn position_y_result<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    CountedQuery,
    PositionYSlot,
    Schema,
    Body,
    PlanarPosition,
    PositionY,
    PositiveLength,
    ReadWrite,
    EqualityPredicate,
    DeclaredApplicationUnit<Metre, <TopologyLengthBinding as ApplicationScalarValueBinding>::Unit>,
> {
    ApplicationQueryResultFieldRef::new("y", PositionY::reference())
}
impl<Schema: TopologySchemaBinding> WorthQueryApplicationProjection<Schema, CountedQuery>
    for CountedReadResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, CountedQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        CALLS.with(|n| n.set(n.get() + 1));
        Ok(Self {
            body_key: row.field(body_key_result())?,
            // A normalized projection deliberately has equal input despite a changed consumed output.
            y: {
                let observed = row
                    .one(planar_successor_result())?
                    .one(second_successor_result())?
                    .field(second_successor_position_y_result())?;
                length(PositiveLength::get(&observed) / PositiveLength::get(&observed))
            },
        })
    }
}
fn body_key_result<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    CountedQuery,
    BodyKeySlot,
    Schema,
    Body,
    PlanarPosition,
    BodyKey,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("body_key", BodyKey::reference())
}
pub struct CountedReadBinding<Schema>(PhantomData<fn() -> Schema>);
pub type CountedReadScope<Schema> = ApplicationQueryFieldScope<
    Schema,
    Body,
    PlanarPosition,
    BodyKey,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl<Schema: TopologySchemaBinding> ApplicationQueryBinding<Schema> for CountedReadBinding<Schema> {
    type Input = CountedRead;
    type InputBinding = CountedReadInputBinding;
    type Query = CountedQuery;
    type ParameterBinding = CountedReadParametersBinding;
    type ResultBinding = CountedReadResultBinding;
    type ScopeBinding = CountedReadScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.certification.courtroom-counted-read.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::bounded(1, 128);
    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Body,
        PlanarPosition,
        BodyKey,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        BodyKey::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
}
impl<Schema: TopologySchemaBinding> ApplicationQueryIntent<Schema> for CountedRead {
    type Binding = CountedReadBinding<Schema>;
    fn parameters(&self) -> ApplicationQueryParameterSet<CountedQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> CountedReadScope<Schema> {
        CountedReadScope::new(BodyKey::reference(), self.body_key)
    }
}

thread_local! { static CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
pub(super) fn take_calls() -> usize {
    CALLS.with(|n| n.replace(0))
}
