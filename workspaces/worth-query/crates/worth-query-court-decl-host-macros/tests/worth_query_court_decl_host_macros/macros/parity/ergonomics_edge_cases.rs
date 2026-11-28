//! Edge-case courts for Blockers 2, 4, 5, 6, and 7:
//! - Blocker 2: `bound_application_query` collision & duplicate query refusals
//! - Blocker 4: uppercase record fields (`X`) and disambiguated `selector_const`
//! - Blocker 5: generic-family `as ActionAlias` (fixed + generic + schema-generic)
//! - Blocker 6: domain result carrier without inherent `result_shape` via `definition_with`
//! - Blocker 7: correction callback hygiene (`original`, `preimage`, `key`) and wrong-scalar pre-image
use super::super::actions::{CompleteBinding, RenameBinding, Renamed};
use super::super::schema::*;
use super::board;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::fragment::NativeScalars;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_identity_namespace, worth_query_portable_type,
    worth_query_structured_value_binding, WorthQuery, WorthQueryRecord,
    WorthQuerySchema,
};
use worth_query_host::facade::worth_query_application_contribution;

const CARRIER_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.parity.domain-carrier.v1");

// --- Blocker 6: Domain result carrier without inherent `result_shape` ---
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomainTaskCarrier {
    pub title: String,
}
worth_query_structured_value_binding!(pub DomainTaskCarrierBinding for DomainTaskCarrier {
    identity: "macros.parity.domain-carrier.row.v1"
});
pub struct CarrierTitleSlot;
worth_query_portable_type!(CarrierTitleSlot => "macros.parity.domain-carrier.slot.title.v1");

#[derive(Clone, Debug, serde::Serialize, WorthQuery)]
#[worth_query(
    schema = Schema,
    context = BoardContext,
    namespace = CARRIER_NS,
    query = DomainCarrierQuery,
    binding = DomainCarrierBinding,
    result = DomainTaskCarrierBinding,
    root = Task,
    explicit_shape,
    limits = 4,
    name = "domain_carrier_read",
)]
pub struct ReadDomainCarrier {
    #[worth_query(scope = ProjectKey)]
    pub project: u64,
}

#[test]
fn domain_result_carrier_without_derived_projection_uses_definition_with() {
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        DomainCarrierQuery,
        Task,
        DomainTaskCarrier,
        DomainTaskCarrierBinding,
    >::new(Task::reference())
    .field(ApplicationQueryResultFieldRef::<
        DomainCarrierQuery,
        CarrierTitleSlot,
        Schema,
        Title,
    >::new("title", TaskFacts::title()))
    .build();
    let def = DomainCarrierQuery::definition_with(shape).root_path(
        ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()),
    );
    let declared = board()
        .bound_application_query::<DomainCarrierBinding>(def)
        .build()
        .expect("domain carrier query declares via definition_with");
    assert!(declared
        .erased()
        .members()
        .iter()
        .any(|m| matches!(m, ApplicationSchemaMember::ApplicationQuery { .. })));
    let req = ReadDomainCarrier { project: 1 };
    assert_eq!(req.project, 1);
}

// --- Blocker 4: Uppercase field `X` + disambiguated `selector_const` ---
#[test]
fn uppercase_and_disambiguated_record_fields_compile_without_selector_collision() {
    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(
        schema,
        owner = "macros.parity.upper",
        name = "upper",
        members(UpperFrag)
    )]
    pub struct UpperRoot;
    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(fragment, name = "upper", bindings = NativeScalars, members(Metric))]
    pub struct UpperFrag;
    #[derive(Clone, Copy, WorthQuerySchema)]
    #[worth_query(entity, aspects(MetricFacts))]
    pub struct Metric;
    #[derive(WorthQueryRecord)]
    #[worth_query(bindings = NativeScalars, aspect = MetricFacts, fragment = UpperFrag, entity = Metric)]
    #[allow(non_snake_case)]
    pub struct MetricRow {
        #[worth_query(equality, read_only)]
        pub X: u64,
        #[worth_query(column = "FooBar")]
        pub foo_bar: u64,
        #[worth_query(column = "FooBarAlt", selector_const = FOO_BAR_ALT)]
        pub fooBar: u64,
    }

    assert_eq!(MetricRow::X.path(), X::at("X").path());
    assert_eq!(MetricRow::select_X().path(), X::at("X").path());
    assert_eq!(MetricFacts::X().path(), X::at("X").path());
    assert_eq!(MetricRow::FOO_BAR.path(), FooBar::at("FooBar").path());
    assert_eq!(
        MetricRow::FOO_BAR_ALT.path(),
        FooBarAlt::at("FooBarAlt").path()
    );
    assert_eq!(
        MetricFacts::fooBar().path(),
        FooBarAlt::at("FooBarAlt").path()
    );
    let _ = UpperRoot::declaration().expect("upper-field record schema declares");
}

// --- Blocker 5: Generic-family `as ActionAlias` (fixed + generic + schema-generic) ---
#[test]
fn generic_feature_action_aliases_avoid_unused_type_parameters() {
    pub struct FamilyFeature;
    impl ApplicationFeature<Schema> for FamilyFeature {
        type Inputs = ApplicationFeatureInputLeaf;
        const IDENTITY: &'static str = "macros.parity.family.v1";
    }
    pub struct GenericTag;
    worth_query_application_contribution! {
        pub feature FamilyDefinition<Schema, T> for FamilyFeature => FamilyRuntime
        where
            T: 'static,
        {
            body_action_ref(RenameBinding, Renamed) as FixedFamilyAction {};
            body_action_ref(CompleteBinding, super::super::actions::Completed) as FixedCompleteAction {};
        }
    }
    pub struct ScopedFeature;
    impl<S: ApplicationSchema> ApplicationFeature<S> for ScopedFeature {
        type Inputs = ApplicationFeatureInputLeaf;
        const IDENTITY: &'static str = "macros.parity.scoped-family.v1";
    }
    worth_query_application_contribution! {
        pub feature ScopedDefinition<S: ApplicationSchema, B, D> for ScopedFeature => ScopedRuntime
        where
            B: worth_query_decl::facade::application_operation::ApplicationMutationBinding<
                    S,
                    Result = (),
                    Denial = worth_query_host::facade::primary_graph::WorthQueryOperationBodyDenial,
                    Decision = worth_query_host::facade::primary_graph::WorthQueryOperationBodyDecision<
                        S,
                        <<B as worth_query_decl::facade::application_operation::ApplicationMutationBinding<S>>::ScopeBinding
                            as worth_query_decl::facade::application_operation::ApplicationMutationScopeBinding<S>>::Scope,
                    >,
                >,
            D: ApplicationActionDelivery<S, B>,
            D::Mode: ApplicationActionOutputRoots<S>,
        {
            body_action_ref(B, D) as ScopedFamilyAction<S, B, D> {};
        }
    }

    let _fixed = FixedFamilyAction::selector();
    let _complete = FixedCompleteAction::selector();
    let _scoped = ScopedFamilyAction::<Schema, RenameBinding, Renamed>::selector();
    let _rt = FamilyRuntime::<GenericTag>::new();
    let _srt = ScopedRuntime::<Schema, RenameBinding, Renamed>::new();
    assert_eq!(
        FixedFamilyAction::selector().binding(),
        <RenameBinding as worth_query_decl::facade::application_operation::ApplicationMutationBinding<Schema>>::IDENTITY
    );
}

// --- Blocker 7: Callback hygiene (`original`, `preimage`, `key`) & wrong-scalar preimage ---
