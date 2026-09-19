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
pub struct PlanarDiscoveryRead {
    pub body_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanarDiscoveryResult {
    pub source_body_keys: Vec<String>,
}

pub struct PlanarDiscoveryParameters;
worth_query_structured_value_binding!(pub PlanarDiscoveryInputBinding for PlanarDiscoveryRead { identity: "worth.query.certification.planar-discovery-input.v1" });
worth_query_structured_value_binding!(pub PlanarDiscoveryParametersBinding for PlanarDiscoveryParameters { identity: "worth.query.certification.planar-discovery-parameters.v1" });
worth_query_structured_value_binding!(pub PlanarDiscoveryResultBinding for PlanarDiscoveryResult { identity: "worth.query.certification.planar-discovery-result.v1" });

pub struct PlanarDiscoveryQuery;
pub struct DiscoverySourcesSlot;
pub struct DiscoveredKeySlot;
worth_query_portable_type!(DiscoverySourcesSlot => "worth.query.certification.discovery-sources.v1");
worth_query_portable_type!(DiscoveredKeySlot => "worth.query.certification.discovered-key.v1");

impl<Schema: TopologySchemaBinding> ApplicationQueryMarkerIdentity<Schema>
    for PlanarDiscoveryQuery
{
    type ParameterBinding = PlanarDiscoveryParametersBinding;
    type ResultBinding = PlanarDiscoveryResultBinding;
    type Scope = Body;
    const IDENTIFIER: &'static str = "PlanarDiscoveryQuery";
    const QUERY_TYPE_NAME: &'static str = "worth.query.certification.planar-discovery-query.v1";
    const SCOPE_TYPE_NAME: &'static str = "Body";
}

type DiscoveryScope<Schema> = ApplicationQueryFieldScope<
    Schema,
    Body,
    PlanarPosition,
    BodyKey,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

pub struct PlanarDiscoveryBinding<Schema>(PhantomData<fn() -> Schema>);
impl<Schema: TopologySchemaBinding> ApplicationQueryBinding<Schema>
    for PlanarDiscoveryBinding<Schema>
{
    type Input = PlanarDiscoveryRead;
    type InputBinding = PlanarDiscoveryInputBinding;
    type Query = PlanarDiscoveryQuery;
    type ParameterBinding = PlanarDiscoveryParametersBinding;
    type ResultBinding = PlanarDiscoveryResultBinding;
    type ScopeBinding = DiscoveryScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.certification.planar-discovery-binding.v1";
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

impl<Schema: TopologySchemaBinding> ApplicationQueryIntent<Schema> for PlanarDiscoveryRead {
    type Binding = PlanarDiscoveryBinding<Schema>;
    fn parameters(&self) -> ApplicationQueryParameterSet<PlanarDiscoveryQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> DiscoveryScope<Schema> {
        DiscoveryScope::new(BodyKey::reference(), self.body_key)
    }
}

fn successor<
    Schema: TopologySchemaBinding,
    Slot: worth_query_decl::facade::portable_identity::WorthQueryPortableType,
>() -> ApplicationQueryResultRelationRef<
    PlanarDiscoveryQuery,
    Slot,
    Schema,
    PlanarDiscoverySource,
    Body,
    Body,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("sources", PlanarDiscoverySource::reference())
}

fn body_key<
    Schema: TopologySchemaBinding,
    Slot: worth_query_decl::facade::portable_identity::WorthQueryPortableType,
>() -> ApplicationQueryResultFieldRef<
    PlanarDiscoveryQuery,
    Slot,
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

pub fn planar_discovery_definition<Schema: TopologySchemaBinding>() -> ApplicationQueryDefinition<
    Schema,
    PlanarDiscoveryQuery,
    PlanarDiscoveryParameters,
    PlanarDiscoveryResult,
    Body,
> {
    let source = ApplicationQueryResultShapeBuilder::<
        Schema,
        PlanarDiscoveryQuery,
        Body,
        PlanarDiscoveryResult,
        PlanarDiscoveryResultBinding,
    >::new(Body::reference())
    .field(body_key::<Schema, DiscoveredKeySlot>());
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        PlanarDiscoveryQuery,
        Body,
        PlanarDiscoveryResult,
        PlanarDiscoveryResultBinding,
    >::new(Body::reference())
    .relation(successor::<Schema, DiscoverySourcesSlot>(), source)
    .build();
    ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::from_declaration())
        .root(Body::reference())
        .scope(Body::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(2, 2, 2))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .expect("planar neighbor discovery is canonical")
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationProjection<Schema, PlanarDiscoveryQuery>
    for PlanarDiscoveryResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, PlanarDiscoveryQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let source_body_keys = row
            .many(successor::<Schema, DiscoverySourcesSlot>())?
            .iter()
            .map(|source| source.field(body_key::<Schema, DiscoveredKeySlot>()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { source_body_keys })
    }
}
