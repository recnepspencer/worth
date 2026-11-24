//! `#[derive(WorthQuery)]` produces identical canonical query declarations,
//! descriptors and runtime rows to hand-written query macros, including
//! omitted `parameters`, fixed `scope_value`, `principal_scope`, and
//! overridden definition defaults.
use super::super::reads::{RowBinding, TaskRow};
use super::super::schema::*;
use super::board;
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_application_query,
    worth_query_identity, worth_query_identity_namespace, worth_query_query_binding,
    worth_query_structured_value_binding, WorthQuery,
};

const PARITY_QUERY: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.parity.query.v1");
const FIXED_SCOPE_QUERY: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.parity.fixed-scope.v1");
const PRINCIPAL_QUERY: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.parity.principal-query.v1");
const LEGACY_QUERY: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.parity.legacy-query.v1");

fn fixed_project(_: &ReadInbox) -> u64 {
    1
}
fn fixed_project_owned(_: ReadInbox) -> u64 {
    1
}

#[derive(Clone, Debug, WorthValue, WorthQuery)]
#[worth_query(
    schema = Schema,
    context = BoardContext,
    namespace = PARITY_QUERY,
    query = DerivedSearchQuery,
    input = DerivedSearchInput,
    binding = DerivedSearchBinding,
    result = RowBinding,
    root = Task,
    limits = 32,
    name = "parity_search",
)]
pub struct DerivedSearch {
    #[worth_query(scope = ProjectKey)]
    pub project: u64,
    pub q: String,
}

#[derive(Clone, Debug, serde::Serialize, WorthQuery)]
#[worth_query(
    schema = Schema,
    context = BoardContext,
    namespace = FIXED_SCOPE_QUERY,
    query = DerivedInboxQuery,
    binding = DerivedInboxBinding,
    result = RowBinding,
    scope = ProjectKey,
    scope_value = fixed_project_owned,
    root = Task,
    cardinality = ExactlyOne,
    basis_support = CurrentOnly,
    limits = 1,
    name = "parity_inbox",
)]
pub struct ReadInbox;

#[derive(Clone, Debug, serde::Serialize, WorthQuery)]
#[worth_query(
    schema = Schema,
    context = BoardContext,
    namespace = PRINCIPAL_QUERY,
    query = DerivedPrincipalQuery,
    binding = DerivedPrincipalBinding,
    result = RowBinding,
    principal_scope = PrincipalIdentity,
    root = Task,
    cardinality = OptionalOne,
    limits = 1,
    name = "parity_principal",
)]
pub struct ReadPrincipal;

worth_query_structured_value_binding!(pub LegacyUnitParams for () {
    identity: "macros.parity.legacy-params.v1"
});

#[derive(Clone, Debug, WorthValue, WorthQuery)]
#[worth_query(
    schema = Schema,
    context = BoardContext,
    namespace = LEGACY_QUERY,
    query = LegacyDerivedQuery,
    binding = LegacyDerivedBinding,
    parameters = LegacyUnitParams,
    result = RowBinding,
    root = Task,
    limits = 8,
    name = "parity_legacy",
)]
pub struct LegacyRead {
    #[worth_query(scope = ProjectKey)]
    pub project: u64,
}

fn derived_search_definition(
) -> ApplicationQueryDefinitionBuilder<Schema, DerivedSearchQuery, (), TaskRow, Project> {
    let filter = ApplicationQueryWhereFilterOperands::new()
        .field("title", TaskFacts::title())
        .and_then(|operands| operands.parameter("q", DerivedSearchQuery::q()))
        .and_then(|operands| operands.parse("contains_ignore_ascii_case(title, q)"))
        .expect("derived filter admits");
    DerivedSearchQuery::definition()
        .root_path(
            ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()),
        )
        .order_by(TaskRow::key(), ApplicationQueryOrderingDirection::Ascending)
        .where_filter(filter)
}

mod written {
    use super::*;

    #[derive(Clone, Debug, WorthValue)]
    pub struct WrittenSearch {
        pub project: u64,
        pub q: String,
    }
    #[derive(Clone, Debug, serde::Serialize)]
    pub struct WrittenReadInbox;

    fn written_fixed_project(_: WrittenReadInbox) -> u64 {
        1
    }

    worth_query_application_query!(pub WrittenSearchQuery for Schema,
        identity worth_query_identity!(PARITY_QUERY, query),
        result RowBinding,
        scope Project => "Project",
        name "parity_search");
    worth_query_structured_value_binding!(pub WrittenSearchInput for WrittenSearch {
        identity: worth_query_identity!(PARITY_QUERY, request)
    });
    pub struct QParam;
    pub fn q_ref(
    ) -> ApplicationQueryParameterRef<WrittenSearchQuery, QParam, StringApplicationValueBinding>
    {
        ApplicationQueryParameterRef::from_query_identifier("q")
    }
    fn search_params(input: &WrittenSearch) -> ApplicationQueryParameterSet<WrittenSearchQuery> {
        ApplicationQueryParameterSet::new()
            .bind(q_ref(), input.q.clone())
            .expect("q encodes")
    }
    fn search_project(input: WrittenSearch) -> u64 {
        input.project
    }
    worth_query_query_binding!(pub WrittenSearchBinding for WrittenSearch, schema Schema,
        context BoardContext,
        identity worth_query_identity!(PARITY_QUERY, binding),
        input WrittenSearchInput,
        query WrittenSearchQuery,
        parameters => search_params,
        scope ProjectKey, field ProjectKey::at("ProjectKey"), value search_project,
        limits results 32
    );

    pub fn search_definition(
        cardinality: ApplicationQueryCardinality,
    ) -> ApplicationQueryDefinitionBuilder<Schema, WrittenSearchQuery, (), TaskRow, Project> {
        let filter = ApplicationQueryWhereFilterOperands::new()
            .field("title", Title::at("Title"))
            .and_then(|operands| operands.parameter("q", q_ref()))
            .and_then(|operands| operands.parse("contains_ignore_ascii_case(title, q)"))
            .expect("written filter admits");
        ApplicationQueryDefinitionBuilder::declare(WrittenSearchQuery::reference())
            .root(Task::reference())
            .scope(Project::reference())
            .result_shape(TaskRow::result_shape::<WrittenSearchQuery>())
            .cardinality(cardinality)
            .disclosure(ApplicationQueryDisclosureContract::public())
            .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
            .lanes(ApplicationQueryLaneEligibility::one_shot())
            .public()
            .parameter(q_ref())
            .root_path(
                ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()),
            )
            .order_by(TaskRow::key(), ApplicationQueryOrderingDirection::Ascending)
            .where_filter(filter)
    }

    worth_query_application_query!(pub WrittenInboxQuery for Schema,
        identity worth_query_identity!(FIXED_SCOPE_QUERY, query),
        result RowBinding,
        scope Project => "Project",
        name "parity_inbox");
    worth_query_structured_value_binding!(pub WrittenInboxInput for WrittenReadInbox {
        identity: worth_query_identity!(FIXED_SCOPE_QUERY, request)
    });
    worth_query_query_binding!(pub WrittenInboxBinding for WrittenReadInbox, schema Schema,
        context BoardContext,
        identity worth_query_identity!(FIXED_SCOPE_QUERY, binding),
        input WrittenInboxInput,
        query WrittenInboxQuery,
        scope ProjectKey, field ProjectKey::at("ProjectKey"), value written_fixed_project,
        limits results 1
    );

    pub fn inbox_definition(
    ) -> ApplicationQueryDefinitionBuilder<Schema, WrittenInboxQuery, (), TaskRow, Project> {
        ApplicationQueryDefinitionBuilder::declare(WrittenInboxQuery::reference())
            .root(Task::reference())
            .scope(Project::reference())
            .result_shape(TaskRow::result_shape::<WrittenInboxQuery>())
            .cardinality(ApplicationQueryCardinality::ExactlyOne)
            .disclosure(ApplicationQueryDisclosureContract::public())
            .basis_support(ApplicationQueryBasisSupport::current_only())
            .lanes(ApplicationQueryLaneEligibility::one_shot())
            .public()
            .root_path(
                ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()),
            )
    }
}

#[test]
fn derived_query_matches_written_parameterized_and_fixed_scope_declarations() {
    let derived = board()
        .bound_application_query::<DerivedSearchBinding>(derived_search_definition())
        .build()
        .expect("derived search query declares");
    let written = board()
        .application_query(written::search_definition(
            ApplicationQueryCardinality::Many,
        ))
        .application_query_binding::<written::WrittenSearchBinding>()
        .build()
        .expect("written search query declares");
    let twin = board()
        .application_query(written::search_definition(
            ApplicationQueryCardinality::ExactlyOne,
        ))
        .application_query_binding::<written::WrittenSearchBinding>()
        .build()
        .expect("twin query declares");

    assert_eq!(derived.identity(), written.identity());
    assert_eq!(derived.erased(), written.erased());
    assert_ne!(derived.erased(), twin.erased());

    let derived_desc = DerivedSearchBinding::descriptor();
    let written_desc = written::WrittenSearchBinding::descriptor();
    assert_eq!(derived_desc.identity(), written_desc.identity());
    assert_eq!(derived_desc.input_identity(), written_desc.input_identity());
    assert_eq!(derived_desc.query_name(), written_desc.query_name());
    assert_eq!(derived_desc.query_identity(), written_desc.query_identity());
    assert_eq!(
        derived_desc.parameter_identity(),
        written_desc.parameter_identity()
    );
    assert_eq!(
        derived_desc.result_identity(),
        written_desc.result_identity()
    );
    assert_eq!(derived_desc.scope_identity(), written_desc.scope_identity());
    assert_eq!(derived_desc.scope(), written_desc.scope());
    assert_eq!(derived_desc.principal(), written_desc.principal());
    assert_eq!(derived_desc.limits(), written_desc.limits());

    let derived_inbox = board()
        .bound_application_query::<DerivedInboxBinding>(DerivedInboxQuery::definition().root_path(
            ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()),
        ))
        .build()
        .expect("derived fixed-scope query declares");
    let written_inbox = board()
        .bound_application_query::<written::WrittenInboxBinding>(written::inbox_definition())
        .build()
        .expect("written fixed-scope query declares");
    assert_eq!(derived_inbox.identity(), written_inbox.identity());
    assert_eq!(derived_inbox.erased(), written_inbox.erased());

    assert_eq!(fixed_project(&ReadInbox), 1);
    assert_eq!(
        <DerivedPrincipalBinding as ApplicationQueryBinding<Schema>>::LIMITS,
        ApplicationQueryBindingLimits::results(1)
    );
    let legacy = LegacyRead { project: 7 };
    assert_eq!(legacy.project, 7);
    assert_eq!(
        <LegacyDerivedQuery as ApplicationQueryMarkerIdentity<Schema>>::ParameterBinding::IDENTITY_NAME,
        "macros.parity.legacy-params.v1"
    );
    assert_eq!(
        <DerivedSearchQuery as ApplicationQueryMarkerIdentity<Schema>>::ParameterBinding::IDENTITY_NAME,
        "worth.rust.unit"
    );
}
