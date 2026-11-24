//! Query intent bindings and schema registration for the related-absence court.
use super::*;
use worth_foundational::facade::CanonicalDate;

#[derive(Clone, Debug, WorthValue)]
pub struct ProjectScope {
    pub project: u64,
}
#[derive(Clone, Debug, WorthValue)]
pub struct ReadyFilteredIntent {
    pub project: u64,
    pub max_due_days: Option<i64>,
    pub text: Option<String>,
    pub blocker_min_rank: Option<i64>,
}
#[derive(Clone, Debug, WorthValue)]
pub struct ExactReadyIntent {
    pub task: u64,
    pub expected_rank: i64,
}
worth_query_structured_value_binding!(pub ProjectScopeInput for ProjectScope { identity: "related-absence.project-scope.v1" });
worth_query_structured_value_binding!(pub ReadyFilteredInput for ReadyFilteredIntent { identity: "related-absence.ready-filtered-input.v1" });
worth_query_structured_value_binding!(pub ExactReadyInput for ExactReadyIntent { identity: "related-absence.exact-ready-input.v1" });

pub fn date(days: i64) -> CanonicalDate {
    CanonicalDate {
        days_from_unix_epoch: i32::try_from(days).expect("court dates fit i32"),
    }
}

fn empty_params<Q>(_input: &ProjectScope) -> ApplicationQueryParameterSet<Q> {
    ApplicationQueryParameterSet::new()
}
fn scope_project(input: ProjectScope) -> u64 {
    input.project
}
fn rf_project(input: ReadyFilteredIntent) -> u64 {
    input.project
}
fn rf_params(input: &ReadyFilteredIntent) -> ApplicationQueryParameterSet<ReadyFilteredQuery> {
    ApplicationQueryParameterSet::new()
        .bind_optional(rf_max_due(), input.max_due_days.map(date))
        .and_then(|s| s.bind_optional(rf_text(), input.text.clone()))
        .and_then(|s| s.bind_optional(rf_blocker_min_rank(), input.blocker_min_rank))
        .expect("ready filtered params encode")
}
fn exact_task_id(input: ExactReadyIntent) -> u64 {
    input.task
}
fn exact_params(input: &ExactReadyIntent) -> ApplicationQueryParameterSet<ExactReadyTaskQuery> {
    ApplicationQueryParameterSet::new()
        .bind(exact_expected_rank(), input.expected_rank)
        .expect("exact expected rank encodes")
}

#[derive(Clone, Debug, WorthValue)]
pub struct ReadyIntent(pub ProjectScope);
#[derive(Clone, Debug, WorthValue)]
pub struct BlockedIntent(pub ProjectScope);
#[derive(Clone, Debug, WorthValue)]
pub struct BlockingIntent(pub ProjectScope);
#[derive(Clone, Debug, WorthValue)]
pub struct UnlinkedIntent(pub ProjectScope);
worth_query_structured_value_binding!(pub ReadyInput for ReadyIntent { identity: "related-absence.ready-in.v1" });
worth_query_structured_value_binding!(pub BlockedInput for BlockedIntent { identity: "related-absence.blocked-in.v1" });
worth_query_structured_value_binding!(pub BlockingInput for BlockingIntent { identity: "related-absence.blocking-in.v1" });
worth_query_structured_value_binding!(pub UnlinkedInput for UnlinkedIntent { identity: "related-absence.unlinked-in.v1" });

macro_rules! project_binding {
    ($binding:ident, $intent:ident, $scope_fn:ident, $identity:literal, $input:ident, $query:ident) => {
        fn $scope_fn(i: $intent) -> u64 {
            scope_project(i.0)
        }
        worth_query_query_binding!(pub $binding for $intent, schema Schema, identity $identity,
            input $input, query $query, parameters ParametersBinding => |input: &$intent| empty_params(&input.0),
            result RowBinding, principal PrincipalBinding, mapping ExternalMapping,
            principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
            scope ProjectKey, field ProjectKey::at("ProjectKey"), value $scope_fn, limits results 64);
    };
}
project_binding!(
    ReadyBinding,
    ReadyIntent,
    ready_scope,
    "related-absence.ready-bind.v1",
    ReadyInput,
    ReadyTasksQuery
);
project_binding!(
    BlockedBinding,
    BlockedIntent,
    blocked_scope,
    "related-absence.blocked-bind.v1",
    BlockedInput,
    BlockedTasksQuery
);
project_binding!(
    BlockingBinding,
    BlockingIntent,
    blocking_scope,
    "related-absence.blocking-bind.v1",
    BlockingInput,
    BlockingTasksQuery
);
project_binding!(
    UnlinkedBinding,
    UnlinkedIntent,
    unlinked_scope,
    "related-absence.unlinked-bind.v1",
    UnlinkedInput,
    UnlinkedTasksQuery
);

worth_query_query_binding!(pub ReadyFilteredBinding for ReadyFilteredIntent, schema Schema,
    identity "related-absence.ready-filtered-bind.v1", input ReadyFilteredInput,
    query ReadyFilteredQuery, parameters ParametersBinding => |input: &ReadyFilteredIntent| rf_params(input),
    result RowBinding, principal PrincipalBinding, mapping ExternalMapping,
    principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ProjectKey, field ProjectKey::at("ProjectKey"), value rf_project, limits results 64);

worth_query_query_binding!(pub ExactReadyBinding for ExactReadyIntent, schema Schema,
    identity "related-absence.exact-ready-bind.v1", input ExactReadyInput,
    query ExactReadyTaskQuery, parameters ParametersBinding => |input: &ExactReadyIntent| exact_params(input),
    result RowBinding, principal PrincipalBinding, mapping ExternalMapping,
    principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope TaskKey, field TaskKey::at("TaskKey"), value exact_task_id, limits results 64);

pub struct Registered;
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for Registered {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder
            .application_query(ready_tasks())
            .application_query_binding::<ReadyBinding>()
            .application_query(blocked_tasks())
            .application_query_binding::<BlockedBinding>()
            .application_query(blocking_tasks())
            .application_query_binding::<BlockingBinding>()
            .application_query(unlinked_tasks())
            .application_query_binding::<UnlinkedBinding>()
            .application_query(ready_filtered())
            .application_query_binding::<ReadyFilteredBinding>()
            .application_query(exact_ready_task())
            .application_query_binding::<ExactReadyBinding>())
    }
}
