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
pub struct SuccessorRead {
    pub body_key: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessorReadResult {
    pub body_key: String,
    pub y: PositiveLength,
}
pub struct SuccessorReadParameters;
worth_query_structured_value_binding!(pub SuccessorReadInputBinding for SuccessorRead {identity:"worth.query.certification.courtroom-successor-read-input.v1"});
worth_query_structured_value_binding!(pub SuccessorReadParametersBinding for SuccessorReadParameters {identity:"worth.query.certification.courtroom-successor-read-parameters.v1"});
worth_query_structured_value_binding!(pub SuccessorReadResultBinding for SuccessorReadResult {identity:"worth.query.certification.courtroom-successor-read-result.v1"});
pub struct SuccessorQuery;
pub struct PositionYSlot;
pub struct BodyKeySlot;
pub struct PlanarSuccessorSlot;
pub struct SuccessorPositionYSlot;
pub struct SecondSuccessorSlot;
pub struct SecondSuccessorPositionYSlot;
worth_query_portable_type!(PositionYSlot => "worth.query.certification.courtroom-successor-position-y-slot.v1");
worth_query_portable_type!(BodyKeySlot => "worth.query.certification.courtroom-successor-body-key-slot.v1");
worth_query_portable_type!(PlanarSuccessorSlot => "worth.query.certification.courtroom-successor-successor-slot.v1");
worth_query_portable_type!(SuccessorPositionYSlot => "worth.query.certification.successor-position-y-slot.v1");
worth_query_portable_type!(SecondSuccessorSlot => "worth.query.certification.second-successor-slot.v1");
worth_query_portable_type!(SecondSuccessorPositionYSlot => "worth.query.certification.second-successor-position-y-slot.v1");
impl<Schema: TopologySchemaBinding> ApplicationQueryMarkerIdentity<Schema> for SuccessorQuery {
    type ParameterBinding = SuccessorReadParametersBinding;
    type ResultBinding = SuccessorReadResultBinding;
    type Scope = Body;
    const IDENTIFIER: &'static str = "SuccessorQuery";
    const QUERY_TYPE_NAME: &'static str = "worth.query.certification.courtroom-successor-query.v1";
    const SCOPE_TYPE_NAME: &'static str = "Body";
}
pub fn successor_query_definition<Schema: TopologySchemaBinding>() -> ApplicationQueryDefinition<
    Schema,
    SuccessorQuery,
    SuccessorReadParameters,
    SuccessorReadResult,
    Body,
> {
    let second_successor_shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        SuccessorQuery,
        Body,
        SuccessorReadResult,
        SuccessorReadResultBinding,
    >::new(Body::reference())
    .field(second_successor_position_y_result());
    let successor_shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        SuccessorQuery,
        Body,
        SuccessorReadResult,
        SuccessorReadResultBinding,
    >::new(Body::reference())
    .field(successor_position_y_result())
    .relation(second_successor_result(), second_successor_shape);
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        SuccessorQuery,
        Body,
        SuccessorReadResult,
        SuccessorReadResultBinding,
    >::new(Body::reference())
    .field(body_key_result())
    .field(position_y_result())
    .relation(planar_successor_result(), successor_shape)
    .build();
    ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::<
        Schema,
        SuccessorQuery,
        SuccessorReadParameters,
        SuccessorReadResult,
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
    SuccessorQuery,
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
    SuccessorQuery,
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
    SuccessorQuery,
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
    SuccessorQuery,
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
    SuccessorQuery,
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
impl<Schema: TopologySchemaBinding> WorthQueryApplicationProjection<Schema, SuccessorQuery>
    for SuccessorReadResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, SuccessorQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            body_key: row.field(body_key_result())?,
            y: row
                .one(planar_successor_result())?
                .field(successor_position_y_result())?,
        })
    }
}
fn body_key_result<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    SuccessorQuery,
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
pub struct SuccessorReadBinding<Schema>(PhantomData<fn() -> Schema>);
pub type SuccessorReadScope<Schema> = ApplicationQueryFieldScope<
    Schema,
    Body,
    PlanarPosition,
    BodyKey,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl<Schema: TopologySchemaBinding> ApplicationQueryBinding<Schema>
    for SuccessorReadBinding<Schema>
{
    type Input = SuccessorRead;
    type InputBinding = SuccessorReadInputBinding;
    type Query = SuccessorQuery;
    type ParameterBinding = SuccessorReadParametersBinding;
    type ResultBinding = SuccessorReadResultBinding;
    type ScopeBinding = SuccessorReadScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.certification.courtroom-successor-read.v1";
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
impl<Schema: TopologySchemaBinding> ApplicationQueryIntent<Schema> for SuccessorRead {
    type Binding = SuccessorReadBinding<Schema>;
    fn parameters(&self) -> ApplicationQueryParameterSet<SuccessorQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> SuccessorReadScope<Schema> {
        SuccessorReadScope::new(BodyKey::reference(), self.body_key)
    }
}
