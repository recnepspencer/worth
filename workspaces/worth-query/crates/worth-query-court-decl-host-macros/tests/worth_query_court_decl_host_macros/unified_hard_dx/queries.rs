//! Unified `#[derive(WorthQuery)]`, `#[derive(WorthManagedIndex)]`, and history
//! declarations for the Daymark acceptance court.
use worth_query_decl::facade::application_history::ApplicationHistoryBinding;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_identity_namespace, worth_query_portable_type, WorthManagedIndex, WorthQuery,
};

use super::schema::*;

pub const READY_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.ready-tasks.v1");
pub const LIVE_READY_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.live-ready-tasks.v1");
pub const BLOCKED_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.blocked-tasks.v1");
pub const SOURCE_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.task-source.v1");
pub const FIND_TITLE_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.find-by-title.v1");
pub const DETAILS_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.task-details.v1");
pub const PROJECT_TASKS_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.project-tasks.v1");

/// Target DX #5: unified `#[derive(WorthQuery)]` with literal equality, optional
/// parameter equality, optional date <= comparison, related-record absence,
/// root path, ordering, and direct `result = TaskRow`!
#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = READY_NS,
    result = TaskRow,
    root = Task,
    scope = ProjectKey,
    fields = TaskFacts,
    root_path(Contains),
    where_equal(state = "OPEN"),
    where_optional(priority = priority),
    where_lte_optional(due_date = due_before),
    where_no_related(BlockedBy, state = "OPEN"),
    order_by(TaskRow::task_key(), Ascending),
    limits = 64,
)]
pub struct ReadyTasks {
    #[worth_query(scope)]
    pub project: u64,
    pub priority: Option<u64>,
    pub due_before: Option<CanonicalDate>,
}

/// Target DX #6: the exact same rich query with `live = whole_result`!
#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = LIVE_READY_NS,
    result = TaskRow,
    root = Task,
    scope = ProjectKey,
    fields = TaskFacts,
    root_path(Contains),
    where_equal(state = "OPEN"),
    where_optional(priority = priority),
    where_lte_optional(due_date = due_before),
    where_no_related(BlockedBy, state = "OPEN"),
    order_by(TaskRow::task_key(), Ascending),
    live = whole_result,
    limits = 64,
)]
pub struct LiveReadyTasks {
    #[worth_query(scope)]
    pub project: u64,
    pub priority: Option<u64>,
    pub due_before: Option<CanonicalDate>,
}

/// Companion `where_related_exists` query finding currently blocked open tasks.
#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = BLOCKED_NS,
    result = TaskRow,
    root = Task,
    scope = ProjectKey,
    fields = TaskFacts,
    root_path(Contains),
    where_equal(state = "OPEN"),
    where_related_exists(BlockedBy, state = "OPEN"),
    order_by(TaskRow::task_key(), Ascending),
    limits = 64,
)]
pub struct BlockedTasks {
    #[worth_query(scope)]
    pub project: u64,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = SOURCE_NS,
    result = TaskRow,
    root = Task,
    scope = ProjectKey,
    root_path(Contains),
    order_by(TaskRow::task_key(), Ascending),
    limits = 64,
)]
pub struct TaskSource {
    #[worth_query(scope)]
    pub project: u64,
}

#[derive(WorthManagedIndex)]
#[worth_query(
    schema = DaymarkSchema,
    entity = Task,
    source = TaskSourceBinding,
    probe = String
)]
pub struct TaskTitleIndex;

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = FIND_TITLE_NS,
    scope = ProjectKey,
    managed_index = TaskTitleIndex,
    lanes = OneShotAndLive,
    order_by(TaskRow::task_key(), Ascending),
    limits = 64,
)]
pub struct FindTasksByTitle {
    #[worth_query(scope)]
    pub project: u64,
    #[worth_query(probe)]
    pub prefix: String,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = DETAILS_NS,
    result = TaskRow,
    root = Task,
    scope = TaskKey,
    cardinality = ExactlyOne,
    limits = 1,
)]
pub struct ReadTaskDetails {
    #[worth_query(scope)]
    pub task: u64,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = PROJECT_TASKS_NS,
    result = TaskRow,
    root = Task,
    scope = ProjectKey,
    root_path(Contains),
    order_by(TaskRow::task_key(), Ascending),
    limits = 64,
)]
pub struct ReadProjectTasks {
    #[worth_query(scope)]
    pub project: u64,
}

pub struct DaymarkHistory;
worth_query_portable_type!(DaymarkHistory => "daymark.unified.history.v1");
impl ApplicationHistoryBinding<DaymarkSchema> for DaymarkHistory {
    type Access = ReadProjectTasks;
    const IDENTITY: &'static str = "DaymarkHistory";
}
