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

use super::*;

#[derive(Clone, Debug)]
pub struct PlanarOutputRead {
    pub body_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanarOutputReadResult {
    pub body_key: String,
    pub successor_body_key: String,
    pub value: PositiveLength,
}

pub struct PlanarOutputReadParameters;
worth_query_structured_value_binding!(pub PlanarOutputReadInputBinding for PlanarOutputRead {
    identity: "worth.query.certification.planar-output-read-input.v1"
});
worth_query_structured_value_binding!(pub PlanarOutputReadParametersBinding for PlanarOutputReadParameters {
    identity: "worth.query.certification.planar-output-read-parameters.v1"
});
worth_query_structured_value_binding!(pub PlanarOutputReadResultBinding for PlanarOutputReadResult {
    identity: "worth.query.certification.planar-output-read-result.v1"
});

pub struct PlanarOutputQuery;
pub struct PlanarOutputBodyKeySlot;
pub struct PlanarOutputSuccessorSlot;
pub struct PlanarOutputSuccessorBodyKeySlot;
pub struct PlanarOutputValueSlot;
worth_query_portable_type!(PlanarOutputBodyKeySlot => "worth.query.certification.planar-output-body-key-slot.v1");
worth_query_portable_type!(PlanarOutputSuccessorSlot => "worth.query.certification.planar-output-successor-slot.v1");
worth_query_portable_type!(PlanarOutputSuccessorBodyKeySlot => "worth.query.certification.planar-output-successor-body-key-slot.v1");
worth_query_portable_type!(PlanarOutputValueSlot => "worth.query.certification.planar-output-value-slot.v1");

impl<Schema: TopologySchemaBinding> ApplicationQueryMarkerIdentity<Schema> for PlanarOutputQuery {
    type ParameterBinding = PlanarOutputReadParametersBinding;
    type ResultBinding = PlanarOutputReadResultBinding;
    type Scope = Body;

    const IDENTIFIER: &'static str = "PlanarOutputQuery";
    const QUERY_TYPE_NAME: &'static str = "worth.query.certification.planar-output-query.v1";
    const SCOPE_TYPE_NAME: &'static str = "Body";
}

pub fn planar_output_query_definition<Schema: TopologySchemaBinding>() -> ApplicationQueryDefinition<
    Schema,
    PlanarOutputQuery,
    PlanarOutputReadParameters,
    PlanarOutputReadResult,
    Body,
> {
    let successor_shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        PlanarOutputQuery,
        Body,
        PlanarOutputReadResult,
        PlanarOutputReadResultBinding,
    >::new(Body::reference())
    .field(successor_body_key_result());
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        PlanarOutputQuery,
        Body,
        PlanarOutputReadResult,
        PlanarOutputReadResultBinding,
    >::new(Body::reference())
    .field(body_key_result())
    .field(output_value_result())
    .relation(output_successor_result(), successor_shape)
    .build();
    ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::<
        Schema,
        PlanarOutputQuery,
        PlanarOutputReadParameters,
        PlanarOutputReadResult,
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
    .expect("planar output query is canonical")
}

fn body_key_result<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    PlanarOutputQuery,
    PlanarOutputBodyKeySlot,
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

fn output_value_result<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    PlanarOutputQuery,
    PlanarOutputValueSlot,
    Schema,
    Body,
    Geometry,
    Length,
    PositiveLength,
    ReadWrite,
    EqualityPredicate,
    DeclaredApplicationUnit<Metre, <TopologyLengthBinding as ApplicationScalarValueBinding>::Unit>,
> {
    ApplicationQueryResultFieldRef::new("value", Length::reference())
}

fn output_successor_result<Schema: TopologySchemaBinding>() -> ApplicationQueryResultRelationRef<
    PlanarOutputQuery,
    PlanarOutputSuccessorSlot,
    Schema,
    PlanarSuccessor,
    Body,
    Body,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("successor", PlanarSuccessor::reference())
}

fn successor_body_key_result<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    PlanarOutputQuery,
    PlanarOutputSuccessorBodyKeySlot,
    Schema,
    Body,
    PlanarPosition,
    BodyKey,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("successor_body_key", BodyKey::reference())
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProjection<Schema, PlanarOutputQuery>
    for PlanarOutputReadResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, PlanarOutputQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            body_key: row.field(body_key_result())?,
            successor_body_key: row
                .one(output_successor_result())?
                .field(successor_body_key_result())?,
            value: row.field(output_value_result())?,
        })
    }
}

pub struct PlanarOutputReadBinding<Schema>(PhantomData<fn() -> Schema>);
pub type PlanarOutputReadScope<Schema> = ApplicationQueryFieldScope<
    Schema,
    Body,
    PlanarPosition,
    BodyKey,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl<Schema: TopologySchemaBinding> ApplicationQueryBinding<Schema>
    for PlanarOutputReadBinding<Schema>
{
    type Input = PlanarOutputRead;
    type InputBinding = PlanarOutputReadInputBinding;
    type Query = PlanarOutputQuery;
    type ParameterBinding = PlanarOutputReadParametersBinding;
    type ResultBinding = PlanarOutputReadResultBinding;
    type ScopeBinding = PlanarOutputReadScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;

    const IDENTITY: &'static str = "worth.query.certification.planar-output-read.v1";
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

impl<Schema: TopologySchemaBinding> ApplicationQueryIntent<Schema> for PlanarOutputRead {
    type Binding = PlanarOutputReadBinding<Schema>;

    fn parameters(&self) -> ApplicationQueryParameterSet<PlanarOutputQuery> {
        ApplicationQueryParameterSet::new()
    }

    fn into_scope(self) -> PlanarOutputReadScope<Schema> {
        PlanarOutputReadScope::new(BodyKey::reference(), self.body_key)
    }
}
