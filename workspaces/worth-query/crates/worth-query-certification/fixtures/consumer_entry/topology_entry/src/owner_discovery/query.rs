//! A separate query binding makes the discovery READ ability part of installed meaning.
use super::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_portable_type,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

#[derive(Clone, Debug)]
pub struct OwnerDiscoveryRead {
    pub body_key: String,
}
worth_query_structured_value_binding!(pub OwnerDiscoveryInputBinding for OwnerDiscoveryRead {
    identity: "worth.query.certification.owner-discovery-input.v1"
});
pub struct OwnerDiscoveryQuery;
pub struct OwnerBodyKeySlot;
pub struct OwnerPositionYSlot;
worth_query_portable_type!(OwnerBodyKeySlot => "worth.query.certification.owner-discovery-body.v1");
worth_query_portable_type!(OwnerPositionYSlot => "worth.query.certification.owner-discovery-y.v1");
impl<S: TopologySchemaBinding> ApplicationQueryMarkerIdentity<S> for OwnerDiscoveryQuery {
    type ParameterBinding = PlanarReadParametersBinding;
    type ResultBinding = PlanarReadResultBinding;
    type Scope = Body;
    const IDENTIFIER: &'static str = "OwnerDiscoveryQuery";
    const QUERY_TYPE_NAME: &'static str = "worth.query.certification.owner-discovery-query.v1";
    const SCOPE_TYPE_NAME: &'static str = "Body";
}
pub struct OwnerDiscoveryBinding<S>(PhantomData<fn() -> S>);
impl<S: TopologySchemaBinding> ApplicationQueryBinding<S> for OwnerDiscoveryBinding<S> {
    type Input = OwnerDiscoveryRead;
    type InputBinding = OwnerDiscoveryInputBinding;
    type Query = OwnerDiscoveryQuery;
    type ParameterBinding = PlanarReadParametersBinding;
    type ResultBinding = PlanarReadResultBinding;
    type ScopeBinding = PlanarReadScope<S>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.certification.owner-discovery-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
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
impl<S: TopologySchemaBinding> ApplicationQueryIntent<S> for OwnerDiscoveryRead {
    type Binding = OwnerDiscoveryBinding<S>;
    fn parameters(&self) -> ApplicationQueryParameterSet<OwnerDiscoveryQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> PlanarReadScope<S> {
        PlanarReadScope::new(BodyKey::reference(), self.body_key)
    }
}
fn body<S: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    OwnerDiscoveryQuery,
    OwnerBodyKeySlot,
    S,
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
fn y<S: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    OwnerDiscoveryQuery,
    OwnerPositionYSlot,
    S,
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
pub(super) fn definition<S: TopologySchemaBinding>(
) -> ApplicationQueryDefinition<S, OwnerDiscoveryQuery, PlanarReadParameters, PlanarReadResult, Body>
{
    let shape = ApplicationQueryResultShapeBuilder::<
        S,
        OwnerDiscoveryQuery,
        Body,
        PlanarReadResult,
        PlanarReadResultBinding,
    >::new(Body::reference())
    .field(body())
    .field(y())
    .build();
    ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::from_declaration())
        .root(Body::reference())
        .scope(Body::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(OwnerOnlyDiscovery::reference())
        .build()
        .expect("owner-only discovery declares a normal scoped READ ability")
}
impl<S: TopologySchemaBinding> WorthQueryApplicationProjection<S, OwnerDiscoveryQuery>
    for PlanarReadResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, S, OwnerDiscoveryQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            body_key: row.field(body())?,
            y: row.field(y())?,
        })
    }
}
