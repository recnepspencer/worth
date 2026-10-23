//! Ordinary current family delivery reaches the optional generated payload.
use super::*;
use members::{GeneratedSupport, SupportKey, SupportPayload, SupportValue, TotalSupport};
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_portable_type,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

#[derive(Clone, Debug)]
pub(super) struct FamilyRead {
    pub(super) body_key: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FamilyRow {
    pub(super) members: Vec<(String, u64)>,
}
pub(super) struct Parameters;
worth_query_structured_value_binding!(pub(super) Input for FamilyRead { identity: "collection-family-read.input.v1" });
worth_query_structured_value_binding!(pub(super) ParameterBinding for Parameters { identity: "collection-family-read.parameters.v1" });
worth_query_structured_value_binding!(pub(super) ResultBinding for FamilyRow { identity: "collection-family-read.result.v1" });
pub(super) struct Query;
impl<Schema: TopologySchemaBinding> ApplicationQueryMarkerIdentity<Schema> for Query {
    type ParameterBinding = ParameterBinding;
    type ResultBinding = ResultBinding;
    type Scope = Body;
    const IDENTIFIER: &'static str = "CollectionFamilyRead";
    const QUERY_TYPE_NAME: &'static str = "collection-family-read.query.v1";
    const SCOPE_TYPE_NAME: &'static str = "Body";
}
pub(super) struct Binding<Schema>(PhantomData<fn() -> Schema>);
impl<Schema: TopologySchemaBinding> ApplicationQueryBinding<Schema> for Binding<Schema> {
    type Input = FamilyRead;
    type InputBinding = Input;
    type Query = Query;
    type ParameterBinding = ParameterBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = PlanarReadScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "collection-family-read.binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
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
impl<Schema: TopologySchemaBinding> ApplicationQueryIntent<Schema> for FamilyRead {
    type Binding = Binding<Schema>;
    fn parameters(&self) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> PlanarReadScope<Schema> {
        PlanarReadScope::new(BodyKey::reference(), self.body_key)
    }
}
pub(super) struct MembersSlot;
pub(super) struct KeySlot;
pub(super) struct ValueSlot;
worth_query_portable_type!(MembersSlot => "collection-family-read.members");
worth_query_portable_type!(KeySlot => "collection-family-read.key");
worth_query_portable_type!(ValueSlot => "collection-family-read.value");
fn members<Schema: TopologySchemaBinding>() -> ApplicationQueryResultRelationRef<
    Query,
    MembersSlot,
    Schema,
    TotalSupport,
    Body,
    GeneratedSupport,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("members", TotalSupport::reference())
}
fn key<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    Query,
    KeySlot,
    Schema,
    GeneratedSupport,
    SupportPayload,
    SupportKey,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("key", SupportKey::reference())
}
fn value<Schema: TopologySchemaBinding>() -> ApplicationQueryResultFieldRef<
    Query,
    ValueSlot,
    Schema,
    GeneratedSupport,
    SupportPayload,
    SupportValue,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("value", SupportValue::reference())
}
pub(super) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let member = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        GeneratedSupport,
        FamilyRow,
        ResultBinding,
    >::new(GeneratedSupport::reference())
    .field(key())
    .field(value());
    let shape =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Body, FamilyRow, ResultBinding>::new(
            Body::reference(),
        )
        .relation(members(), member)
        .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::<
        Schema,
        Query,
        Parameters,
        FamilyRow,
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
    .unwrap();
    schema
        .application_query(definition)
        .application_query_binding::<Binding<Schema>>()
}
impl<Schema: TopologySchemaBinding> WorthQueryApplicationProjection<Schema, Query> for FamilyRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let mut members = row
            .many(members())?
            .iter()
            .map(|member| Ok((member.field(key())?, member.field(value())?)))
            .collect::<Result<Vec<_>, _>>()?;
        members.sort();
        Ok(Self { members })
    }
}
