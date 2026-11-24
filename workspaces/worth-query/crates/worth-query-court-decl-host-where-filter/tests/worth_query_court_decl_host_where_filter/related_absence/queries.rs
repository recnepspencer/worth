//! Daymark `Ready` view and related-record absence/existence query definitions
//! covering:
//! - `ReadyTasks`: open tasks with ZERO open `BlockedBy` blockers (P2.2 / G8)
//! - `BlockedTasks`: open tasks with AT LEAST ONE open `BlockedBy` blocker
//! - `BlockingTasks`: open tasks that block AT LEAST ONE open task (Reverse)
//! - `UnlinkedTasks`: open tasks with ZERO `BlockedBy` edges (no target pred)
//! - `ReadyFiltered`: combines literal equality + optional root parameters +
//!   related-record absence with a target `where_filter` (optional min rank)
//! - `ExactReadyTask`: multi-predicate exact-root query with `where_no_related`
use super::schema::*;
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

pub struct Parameters;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskRow {
    pub key: u64,
    pub title: String,
    pub rank: i64,
}
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "related-absence.params.v1" });
worth_query_structured_value_binding!(pub RowBinding for TaskRow { identity: "related-absence.row.v1" });
pub struct KeySlot;
pub struct TitleSlot;
pub struct RankSlot;
worth_query_portable_type!(KeySlot => "related-absence.key-slot.v1");
worth_query_portable_type!(TitleSlot => "related-absence.title-slot.v1");
worth_query_portable_type!(RankSlot => "related-absence.rank-slot.v1");

macro_rules! project_task_query {
    ($marker:ident, $identity:literal, $name:literal) => {
        worth_query_application_query!(pub $marker for Schema, identity $identity,
            parameters ParametersBinding, result RowBinding, scope Project => "Project", name $name);
        impl WorthQueryApplicationProjection<Schema, $marker> for TaskRow {
            fn project(
                row: &WorthQueryApplicationProjectionRow<'_, Schema, $marker>,
            ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
                Ok(Self {
                    key: row.field(key())?,
                    title: row.field(title())?,
                    rank: row.field(rank())?,
                })
            }
        }
    };
}
project_task_query!(ReadyTasksQuery, "related-absence.ready.v1", "ready_tasks");
project_task_query!(
    BlockedTasksQuery,
    "related-absence.blocked.v1",
    "blocked_tasks"
);
project_task_query!(
    BlockingTasksQuery,
    "related-absence.blocking.v1",
    "blocking_tasks"
);
project_task_query!(
    UnlinkedTasksQuery,
    "related-absence.unlinked.v1",
    "unlinked_tasks"
);
project_task_query!(
    ReadyFilteredQuery,
    "related-absence.ready-filtered.v1",
    "ready_filtered"
);

worth_query_application_query!(pub ExactReadyTaskQuery for Schema, identity "related-absence.exact-ready.v1",
    parameters ParametersBinding, result RowBinding, scope Task => "Task", name "exact_ready_task");
impl WorthQueryApplicationProjection<Schema, ExactReadyTaskQuery> for TaskRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, ExactReadyTaskQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key())?,
            title: row.field(title())?,
            rank: row.field(rank())?,
        })
    }
}

pub fn key<Q>() -> ApplicationQueryResultFieldRef<Q, KeySlot, Schema, TaskKey> {
    ApplicationQueryResultFieldRef::new("key", TaskKey::at("TaskKey"))
}
pub fn title<Q>() -> ApplicationQueryResultFieldRef<Q, TitleSlot, Schema, Title> {
    ApplicationQueryResultFieldRef::new("title", Title::at("Title"))
}
pub fn rank<Q>() -> ApplicationQueryResultFieldRef<Q, RankSlot, Schema, Rank> {
    ApplicationQueryResultFieldRef::new("rank", Rank::at("Rank"))
}

fn open_literal() -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new("open".to_owned())
        .expect("open encodes")
}

fn open_project_tasks<Q: ApplicationQueryMarkerIdentity<Schema, ResultBinding = RowBinding>>(
    reference: ApplicationQueryReference<Schema, Q, Parameters, TaskRow, Project>,
) -> ApplicationQueryDefinitionBuilder<Schema, Q, Parameters, TaskRow, Project> {
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Task::reference())
        .scope(Project::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Schema, Q, Task, TaskRow, RowBinding>::new(
                Task::reference(),
            )
            .field(key())
            .field(title())
            .field(rank())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_historical())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()),
        )
        .where_equal_value(Status::at("Status"), open_literal())
}

pub struct MaxDueParam;
pub struct TextParam;
pub struct BlockerMinRankParam;
pub struct ExpectedRankParam;

pub fn rf_max_due() -> ApplicationQueryParameterRef<
    ReadyFilteredQuery,
    MaxDueParam,
    CanonicalDateApplicationValueBinding,
    true,
> {
    ApplicationQueryParameterRef::optional("max_due")
}
pub fn rf_text(
) -> ApplicationQueryParameterRef<ReadyFilteredQuery, TextParam, StringApplicationValueBinding, true>
{
    ApplicationQueryParameterRef::optional("text")
}
pub fn rf_blocker_min_rank() -> ApplicationQueryParameterRef<
    ReadyFilteredQuery,
    BlockerMinRankParam,
    I64ApplicationValueBinding,
    true,
> {
    ApplicationQueryParameterRef::optional("blocker_min_rank")
}
pub fn exact_expected_rank(
) -> ApplicationQueryParameterRef<ExactReadyTaskQuery, ExpectedRankParam, I64ApplicationValueBinding>
{
    ApplicationQueryParameterRef::from_query_identifier("expected_rank")
}

pub fn ready_tasks(
) -> ApplicationQueryDefinitionBuilder<Schema, ReadyTasksQuery, Parameters, TaskRow, Project> {
    open_project_tasks(ReadyTasksQuery::reference())
        .where_no_related(
            ApplicationQueryRelatedPredicate::forward(BlockedBy::reference())
                .where_equal_value(Status::at("Status"), open_literal()),
        )
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

pub fn blocked_tasks(
) -> ApplicationQueryDefinitionBuilder<Schema, BlockedTasksQuery, Parameters, TaskRow, Project> {
    open_project_tasks(BlockedTasksQuery::reference())
        .where_related_exists(
            ApplicationQueryRelatedPredicate::forward(BlockedBy::reference())
                .where_equal_value(Status::at("Status"), open_literal()),
        )
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

pub fn blocking_tasks(
) -> ApplicationQueryDefinitionBuilder<Schema, BlockingTasksQuery, Parameters, TaskRow, Project> {
    open_project_tasks(BlockingTasksQuery::reference())
        .where_related_exists(
            ApplicationQueryRelatedPredicate::reverse(BlockedBy::reference())
                .where_equal_value(Status::at("Status"), open_literal()),
        )
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

pub fn unlinked_tasks(
) -> ApplicationQueryDefinitionBuilder<Schema, UnlinkedTasksQuery, Parameters, TaskRow, Project> {
    open_project_tasks(UnlinkedTasksQuery::reference())
        .where_no_related(ApplicationQueryRelatedPredicate::forward(
            BlockedBy::reference(),
        ))
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

pub fn ready_filtered(
) -> ApplicationQueryDefinitionBuilder<Schema, ReadyFilteredQuery, Parameters, TaskRow, Project> {
    let max_due = rf_max_due();
    let text = rf_text();
    let blocker_min_rank = rf_blocker_min_rank();
    let root_filter = ApplicationQueryWhereFilterOperands::new()
        .field("due", Due::at("Due"))
        .and_then(|o| o.field("notes", Notes::at("Notes")))
        .and_then(|o| o.parameter("max_due", max_due))
        .and_then(|o| o.parameter("text", text))
        .and_then(|o| {
            o.parse(
                "(!is_some(max_due) || (is_some(due) && unwrap(due) <= unwrap(max_due))) && \
                 (!is_some(text) || (is_some(notes) && contains_ignore_ascii_case(unwrap(notes), unwrap(text))))",
            )
        })
        .expect("root filter admits");
    let target_filter = ApplicationQueryWhereFilterOperands::new()
        .field("rank", Rank::at("Rank"))
        .and_then(|o| o.parameter("blocker_min_rank", blocker_min_rank))
        .and_then(|o| o.parse("!is_some(blocker_min_rank) || rank >= unwrap(blocker_min_rank)"))
        .expect("target filter admits");
    open_project_tasks(ReadyFilteredQuery::reference())
        .parameter(max_due)
        .parameter(text)
        .parameter(blocker_min_rank)
        .where_filter(root_filter)
        .where_no_related(
            ApplicationQueryRelatedPredicate::forward(BlockedBy::reference())
                .where_equal_value(Status::at("Status"), open_literal())
                .where_filter(target_filter),
        )
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

pub fn exact_ready_task(
) -> ApplicationQueryDefinitionBuilder<Schema, ExactReadyTaskQuery, Parameters, TaskRow, Task> {
    let expected_rank = exact_expected_rank();
    ApplicationQueryDefinitionBuilder::declare(ExactReadyTaskQuery::reference())
        .root(Task::reference())
        .scope(Task::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                Schema,
                ExactReadyTaskQuery,
                Task,
                TaskRow,
                RowBinding,
            >::new(Task::reference())
            .field(key())
            .field(title())
            .field(rank())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::OptionalOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_historical())
        .public()
        .parameter(expected_rank)
        .where_equal_value(Status::at("Status"), open_literal())
        .where_equal(Rank::at("Rank"), expected_rank)
        .where_no_related(
            ApplicationQueryRelatedPredicate::forward(BlockedBy::reference())
                .where_equal_value(Status::at("Status"), open_literal()),
        )
}

mod bindings;
pub use bindings::*;
