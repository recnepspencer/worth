//! Read the published total independently of the immutable producer source.
use super::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    application_query::*, worth_query_portable_type, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

#[derive(Clone, Debug)]
pub struct OutputRead {
    pub key: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputTotal {
    pub key: String,
    pub total: u64,
}
pub struct Parameters;
worth_query_structured_value_binding!(pub InputValue for OutputRead {identity:"quote-total.input"});
worth_query_structured_value_binding!(pub ParameterValue for Parameters {identity:"quote-total.parameters"});
worth_query_structured_value_binding!(pub ResultValue for OutputTotal {identity:"quote-total.result"});
pub struct OutputQuery;
pub struct KeySlot;
pub struct TotalSlot;
worth_query_portable_type!(KeySlot => "quote-total.key-slot");
worth_query_portable_type!(TotalSlot => "quote-total.total-slot");
impl<S: TopologySchemaBinding> ApplicationQueryMarkerIdentity<S> for OutputQuery {
    type ParameterBinding = ParameterValue;
    type ResultBinding = ResultValue;
    type Scope = Body;
    const IDENTIFIER: &'static str = "QuoteCountOutputQuery";
    const QUERY_TYPE_NAME: &'static str = "quote-total.query";
    const SCOPE_TYPE_NAME: &'static str = "Body";
}
fn key<S: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    OutputQuery,
    KeySlot,
    S,
    Body,
    PlanarPosition,
    BodyKey,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("key", BodyKey::reference())
}
fn total<S: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    OutputQuery,
    TotalSlot,
    S,
    Body,
    QuoteOutputFacts,
    CountedTotal,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("total", CountedTotal::reference())
}
pub(super) fn definition<S: TopologySchemaBinding>(
) -> ApplicationQueryDefinition<S, OutputQuery, Parameters, OutputTotal, Body> {
    let shape =
        ApplicationQueryResultShapeBuilder::<S, OutputQuery, Body, OutputTotal, ResultValue>::new(
            Body::reference(),
        )
        .field(key())
        .field(total())
        .build();
    ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::<
        S,
        OutputQuery,
        Parameters,
        OutputTotal,
        Body,
    >::from_declaration())
    .root(Body::reference())
    .scope(Body::reference())
    .result_shape(shape)
    .cardinality(ApplicationQueryCardinality::ExactlyOne)
    .disclosure(ApplicationQueryDisclosureContract::public())
    .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
    .lanes(ApplicationQueryLaneEligibility::one_shot())
    .public()
    .build()
    .unwrap()
}
impl<S: TopologySchemaBinding> WorthQueryApplicationProjection<S, OutputQuery> for OutputTotal {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, S, OutputQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key())?,
            total: row.field(total())?,
        })
    }
}
pub struct OutputReadBinding<S>(PhantomData<fn() -> S>);
impl<S: TopologySchemaBinding> ApplicationQueryBinding<S> for OutputReadBinding<S> {
    type Input = OutputRead;
    type InputBinding = InputValue;
    type Query = OutputQuery;
    type ParameterBinding = ParameterValue;
    type ResultBinding = ResultValue;
    type ScopeBinding = PlanarReadScope<S>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "quote-total.read";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::bounded(1, 128);
    fn scope_field() -> ApplicationFieldRef<
        S,
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
        S,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
}
impl<S: TopologySchemaBinding> ApplicationQueryIntent<S> for OutputRead {
    type Binding = OutputReadBinding<S>;
    fn parameters(&self) -> ApplicationQueryParameterSet<OutputQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> PlanarReadScope<S> {
        PlanarReadScope::new(BodyKey::reference(), self.key)
    }
}
