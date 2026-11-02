//! One principal-scoped Source, one Parent that reads it through one managed
//! index, and the query bindings both are admitted under.
use super::schema::*;
use worth_query_declaration::facade::{application_query::*, application_schema::*};
use worth_query_declaration::{
    worth_query_application_query, worth_query_portable_type, worth_query_structured_value_binding,
};

worth_query_structured_value_binding!(pub UnitBinding for () { identity: "worth.rust.unit" });
pub struct PrincipalRow;
worth_query_structured_value_binding!(pub PrincipalRowBinding for PrincipalRow {
    identity: "worth.query.tests.decision-read-row.v1"
});
worth_query_application_query!(pub SourceQuery for DecisionReadSchema,
    identity "worth.query.tests.decision-read-source.v1", parameters UnitBinding,
    result PrincipalRowBinding, scope Principal => "Principal", name "decision_read_source");
worth_query_application_query!(pub ParentQuery for DecisionReadSchema,
    identity "worth.query.tests.decision-read-parent.v1", parameters UnitBinding,
    result PrincipalRowBinding, scope Principal => "Principal", name "decision_read_parent");
/// The probe codec must be canonical; a bare string profile is refused.
#[derive(worth_foundational::facade::WorthValue)]
pub struct Probe {
    prefix: String,
}
worth_query_declaration::worth_query_value_binding!(pub ProbeBinding for Probe {
    identity: "worth.query.tests.decision-read-probe.v1", scalar: String,
    canonical: ValueDecodeBounds::new(1024, 512, 4, 4),
});
pub struct IdentitySlot;
pub struct ProbeParameter;
worth_query_portable_type!(IdentitySlot => "worth.query.tests.decision-read-slot.v1");

pub struct DecisionIndex;
impl ApplicationManagedIndexBinding<DecisionReadSchema> for DecisionIndex {
    type SourceBinding = SourceBinding;
    type ProbeBinding = ProbeBinding;
    type Entity = Principal;
    const NAME: &'static str = "DecisionReadIndex";
    const PROVIDER: &'static str = "worth.query.tests.decision-read-provider.v1";
    const PROVIDER_VERSION: u32 = 1;
}

fn shape<Q: ApplicationQueryMarkerIdentity<DecisionReadSchema>>(
) -> TypedApplicationQueryResultShape<
    DecisionReadSchema,
    Q,
    Principal,
    PrincipalRow,
    PrincipalRowBinding,
> {
    ApplicationQueryResultShapeBuilder::new(Principal::reference())
        .field(ApplicationQueryResultFieldRef::<
            Q,
            IdentitySlot,
            DecisionReadSchema,
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

pub(super) fn source(
) -> ApplicationQueryDefinition<DecisionReadSchema, SourceQuery, (), PrincipalRow, Principal> {
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

pub(super) fn parent(
) -> ApplicationQueryDefinition<DecisionReadSchema, ParentQuery, (), PrincipalRow, Principal> {
    ApplicationQueryDefinitionBuilder::declare(ParentQuery::reference())
        .root(Principal::reference())
        .scope(Principal::reference())
        .result_shape(shape())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .managed_index(
            DecisionIndex::reference(),
            ApplicationQueryParameterRef::<
                ParentQuery,
                ProbeParameter,
                ProbeBinding,
            >::from_query_identifier("probe"),
        )
        .build()
        .unwrap()
}

pub struct SourceBinding;
pub struct ParentBinding;
macro_rules! binding {
    ($binding:ident, $query:ty, $identity:literal) => {
        impl ApplicationQueryBinding<DecisionReadSchema> for $binding {
            type Input = ();
            type InputBinding = UnitBinding;
            type Query = $query;
            type ParameterBinding = UnitBinding;
            type ResultBinding = PrincipalRowBinding;
            type ScopeBinding = ApplicationQueryPrincipalScope<
                DecisionReadSchema,
                Principal,
                PrincipalFacts,
                PrincipalIdentity,
                u64,
                ReadOnly,
                NoApplicationUnit,
            >;
            type PrincipalBinding = DecisionPrincipalBinding;
            type Mapping = Mapping;
            type Principal = Principal;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;
            const IDENTITY: &'static str = $identity;
            const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(8);
            fn scope_field() -> ApplicationFieldRef<
                DecisionReadSchema,
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
                DecisionReadSchema,
                DecisionPrincipalBinding,
                Mapping,
                Principal,
                u64,
                U64ApplicationValueBinding,
            > {
                DecisionPrincipalBinding::reference()
            }
        }
    };
}
binding!(
    SourceBinding,
    SourceQuery,
    "worth.query.tests.decision-read-source-binding.v1"
);
binding!(
    ParentBinding,
    ParentQuery,
    "worth.query.tests.decision-read-parent-binding.v1"
);
