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
pub struct PlanarRead {
    pub body_key: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanarReadResult {
    pub y: PositiveLength,
}
pub struct PlanarReadParameters;
worth_query_structured_value_binding!(pub PlanarReadInputBinding for PlanarRead {identity:"worth.query.certification.planar-read-input.v1"});
worth_query_structured_value_binding!(pub PlanarReadParametersBinding for PlanarReadParameters {identity:"worth.query.certification.planar-read-parameters.v1"});
worth_query_structured_value_binding!(pub PlanarReadResultBinding for PlanarReadResult {identity:"worth.query.certification.planar-read-result.v1"});
pub struct PlanarQuery;
pub struct PositionYSlot;
worth_query_portable_type!(PositionYSlot => "worth.query.certification.planar-position-y-slot.v1");
impl<Schema: TopologySchemaBinding> ApplicationQueryMarkerIdentity<Schema> for PlanarQuery {
    type ParameterBinding = PlanarReadParametersBinding;
    type ResultBinding = PlanarReadResultBinding;
    type Scope = Body;
    const IDENTIFIER: &'static str = "PlanarQuery";
    const QUERY_TYPE_NAME: &'static str = "worth.query.certification.planar-query.v1";
    const SCOPE_TYPE_NAME: &'static str = "Body";
}
pub fn planar_query_definition<Schema: TopologySchemaBinding>(
) -> ApplicationQueryDefinition<Schema, PlanarQuery, PlanarReadParameters, PlanarReadResult, Body> {
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        PlanarQuery,
        Body,
        PlanarReadResult,
        PlanarReadResultBinding,
    >::new(Body::reference())
    .field(position_y_result())
    .build();
    ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::<
        Schema,
        PlanarQuery,
        PlanarReadParameters,
        PlanarReadResult,
        Body,
    >::from_declaration())
    .root(Body::reference())
    .scope(Body::reference())
    .result_shape(shape)
    .cardinality(ApplicationQueryCardinality::ExactlyOne)
    .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 1))
    .disclosure(ApplicationQueryDisclosureContract::public())
    .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
    .lanes(ApplicationQueryLaneEligibility::one_shot())
    .public()
    .build()
    .expect("planar query is canonical")
}
fn position_y_result<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    PlanarQuery,
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
impl<Schema: TopologySchemaBinding> WorthQueryApplicationProjection<Schema, PlanarQuery>
    for PlanarReadResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, PlanarQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            y: row.field(position_y_result())?,
        })
    }
}
pub struct PlanarReadBinding<Schema>(PhantomData<fn() -> Schema>);
pub type PlanarReadScope<Schema> = ApplicationQueryFieldScope<
    Schema,
    Body,
    PlanarPosition,
    BodyKey,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl<Schema: TopologySchemaBinding> ApplicationQueryBinding<Schema> for PlanarReadBinding<Schema> {
    type Input = PlanarRead;
    type InputBinding = PlanarReadInputBinding;
    type Query = PlanarQuery;
    type ParameterBinding = PlanarReadParametersBinding;
    type ResultBinding = PlanarReadResultBinding;
    type ScopeBinding = PlanarReadScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.certification.planar-read.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::bounded(1, 64);
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
impl<Schema: TopologySchemaBinding> ApplicationQueryIntent<Schema> for PlanarRead {
    type Binding = PlanarReadBinding<Schema>;
    fn parameters(&self) -> ApplicationQueryParameterSet<PlanarQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> PlanarReadScope<Schema> {
        PlanarReadScope::new(BodyKey::reference(), self.body_key)
    }
}
