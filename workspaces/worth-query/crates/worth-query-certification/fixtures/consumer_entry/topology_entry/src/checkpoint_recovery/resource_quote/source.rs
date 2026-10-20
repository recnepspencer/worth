//! An authored row count is a real source fact, independent of geometry or policy.
use super::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    application_query::*, worth_query_aspect, worth_query_field, worth_query_portable_type,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

worth_query_aspect!(
    pub QuoteSourceFacts for S: TopologySchemaBinding, Body;
    identity = AspectIdentity(0x9174_10e1),
    revision = AspectContractRevision(1),
);
worth_query_aspect!(
    pub QuoteOutputFacts for S: TopologySchemaBinding, Body;
    identity = AspectIdentity(0x9174_10e2),
    revision = AspectContractRevision(1),
);
worth_query_field!(pub RequestedRows for S: TopologySchemaBinding, Body, QuoteSourceFacts:
    u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub CountedTotal for S: TopologySchemaBinding, Body, QuoteOutputFacts:
    u64 => U64ApplicationValueBinding, read_write, equality);
#[derive(Clone, Debug)]
pub struct Read {
    pub key: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Source {
    pub key: String,
    pub count: u64,
}
pub struct Parameters;
worth_query_structured_value_binding!(pub InputValue for Read {identity:"quote-count.input"});
worth_query_structured_value_binding!(pub ParameterValue for Parameters {identity:"quote-count.parameters"});
worth_query_structured_value_binding!(pub ResultValue for Source {identity:"quote-count.result"});
pub struct Query;
pub struct KeySlot;
pub struct CountSlot;
worth_query_portable_type!(KeySlot => "quote-count.key-slot");
worth_query_portable_type!(CountSlot => "quote-count.count-slot");
impl<S: TopologySchemaBinding> ApplicationQueryMarkerIdentity<S> for Query {
    type ParameterBinding = ParameterValue;
    type ResultBinding = ResultValue;
    type Scope = Body;
    const IDENTIFIER: &'static str = "QuoteCountQuery";
    const QUERY_TYPE_NAME: &'static str = "quote-count.query";
    const SCOPE_TYPE_NAME: &'static str = "Body";
}
fn key<S: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    Query,
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
fn count<S: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    Query,
    CountSlot,
    S,
    Body,
    QuoteSourceFacts,
    RequestedRows,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("count", RequestedRows::reference())
}
pub(super) fn definition<S: TopologySchemaBinding>(
) -> ApplicationQueryDefinition<S, Query, Parameters, Source, Body> {
    let shape = ApplicationQueryResultShapeBuilder::<S, Query, Body, Source, ResultValue>::new(
        Body::reference(),
    )
    .field(key())
    .field(count())
    .build();
    ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::<
        S,
        Query,
        Parameters,
        Source,
        Body,
    >::from_declaration())
    .root(Body::reference())
    .scope(Body::reference())
    .result_shape(shape)
    .cardinality(ApplicationQueryCardinality::ExactlyOne)
    .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 2))
    .disclosure(ApplicationQueryDisclosureContract::public())
    .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
    .lanes(ApplicationQueryLaneEligibility::one_shot())
    .public()
    .build()
    .unwrap()
}
impl<S: TopologySchemaBinding> WorthQueryApplicationProjection<S, Query> for Source {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, S, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key())?,
            count: row.field(count())?,
        })
    }
}
pub struct ReadBinding<S>(PhantomData<fn() -> S>);
impl<S: TopologySchemaBinding> ApplicationQueryBinding<S> for ReadBinding<S> {
    type Input = Read;
    type InputBinding = InputValue;
    type Query = Query;
    type ParameterBinding = ParameterValue;
    type ResultBinding = ResultValue;
    type ScopeBinding = PlanarReadScope<S>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "quote-count.read";
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
impl<S: TopologySchemaBinding> ApplicationQueryIntent<S> for Read {
    type Binding = ReadBinding<S>;
    fn parameters(&self) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> PlanarReadScope<S> {
        PlanarReadScope::new(BodyKey::reference(), self.key)
    }
}
