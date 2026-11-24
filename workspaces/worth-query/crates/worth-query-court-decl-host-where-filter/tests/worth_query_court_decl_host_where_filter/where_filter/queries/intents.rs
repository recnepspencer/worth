//! Typed read intents and their parameter bindings.
use super::*;
use worth_foundational::facade::{CanonicalDate, CanonicalF64};

pub struct SearchText;
pub struct MaxDue;
pub struct MinRank;
pub struct MaxLength;
pub struct StatusValue;
pub struct ExpectedTaskKey;

#[derive(Clone, Debug, WorthValue)]
pub struct Search {
    pub project: u64,
    pub q: String,
}
#[derive(Clone, Debug, WorthValue)]
pub struct DueBy {
    pub project: u64,
    pub max_due_days: Option<i64>,
    pub text: Option<String>,
}
#[derive(Clone, Debug, WorthValue)]
pub struct OpenInbox {
    pub project: u64,
    pub max_due_days: Option<i64>,
    pub text: Option<String>,
}
#[derive(Clone, Debug, WorthValue)]
pub struct LiteralFilter {
    pub project: u64,
}
#[derive(Clone, Debug, WorthValue)]
pub struct ExactProject {
    pub project: u64,
}
#[derive(Clone, Debug, WorthValue)]
pub struct ExactOpenTask {
    pub task: u64,
    pub expected_key: u64,
}
#[derive(Clone, Debug, WorthValue)]
pub struct Fit {
    pub project: u64,
    pub min_rank: i64,
    pub max_length_micros: u64,
}
#[derive(Clone, Debug, WorthValue)]
pub struct ByStatus {
    pub project: u64,
    pub status: String,
    pub min_rank: i64,
}
worth_query_structured_value_binding!(pub SearchInput for Search {identity:"where-filter.search-input.v1"});
worth_query_structured_value_binding!(pub DueInput for DueBy {identity:"where-filter.due-input.v1"});
worth_query_structured_value_binding!(pub OpenInboxInput for OpenInbox {identity:"where-filter.open-inbox-input.v1"});
worth_query_structured_value_binding!(pub LiteralFilterInput for LiteralFilter {identity:"where-filter.literal-filter-input.v1"});
worth_query_structured_value_binding!(pub ExactProjectInput for ExactProject {identity:"where-filter.exact-project-input.v1"});
worth_query_structured_value_binding!(pub ExactOpenTaskInput for ExactOpenTask {identity:"where-filter.exact-open-task-input.v1"});
worth_query_structured_value_binding!(pub FitInput for Fit {identity:"where-filter.fit-input.v1"});
worth_query_structured_value_binding!(pub StatusInput for ByStatus {identity:"where-filter.status-input.v1"});

pub fn date(days: i64) -> CanonicalDate {
    CanonicalDate {
        days_from_unix_epoch: i32::try_from(days).expect("court dates fit i32"),
    }
}
pub fn length(micros: u64) -> CanonicalF64 {
    CanonicalF64::from_f64(micros as f64 / 1_000_000.0)
}
pub fn search_parameters(input: &Search) -> ApplicationQueryParameterSet<SearchQuery> {
    ApplicationQueryParameterSet::new()
        .bind(
            parameter::<SearchQuery, SearchText, StringApplicationValueBinding>("q"),
            input.q.clone(),
        )
        .expect("search text encodes")
}
pub fn max_due(
) -> ApplicationQueryParameterRef<DueQuery, MaxDue, CanonicalDateApplicationValueBinding, true> {
    ApplicationQueryParameterRef::optional("max_due")
}
pub fn due_text(
) -> ApplicationQueryParameterRef<DueQuery, SearchText, StringApplicationValueBinding, true> {
    ApplicationQueryParameterRef::optional("text")
}
pub fn inbox_max_due(
) -> ApplicationQueryParameterRef<OpenInboxQuery, MaxDue, CanonicalDateApplicationValueBinding, true>
{
    ApplicationQueryParameterRef::optional("max_due")
}
pub fn inbox_text(
) -> ApplicationQueryParameterRef<OpenInboxQuery, SearchText, StringApplicationValueBinding, true> {
    ApplicationQueryParameterRef::optional("text")
}
fn due_parameters(input: &DueBy) -> ApplicationQueryParameterSet<DueQuery> {
    ApplicationQueryParameterSet::new()
        .bind_optional(max_due(), input.max_due_days.map(date))
        .and_then(|set| set.bind_optional(due_text(), input.text.clone()))
        .expect("the date and text encode")
}
fn open_inbox_parameters(input: &OpenInbox) -> ApplicationQueryParameterSet<OpenInboxQuery> {
    ApplicationQueryParameterSet::new()
        .bind_optional(inbox_max_due(), input.max_due_days.map(date))
        .and_then(|set| set.bind_optional(inbox_text(), input.text.clone()))
        .expect("open inbox optional parameters encode")
}
fn literal_filter_parameters(
    _input: &LiteralFilter,
) -> ApplicationQueryParameterSet<LiteralFilterQuery> {
    ApplicationQueryParameterSet::new()
}
fn exact_project_parameters(
    _input: &ExactProject,
) -> ApplicationQueryParameterSet<ExactProjectQuery> {
    ApplicationQueryParameterSet::new()
}
fn exact_open_task_parameters(
    input: &ExactOpenTask,
) -> ApplicationQueryParameterSet<ExactOpenTaskQuery> {
    ApplicationQueryParameterSet::new()
        .bind(
            parameter::<ExactOpenTaskQuery, ExpectedTaskKey, U64ApplicationValueBinding>(
                "expected_key",
            ),
            input.expected_key,
        )
        .expect("expected task key encodes")
}
fn fit_parameters(input: &Fit) -> ApplicationQueryParameterSet<FitQuery> {
    ApplicationQueryParameterSet::new()
        .bind(
            parameter::<FitQuery, MinRank, I64ApplicationValueBinding>("min_rank"),
            input.min_rank,
        )
        .and_then(|set| {
            set.bind(
                parameter::<FitQuery, MaxLength, CanonicalF64ApplicationValueBinding>("max_length"),
                length(input.max_length_micros),
            )
        })
        .expect("fit parameters encode")
}
fn status_parameters(input: &ByStatus) -> ApplicationQueryParameterSet<StatusQuery> {
    ApplicationQueryParameterSet::new()
        .bind(
            parameter::<StatusQuery, StatusValue, StringApplicationValueBinding>("status"),
            input.status.clone(),
        )
        .and_then(|set| {
            set.bind(
                parameter::<StatusQuery, MinRank, I64ApplicationValueBinding>("min_rank"),
                input.min_rank,
            )
        })
        .expect("status parameters encode")
}
fn search_project(input: Search) -> u64 {
    input.project
}
fn due_project(input: DueBy) -> u64 {
    input.project
}
fn fit_project(input: Fit) -> u64 {
    input.project
}
fn status_project(input: ByStatus) -> u64 {
    input.project
}
fn open_inbox_project(input: OpenInbox) -> u64 {
    input.project
}
fn literal_filter_project(input: LiteralFilter) -> u64 {
    input.project
}
fn exact_project_id(input: ExactProject) -> u64 {
    input.project
}
fn exact_open_task_id(input: ExactOpenTask) -> u64 {
    input.task
}
macro_rules! binding {
    ($binding:ident, $intent:ident, $identity:literal, $input:ident, $query:ident, $parameters:ident, $project:ident) => {
        worth_query_query_binding!(pub $binding for $intent, schema Schema, identity $identity,
            input $input, query $query, parameters ParametersBinding => |input: &$intent| $parameters(input),
            result RowBinding, principal PrincipalBinding, mapping ExternalMapping,
            principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
            scope ProjectKey, field ProjectKey::at("ProjectKey"), value $project, limits results 64);
    };
}
binding!(
    SearchBinding,
    Search,
    "where-filter.search-binding.v1",
    SearchInput,
    SearchQuery,
    search_parameters,
    search_project
);
binding!(
    DueBinding,
    DueBy,
    "where-filter.due-binding.v1",
    DueInput,
    DueQuery,
    due_parameters,
    due_project
);
binding!(
    FitBinding,
    Fit,
    "where-filter.fit-binding.v1",
    FitInput,
    FitQuery,
    fit_parameters,
    fit_project
);
binding!(
    StatusBinding,
    ByStatus,
    "where-filter.status-binding.v1",
    StatusInput,
    StatusQuery,
    status_parameters,
    status_project
);
binding!(
    OpenInboxBinding,
    OpenInbox,
    "where-filter.open-inbox-binding.v1",
    OpenInboxInput,
    OpenInboxQuery,
    open_inbox_parameters,
    open_inbox_project
);
binding!(
    LiteralFilterBinding,
    LiteralFilter,
    "where-filter.literal-filter-binding.v1",
    LiteralFilterInput,
    LiteralFilterQuery,
    literal_filter_parameters,
    literal_filter_project
);
worth_query_query_binding!(pub ExactProjectBinding for ExactProject, schema Schema,
    identity "where-filter.exact-project-binding.v1", input ExactProjectInput,
    query ExactProjectQuery, parameters ParametersBinding => |input: &ExactProject| exact_project_parameters(input),
    result ProjectRowBinding, principal PrincipalBinding, mapping ExternalMapping,
    principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ProjectKey, field ProjectKey::at("ProjectKey"), value exact_project_id, limits results 64);
worth_query_query_binding!(pub ExactOpenTaskBinding for ExactOpenTask, schema Schema,
    identity "where-filter.exact-open-task-binding.v1", input ExactOpenTaskInput,
    query ExactOpenTaskQuery, parameters ParametersBinding => |input: &ExactOpenTask| exact_open_task_parameters(input),
    result RowBinding, principal PrincipalBinding, mapping ExternalMapping,
    principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope TaskKey, field TaskKey::at("TaskKey"), value exact_open_task_id, limits results 64);
