//! Two parameterized project reads whose rows share one derived projection:
//! a case-insensitive text search and a native due-date range.
use super::schema::*;
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_application_query,
    worth_query_identity, worth_query_identity_namespace, worth_query_query_binding,
    worth_query_structured_value_binding, WorthQueryProjection,
};

pub const ROWS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.tasks.rows.v1");
pub const SEARCH: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.tasks.search.v1");
pub const DUE: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.tasks.due.v1");

/// One task as every board read returns it.
#[derive(Clone, Debug, Eq, PartialEq, WorthQueryProjection)]
#[worth_query(schema = Schema, root = Task, namespace = ROWS)]
pub struct TaskRow {
    #[worth_query(field = TaskKey)]
    pub key: u64,
    #[worth_query(field = Title)]
    pub title: String,
    #[worth_query(field = Done)]
    pub done: bool,
    #[worth_query(field = Due)]
    pub due: Option<CanonicalDate>,
}
worth_query_structured_value_binding!(pub RowBinding for TaskRow {
    identity: worth_query_identity!(ROWS, result)
});
worth_query_structured_value_binding!(pub ParametersBinding for () {
    identity: worth_query_identity!(ROWS, parameters)
});
worth_query_application_query!(pub SearchQuery for Schema, identity worth_query_identity!(SEARCH, query),
    parameters ParametersBinding, result RowBinding, scope Project => "Project", name "task_search");
worth_query_application_query!(pub DueQuery for Schema, identity worth_query_identity!(DUE, query),
    result RowBinding, scope Project => "Project", name "task_due");

pub struct SearchText;
pub struct MaxDue;
fn text() -> ApplicationQueryParameterRef<SearchQuery, SearchText, StringApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("q")
}
fn max_due() -> ApplicationQueryParameterRef<DueQuery, MaxDue, CanonicalDateApplicationValueBinding>
{
    ApplicationQueryParameterRef::from_query_identifier("max_due")
}

/// Every task of the scope project, keyed in order.
fn tasks<Q: ApplicationQueryMarkerIdentity<Schema, ResultBinding = RowBinding>>(
    reference: ApplicationQueryReference<Schema, Q, (), TaskRow, Project>,
) -> ApplicationQueryDefinitionBuilder<Schema, Q, (), TaskRow, Project> {
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Task::reference())
        .scope(Project::reference())
        .result_shape(TaskRow::result_shape::<Q>())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()),
        )
        .order_by(TaskRow::key(), ApplicationQueryOrderingDirection::Ascending)
}

pub fn search() -> ApplicationQueryDefinitionBuilder<Schema, SearchQuery, (), TaskRow, Project> {
    let filter = ApplicationQueryWhereFilterOperands::new()
        .field("title", Title::at("Title"))
        .and_then(|operands| operands.field("notes", Notes::at("Notes")))
        .and_then(|operands| operands.parameter("q", text()))
        .and_then(|operands| {
            operands.parse(
                "contains_ignore_ascii_case(title, q) \
                 || (is_some(notes) && contains_ignore_ascii_case(unwrap(notes), q))",
            )
        })
        .expect("the text search admits statically");
    tasks(SearchQuery::reference())
        .parameter(text())
        .where_filter(filter)
}

pub fn due() -> ApplicationQueryDefinitionBuilder<Schema, DueQuery, (), TaskRow, Project> {
    let filter = ApplicationQueryWhereFilterOperands::new()
        .field("due", Due::at("Due"))
        .and_then(|operands| operands.parameter("max_due", max_due()))
        .and_then(|operands| operands.parse("is_some(due) && unwrap(due) <= max_due"))
        .expect("the date range admits statically");
    tasks(DueQuery::reference())
        .parameter(max_due())
        .where_filter(filter)
}

#[derive(Clone, Debug, WorthValue)]
pub struct Search {
    pub project: u64,
    pub q: String,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct DueBy {
    pub project: u64,
    pub max_due: CanonicalDate,
}
worth_query_structured_value_binding!(pub SearchInput for Search {
    identity: worth_query_identity!(SEARCH, request)
});
worth_query_structured_value_binding!(pub DueInput for DueBy {
    identity: worth_query_identity!(DUE, request)
});
fn search_parameters(input: &Search) -> ApplicationQueryParameterSet<SearchQuery> {
    ApplicationQueryParameterSet::new()
        .bind(text(), input.q.clone())
        .expect("search text encodes")
}
fn due_parameters(input: &DueBy) -> ApplicationQueryParameterSet<DueQuery> {
    ApplicationQueryParameterSet::new()
        .bind(max_due(), input.max_due)
        .expect("the date encodes")
}
fn search_project(input: Search) -> u64 {
    input.project
}
fn due_project(input: DueBy) -> u64 {
    input.project
}
worth_query_query_binding!(pub SearchBinding for Search, schema Schema, context BoardContext,
    identity worth_query_identity!(SEARCH, binding), input SearchInput, query SearchQuery,
    parameters ParametersBinding => search_parameters, result RowBinding,
    scope ProjectKey, field ProjectKey::at("ProjectKey"), value search_project, limits results 32
);
worth_query_query_binding!(pub DueBinding for DueBy, schema Schema, context BoardContext,
    identity worth_query_identity!(DUE, binding), input DueInput, query DueQuery,
    parameters => due_parameters,
    scope ProjectKey, field ProjectKey::at("ProjectKey"), value due_project, limits results 32
);
