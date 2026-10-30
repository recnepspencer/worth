//! Typed query recipes belong to each real ParentSchema mode.
use super::{
    model::*, OtherProbeBinding, PrincipalQueryResult, PrincipalQueryResultBinding, ProbeBinding,
};
use std::marker::PhantomData;
use worth_query_declaration::facade::{application_query::*, application_schema::*};
use worth_query_declaration::{
    worth_query_application_query, worth_query_portable_type, worth_query_structured_value_binding,
};

worth_query_structured_value_binding!(pub ParametersBinding for () { identity: "worth.rust.unit" });
worth_query_application_query!(pub SourceQuery for Schema: ModelSchema,
    identity "worth.query.tests.managed-parent-source.v1", parameters ParametersBinding,
    result PrincipalQueryResultBinding, scope Principal => "Principal", name "managed_parent_source");
worth_query_application_query!(pub ParentQuery for Schema: ModelSchema,
    identity "worth.query.tests.managed-parent-read.v1", parameters ParametersBinding,
    result PrincipalQueryResultBinding, scope Principal => "Principal", name "managed_parent_read");
pub struct IdentitySlot;
pub struct ProbeParameter;
worth_query_portable_type!(IdentitySlot => "worth.query.tests.managed-parent-slot.v1");

pub struct ModelIndex<S>(PhantomData<fn() -> S>);
impl<S: ModelSchema> ApplicationManagedIndexBinding<S> for ModelIndex<S> {
    type SourceBinding = SourceBinding<S>;
    type ProbeBinding = ProbeBinding;
    type Entity = Principal;
    const NAME: &'static str = "PrincipalPrefix";
    const PROVIDER: &'static str = "worth.query.tests.prefix-provider.v1";
    const PROVIDER_VERSION: u32 = 1;
}
pub struct OtherIndex<S>(PhantomData<fn() -> S>);
impl<S: ModelSchema> ApplicationManagedIndexBinding<S> for OtherIndex<S> {
    type SourceBinding = SourceBinding<S>;
    type ProbeBinding = OtherProbeBinding;
    type Entity = Principal;
    const NAME: &'static str = ModelIndex::<S>::NAME;
    const PROVIDER: &'static str = ModelIndex::<S>::PROVIDER;
    const PROVIDER_VERSION: u32 = ModelIndex::<S>::PROVIDER_VERSION;
}
fn shape<S: ModelSchema, Q: ApplicationQueryMarkerIdentity<S>>() -> TypedApplicationQueryResultShape<
    S,
    Q,
    Principal,
    PrincipalQueryResult,
    PrincipalQueryResultBinding,
> {
    ApplicationQueryResultShapeBuilder::new(Principal::reference())
        .field(ApplicationQueryResultFieldRef::<
            Q,
            IdentitySlot,
            S,
            Principal,
            PrincipalFacts,
            PrincipalIdentity,
            u64,
            ReadOnly,
            EqualityPredicate,
            NoApplicationUnit,
        >::new("identity", PrincipalIdentity::reference()))
        .build()
}
pub fn source<S: ModelSchema>(
) -> ApplicationQueryDefinition<S, SourceQuery, (), PrincipalQueryResult, Principal> {
    ApplicationQueryDefinitionBuilder::declare(SourceQuery::reference())
        .root(Principal::reference())
        .scope(Principal::reference())
        .result_shape(shape())
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
pub fn parent<S: ModelSchema>(
) -> ApplicationQueryDefinition<S, ParentQuery, (), PrincipalQueryResult, Principal> {
    let query = ApplicationQueryDefinitionBuilder::declare(ParentQuery::reference())
        .root(Principal::reference())
        .scope(Principal::reference())
        .result_shape(shape())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(if S::MODE == 3 {
            ApplicationQueryDisclosureContract::installed_policy("ManagedParentPolicy")
        } else {
            ApplicationQueryDisclosureContract::public()
        })
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot());
    let query = if S::MODE == 2 {
        query.requires_ability(ability::<S>())
    } else {
        query.public()
    };
    let query = if S::MODE == 1 {
        query.managed_index(OtherIndex::<S>::reference(), ApplicationQueryParameterRef::<ParentQuery, ProbeParameter, OtherProbeBinding>::from_query_identifier("probe"))
    } else {
        query.managed_index(ModelIndex::<S>::reference(), ApplicationQueryParameterRef::<ParentQuery, ProbeParameter, ProbeBinding>::from_query_identifier("probe"))
    };
    query.build().unwrap()
}
pub struct SourceBinding<S>(PhantomData<fn() -> S>);
pub struct ParentBinding<S>(PhantomData<fn() -> S>);
type Scope<S> = ApplicationQueryPrincipalScope<
    S,
    Principal,
    PrincipalFacts,
    PrincipalIdentity,
    u64,
    ReadOnly,
    NoApplicationUnit,
>;
macro_rules! binding {
    ($binding:ident, $query:ty, $identity:literal) => {
        impl<S: ModelSchema> ApplicationQueryBinding<S> for $binding<S> {
            type Input = ();
            type InputBinding = ParametersBinding;
            type Query = $query;
            type ParameterBinding = ParametersBinding;
            type ResultBinding = PrincipalQueryResultBinding;
            type ScopeBinding = Scope<S>;
            type PrincipalBinding = PrincipalBinding;
            type Mapping = Mapping;
            type Principal = Principal;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;
            const IDENTITY: &'static str = $identity;
            const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(8);
            fn scope_field() -> ApplicationFieldRef<
                S,
                Principal,
                PrincipalFacts,
                PrincipalIdentity,
                u64,
                ReadOnly,
                EqualityPredicate,
                NoApplicationUnit,
            > {
                PrincipalIdentity::reference()
            }
            fn principal_binding() -> ApplicationPrincipalBindingRef<
                S,
                PrincipalBinding,
                Mapping,
                Principal,
                u64,
                U64ApplicationValueBinding,
            > {
                principal_binding::<S>()
            }
        }
    };
}
binding!(
    SourceBinding,
    SourceQuery,
    "worth.query.tests.managed-source-binding.v1"
);
binding!(
    ParentBinding,
    ParentQuery,
    "worth.query.tests.managed-parent-binding.v1"
);
