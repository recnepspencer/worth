//! Four parameterized task queries: one shared row, structured selection,
//! and one residual where filter each.
use super::schema::*;
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

#[path = "queries/intents.rs"]
mod intents;
pub use intents::*;

pub struct Parameters;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskRow {
    pub key: u64,
    pub title: String,
    pub rank: i64,
}
worth_query_structured_value_binding!(pub ParametersBinding for Parameters {identity:"where-filter.parameters.v1"});
worth_query_structured_value_binding!(pub RowBinding for TaskRow {identity:"where-filter.row.v1"});
pub struct KeySlot;
pub struct TitleSlot;
pub struct RankSlot;
worth_query_portable_type!(KeySlot => "where-filter.key-slot.v1");
worth_query_portable_type!(TitleSlot => "where-filter.title-slot.v1");
worth_query_portable_type!(RankSlot => "where-filter.rank-slot.v1");

macro_rules! task_query {
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
task_query!(SearchQuery, "where-filter.search.v1", "task_search");
task_query!(DueQuery, "where-filter.due.v1", "task_due");
task_query!(FitQuery, "where-filter.fit.v1", "part_fit");
task_query!(StatusQuery, "where-filter.status.v1", "task_status");
task_query!(OpenInboxQuery, "where-filter.open-inbox.v1", "open_inbox");
task_query!(
    LiteralFilterQuery,
    "where-filter.literal-filter.v1",
    "literal_filter"
);
task_query!(RefusedQuery, "where-filter.refused.v1", "refused_filter");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectRow {
    pub key: u64,
}
worth_query_structured_value_binding!(pub ProjectRowBinding for ProjectRow {identity:"where-filter.project-row.v1"});
pub struct ProjectKeySlot;
worth_query_portable_type!(ProjectKeySlot => "where-filter.project-key-slot.v1");
worth_query_application_query!(pub ExactProjectQuery for Schema, identity "where-filter.exact-project.v1",
    parameters ParametersBinding, result ProjectRowBinding, scope Project => "Project", name "exact_project");
impl WorthQueryApplicationProjection<Schema, ExactProjectQuery> for ProjectRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, ExactProjectQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(project_key_slot())?,
        })
    }
}
pub fn project_key_slot(
) -> ApplicationQueryResultFieldRef<ExactProjectQuery, ProjectKeySlot, Schema, ProjectKey> {
    ApplicationQueryResultFieldRef::new("key", ProjectKey::at("ProjectKey"))
}

worth_query_application_query!(pub ExactOpenTaskQuery for Schema, identity "where-filter.exact-open-task.v1",
    parameters ParametersBinding, result RowBinding, scope Task => "Task", name "exact_open_task");
impl WorthQueryApplicationProjection<Schema, ExactOpenTaskQuery> for TaskRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, ExactOpenTaskQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key())?,
            title: row.field(title())?,
            rank: row.field(rank())?,
        })
    }
}

type Declaration = ApplicationSchemaDeclarationBuilder<Schema>;
thread_local! {
    /// A court-local extra declaration, so refusal courts reach the real
    /// schema declaration boundary with one deliberately invalid query.
    static EXTRA: std::cell::Cell<Option<fn(Declaration) -> Declaration>> =
        const { std::cell::Cell::new(None) };
}
pub fn declaration_with(
    extra: fn(Declaration) -> Declaration,
) -> Result<ApplicationSchemaDeclaration<Schema>, ApplicationSchemaDeclarationDenial> {
    EXTRA.with(|slot| slot.set(Some(extra)));
    let declaration = Schema::declaration();
    EXTRA.with(|slot| slot.set(None));
    declaration
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
pub fn parameter<Q, P, B>(name: &'static str) -> ApplicationQueryParameterRef<Q, P, B> {
    ApplicationQueryParameterRef::from_query_identifier(name)
}

/// The shared unfiltered list: every task of the scope project, ordered.
pub fn tasks<Q: ApplicationQueryMarkerIdentity<Schema, ResultBinding = RowBinding>>(
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
}

pub fn search(
) -> ApplicationQueryDefinitionBuilder<Schema, SearchQuery, Parameters, TaskRow, Project> {
    let q = parameter::<SearchQuery, SearchText, StringApplicationValueBinding>("q");
    let filter = ApplicationQueryWhereFilterOperands::new()
        .field("title", Title::at("Title"))
        .and_then(|operands| operands.field("notes", Notes::at("Notes")))
        .and_then(|operands| operands.parameter("q", q))
        .and_then(|operands| {
            operands.parse(
                "contains_ignore_ascii_case(title, q) \
                 || (is_some(notes) && contains_ignore_ascii_case(unwrap(notes), q))",
            )
        })
        .expect("the text search admits statically");
    tasks(SearchQuery::reference())
        .parameter(q)
        .where_filter(filter)
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

pub fn due() -> ApplicationQueryDefinitionBuilder<Schema, DueQuery, Parameters, TaskRow, Project> {
    let max_due = max_due();
    let text = due_text();
    let filter = ApplicationQueryWhereFilterOperands::new()
        .field("due", Due::at("Due"))
        .and_then(|operands| operands.field("title", Title::at("Title")))
        .and_then(|operands| operands.field("notes", Notes::at("Notes")))
        .and_then(|operands| operands.parameter("max_due", max_due))
        .and_then(|operands| operands.parameter("text", text))
        .and_then(|operands| {
            operands.parse(
                "(!is_some(max_due) || (is_some(due) && unwrap(due) <= unwrap(max_due))) && \
                 (!is_some(text) || contains_ignore_ascii_case(title, unwrap(text)) || \
                 (is_some(notes) && contains_ignore_ascii_case(unwrap(notes), unwrap(text))))",
            )
        })
        .expect("the optional date and text filters admit statically");
    tasks(DueQuery::reference())
        .parameter(max_due)
        .parameter(text)
        .where_filter(filter)
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

pub fn open_inbox(
) -> ApplicationQueryDefinitionBuilder<Schema, OpenInboxQuery, Parameters, TaskRow, Project> {
    let max_due = inbox_max_due();
    let text = inbox_text();
    let open =
        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new("open".to_owned())
            .expect("open status encodes");
    let filter = ApplicationQueryWhereFilterOperands::new()
        .field("due", Due::at("Due"))
        .and_then(|operands| operands.field("title", Title::at("Title")))
        .and_then(|operands| operands.field("notes", Notes::at("Notes")))
        .and_then(|operands| operands.parameter("max_due", max_due))
        .and_then(|operands| operands.parameter("text", text))
        .and_then(|operands| {
            operands.parse(
                "(!is_some(max_due) || (is_some(due) && unwrap(due) <= unwrap(max_due))) && \
                 (!is_some(text) || contains_ignore_ascii_case(title, unwrap(text)) || \
                 (is_some(notes) && contains_ignore_ascii_case(unwrap(notes), unwrap(text))))",
            )
        })
        .expect("open inbox optional filters admit statically");
    tasks(OpenInboxQuery::reference())
        .parameter(max_due)
        .parameter(text)
        .where_equal_value(Status::at("Status"), open)
        .where_filter(filter)
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

pub fn literal_filter(
) -> ApplicationQueryDefinitionBuilder<Schema, LiteralFilterQuery, Parameters, TaskRow, Project> {
    let open =
        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new("open".to_owned())
            .expect("open encodes");
    let done =
        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new("done".to_owned())
            .expect("done encodes");
    let cutoff =
        ApplicationEncodedScalarValue::<CanonicalDateApplicationValueBinding>::try_new(date(150))
            .expect("cutoff date encodes");
    let filter = ApplicationQueryWhereFilterOperands::new()
        .field("status", Status::at("Status"))
        .and_then(|operands| operands.field("due", Due::at("Due")))
        .and_then(|operands| operands.literal("open", open))
        .and_then(|operands| operands.literal("done", done))
        .and_then(|operands| operands.literal("cutoff", cutoff))
        .and_then(|operands| {
            operands.parse(
                "status == open && status != done && (!is_some(due) || unwrap(due) <= cutoff)",
            )
        })
        .expect("literal where filter admits statically");
    tasks(LiteralFilterQuery::reference())
        .where_filter(filter)
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

pub fn exact_project(
) -> ApplicationQueryDefinitionBuilder<Schema, ExactProjectQuery, Parameters, ProjectRow, Project> {
    let project_one =
        ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(1).expect("1 encodes");
    ApplicationQueryDefinitionBuilder::declare(ExactProjectQuery::reference())
        .root(Project::reference())
        .scope(Project::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                Schema,
                ExactProjectQuery,
                Project,
                ProjectRow,
                ProjectRowBinding,
            >::new(Project::reference())
            .field(project_key_slot())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::OptionalOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_historical())
        .public()
        .where_equal_value(ProjectKey::at("ProjectKey"), project_one)
}

pub fn exact_open_task(
) -> ApplicationQueryDefinitionBuilder<Schema, ExactOpenTaskQuery, Parameters, TaskRow, Task> {
    let open =
        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new("open".to_owned())
            .expect("open encodes");
    let expected_key =
        parameter::<ExactOpenTaskQuery, ExpectedTaskKey, U64ApplicationValueBinding>("expected_key");
    ApplicationQueryDefinitionBuilder::declare(ExactOpenTaskQuery::reference())
        .root(Task::reference())
        .scope(Task::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                Schema,
                ExactOpenTaskQuery,
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
        .parameter(expected_key)
        .where_equal_value(Status::at("Status"), open)
        .where_equal(TaskKey::at("TaskKey"), expected_key)
}

pub fn fit() -> ApplicationQueryDefinitionBuilder<Schema, FitQuery, Parameters, TaskRow, Project> {
    let min_rank = parameter::<FitQuery, MinRank, I64ApplicationValueBinding>("min_rank");
    let max_length =
        parameter::<FitQuery, MaxLength, CanonicalF64ApplicationValueBinding>("max_length");
    let filter = ApplicationQueryWhereFilterOperands::new()
        .field("rank", Rank::at("Rank"))
        .and_then(|operands| operands.field("length", Length::at("Length")))
        .and_then(|operands| operands.parameter("min_rank", min_rank))
        .and_then(|operands| operands.parameter("max_length", max_length))
        .and_then(|operands| operands.parse("rank >= min_rank && length <= max_length"))
        .expect("the numeric fit admits statically");
    tasks(FitQuery::reference())
        .parameter(min_rank)
        .parameter(max_length)
        .where_filter(filter)
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

pub fn status(
) -> ApplicationQueryDefinitionBuilder<Schema, StatusQuery, Parameters, TaskRow, Project> {
    let status = parameter::<StatusQuery, StatusValue, StringApplicationValueBinding>("status");
    let min_rank = parameter::<StatusQuery, MinRank, I64ApplicationValueBinding>("min_rank");
    let filter = ApplicationQueryWhereFilterOperands::new()
        .field("rank", Rank::at("Rank"))
        .and_then(|operands| operands.parameter("min_rank", min_rank))
        .and_then(|operands| operands.parse("rank >= min_rank"))
        .expect("the rank floor admits statically");
    tasks(StatusQuery::reference())
        .parameter(status)
        .parameter(min_rank)
        .where_equal(Status::at("Status"), status)
        .where_filter(filter)
        .order_by(rank(), ApplicationQueryOrderingDirection::Descending)
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

pub struct Registered;
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for Registered {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        let builder = match EXTRA.with(std::cell::Cell::get) {
            Some(extra) => extra(builder),
            None => builder,
        };
        Ok(builder
            .application_query(search())
            .application_query_binding::<SearchBinding>()
            .application_query(due())
            .application_query_binding::<DueBinding>()
            .application_query(fit())
            .application_query_binding::<FitBinding>()
            .application_query(status())
            .application_query_binding::<StatusBinding>()
            .application_query(open_inbox())
            .application_query_binding::<OpenInboxBinding>()
            .application_query(literal_filter())
            .application_query_binding::<LiteralFilterBinding>()
            .application_query(exact_project())
            .application_query_binding::<ExactProjectBinding>()
            .application_query(exact_open_task())
            .application_query_binding::<ExactOpenTaskBinding>())
    }
}
