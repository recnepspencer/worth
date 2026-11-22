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
task_query!(RefusedQuery, "where-filter.refused.v1", "refused_filter");

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
        .lanes(ApplicationQueryLaneEligibility::one_shot())
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
                "contains_ignore_ascii_case(title, q) || contains_ignore_ascii_case(notes, q)",
            )
        })
        .expect("the text search admits statically");
    tasks(SearchQuery::reference())
        .parameter(q)
        .where_filter(filter)
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

pub fn due() -> ApplicationQueryDefinitionBuilder<Schema, DueQuery, Parameters, TaskRow, Project> {
    let max_due = parameter::<DueQuery, MaxDue, CanonicalDateApplicationValueBinding>("max_due");
    let filter = ApplicationQueryWhereFilterOperands::new()
        .field("due", Due::at("Due"))
        .and_then(|operands| operands.parameter("max_due", max_due))
        .and_then(|operands| operands.parse("is_some(due) && unwrap(due) <= max_due"))
        .expect("the date range admits statically");
    tasks(DueQuery::reference())
        .parameter(max_due)
        .where_filter(filter)
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
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
            .application_query_binding::<StatusBinding>())
    }
}
