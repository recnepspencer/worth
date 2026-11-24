//! Public and capability-governed whole-result live queries (`Today` and `Ready`).
use super::access::{tag, ReadBoard, ReadBoardRows};
use super::schema::*;
use worth_foundational::facade::CanonicalDate;
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

pub struct EmptyParams;
pub struct TodayParams;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskRow {
    pub key: u64,
    pub title: String,
    pub status: String,
    pub due: Option<CanonicalDate>,
}
worth_query_structured_value_binding!(pub EmptyParamsBinding for EmptyParams { identity: "live-whole-result.empty-params.v1" });
worth_query_structured_value_binding!(pub TodayParamsBinding for TodayParams { identity: "live-whole-result.today-params.v1" });
worth_query_structured_value_binding!(pub RowBinding for TaskRow { identity: "live-whole-result.row.v1" });

pub struct KeySlot;
pub struct TitleSlot;
pub struct StatusSlot;
pub struct DueSlot;
pub struct CutoffParam;
worth_query_portable_type!(KeySlot => "live-whole-result.key-slot.v1");
worth_query_portable_type!(TitleSlot => "live-whole-result.title-slot.v1");
worth_query_portable_type!(StatusSlot => "live-whole-result.status-slot.v1");
worth_query_portable_type!(DueSlot => "live-whole-result.due-slot.v1");

pub fn key<Q>() -> ApplicationQueryResultFieldRef<Q, KeySlot, Schema, TaskKey> {
    ApplicationQueryResultFieldRef::new("key", TaskKey::at("TaskKey"))
}
pub fn title<Q>() -> ApplicationQueryResultFieldRef<Q, TitleSlot, Schema, Title> {
    ApplicationQueryResultFieldRef::new("title", Title::at("Title"))
}
pub fn status<Q>() -> ApplicationQueryResultFieldRef<Q, StatusSlot, Schema, Status> {
    ApplicationQueryResultFieldRef::new("status", Status::at("Status"))
}
pub fn due<Q>() -> ApplicationQueryOptionalResultFieldRef<Q, DueSlot, Schema, Due> {
    ApplicationQueryOptionalResultFieldRef::new("due", Due::at("Due"))
}
pub fn cutoff_param<Q>(
) -> ApplicationQueryParameterRef<Q, CutoffParam, CanonicalDateApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("cutoff")
}
pub fn date(days: i64) -> CanonicalDate {
    CanonicalDate {
        days_from_unix_epoch: i32::try_from(days).expect("valid date"),
    }
}

macro_rules! define_query {
    ($marker:ident, $params:ident, $identity:literal, $name:literal) => {
        worth_query_application_query!(
            pub $marker for Schema,
            identity $identity,
            parameters $params,
            result RowBinding,
            scope Project => "Project",
            name $name
        );
        impl WorthQueryApplicationProjection<Schema, $marker> for TaskRow {
            fn project(
                row: &WorthQueryApplicationProjectionRow<'_, Schema, $marker>,
            ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
                Ok(Self {
                    key: row.field(key())?,
                    title: row.field(title())?,
                    status: row.field(status())?,
                    due: row.optional_field(due())?,
                })
            }
        }
    };
}
define_query!(
    TodayLiveQuery,
    TodayParamsBinding,
    "live-whole-result.today.v1",
    "today_live"
);
define_query!(
    ReadyLiveQuery,
    EmptyParamsBinding,
    "live-whole-result.ready.v1",
    "ready_live"
);
define_query!(
    GovTodayLiveQuery,
    TodayParamsBinding,
    "live-whole-result.gov-today.v1",
    "gov_today_live"
);
define_query!(
    GovReadyLiveQuery,
    EmptyParamsBinding,
    "live-whole-result.gov-ready.v1",
    "gov_ready_live"
);

fn open_literal() -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new("open".to_owned())
        .expect("open literal")
}

fn today_filter<Q>() -> ApplicationQueryWhereFilter<Q> {
    ApplicationQueryWhereFilterOperands::new()
        .field("due", Due::at("Due"))
        .and_then(|o| o.parameter("cutoff", cutoff_param::<Q>()))
        .and_then(|o| o.parse("is_some(due) && unwrap(due) <= cutoff"))
        .expect("today filter parses")
}

fn shape<Q: ApplicationQueryMarkerIdentity<Schema, ResultBinding = RowBinding>>(
) -> TypedApplicationQueryResultShape<Schema, Q, Task, TaskRow, RowBinding> {
    ApplicationQueryResultShapeBuilder::<Schema, Q, Task, TaskRow, RowBinding>::new(
        Task::reference(),
    )
    .field(key())
    .field(title())
    .field(status())
    .optional_field(due())
    .build()
}

fn governed_today_disclosure() -> ApplicationQueryDisclosureContract {
    let all = ApplicationQueryInfluenceContract::permit_all();
    ApplicationQueryDisclosureContract::governed_by("gov-today", ReadBoard::reference())
        .use_field_by(TaskKey::at("TaskKey"), tag(1), all.clone())
        .unwrap()
        .use_field_by(Status::at("Status"), tag(1), all.clone())
        .unwrap()
        .use_field_by(Due::at("Due"), tag(1), all.clone())
        .unwrap()
        .disclose_field_by(key::<GovTodayLiveQuery>(), tag(1), all.clone())
        .disclose_field_by(title::<GovTodayLiveQuery>(), tag(1), all.clone())
        .disclose_field_by(status::<GovTodayLiveQuery>(), tag(1), all.clone())
        .disclose_optional_field_by(due::<GovTodayLiveQuery>(), tag(1), all)
}

fn governed_ready_disclosure() -> ApplicationQueryDisclosureContract {
    let all = ApplicationQueryInfluenceContract::permit_all();
    ApplicationQueryDisclosureContract::governed_by("gov-ready", ReadBoard::reference())
        .use_field_by(TaskKey::at("TaskKey"), tag(1), all.clone())
        .unwrap()
        .use_field_by(Status::at("Status"), tag(1), all.clone())
        .unwrap()
        .use_field_by(Due::at("Due"), tag(1), all.clone())
        .unwrap()
        .use_relation_by(
            BlockedBy::reference(),
            RelationDirection::Forward,
            tag(1),
            all.clone(),
        )
        .disclose_field_by(key::<GovReadyLiveQuery>(), tag(1), all.clone())
        .disclose_field_by(title::<GovReadyLiveQuery>(), tag(1), all.clone())
        .disclose_field_by(status::<GovReadyLiveQuery>(), tag(1), all.clone())
        .disclose_optional_field_by(due::<GovReadyLiveQuery>(), tag(1), all)
}

fn today_definition(
) -> ApplicationQueryDefinitionBuilder<Schema, TodayLiveQuery, TodayParams, TaskRow, Project> {
    ApplicationQueryDefinitionBuilder::declare(TodayLiveQuery::reference())
        .root(Task::reference())
        .scope(Project::reference())
        .result_shape(shape::<TodayLiveQuery>())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .public()
        .live_whole_result()
        .parameter(cutoff_param())
        .root_path(
            ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()),
        )
        .where_equal_value(Status::at("Status"), open_literal())
        .where_filter(today_filter::<TodayLiveQuery>())
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

fn ready_definition(
) -> ApplicationQueryDefinitionBuilder<Schema, ReadyLiveQuery, EmptyParams, TaskRow, Project> {
    ApplicationQueryDefinitionBuilder::declare(ReadyLiveQuery::reference())
        .root(Task::reference())
        .scope(Project::reference())
        .result_shape(shape::<ReadyLiveQuery>())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .public()
        .live_whole_result()
        .root_path(
            ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()),
        )
        .where_equal_value(Status::at("Status"), open_literal())
        .where_no_related(
            ApplicationQueryRelatedPredicate::forward(BlockedBy::reference())
                .where_equal_value(Status::at("Status"), open_literal()),
        )
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

fn gov_today_definition(
) -> ApplicationQueryDefinitionBuilder<Schema, GovTodayLiveQuery, TodayParams, TaskRow, Project> {
    ApplicationQueryDefinitionBuilder::declare(GovTodayLiveQuery::reference())
        .root(Task::reference())
        .scope(Project::reference())
        .result_shape(shape::<GovTodayLiveQuery>())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(governed_today_disclosure())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .requires_ability(ReadBoardRows::reference())
        .live_whole_result()
        .parameter(cutoff_param())
        .root_path(
            ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()),
        )
        .where_equal_value(Status::at("Status"), open_literal())
        .where_filter(today_filter::<GovTodayLiveQuery>())
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

fn gov_ready_definition(
) -> ApplicationQueryDefinitionBuilder<Schema, GovReadyLiveQuery, EmptyParams, TaskRow, Project> {
    ApplicationQueryDefinitionBuilder::declare(GovReadyLiveQuery::reference())
        .root(Task::reference())
        .scope(Project::reference())
        .result_shape(shape::<GovReadyLiveQuery>())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(governed_ready_disclosure())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .requires_ability(ReadBoardRows::reference())
        .live_whole_result()
        .root_path(
            ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()),
        )
        .where_equal_value(Status::at("Status"), open_literal())
        .where_no_related(
            ApplicationQueryRelatedPredicate::forward(BlockedBy::reference())
                .where_equal_value(Status::at("Status"), open_literal()),
        )
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

#[derive(Clone, Debug, WorthValue)]
pub struct TodayIntent {
    pub project: String,
    pub cutoff_days: i64,
}
#[derive(Clone, Debug, WorthValue)]
pub struct ReadyIntent {
    pub project: String,
}
#[derive(Clone, Debug, WorthValue)]
pub struct GovTodayIntent {
    pub project: String,
    pub cutoff_days: i64,
}
#[derive(Clone, Debug, WorthValue)]
pub struct GovReadyIntent {
    pub project: String,
}
worth_query_structured_value_binding!(pub TodayInput for TodayIntent { identity: "live-whole-result.today.in.v1" });
worth_query_structured_value_binding!(pub ReadyInput for ReadyIntent { identity: "live-whole-result.ready.in.v1" });
worth_query_structured_value_binding!(pub GovTodayInput for GovTodayIntent { identity: "live-whole-result.gov-today.in.v1" });
worth_query_structured_value_binding!(pub GovReadyInput for GovReadyIntent { identity: "live-whole-result.gov-ready.in.v1" });

fn today_scope(i: TodayIntent) -> String {
    i.project
}
fn ready_scope(i: ReadyIntent) -> String {
    i.project
}
fn gov_today_scope(i: GovTodayIntent) -> String {
    i.project
}
fn gov_ready_scope(i: GovReadyIntent) -> String {
    i.project
}

worth_query_query_binding!(
    pub TodayBinding for TodayIntent,
    schema Schema,
    identity "live-whole-result.today.binding.v1",
    input TodayInput,
    query TodayLiveQuery,
    parameters TodayParamsBinding => |i: &TodayIntent| ApplicationQueryParameterSet::new().bind(cutoff_param(), date(i.cutoff_days)).expect("cutoff binds"),
    result RowBinding,
    principal PrincipalBinding,
    mapping ExternalMapping,
    principal_entity Principal,
    principal_identity u64,
    identity_binding U64ApplicationValueBinding,
    scope ProjectKey,
    field ProjectKey::at("ProjectKey"),
    value today_scope,
    limits results 64
);
worth_query_query_binding!(
    pub ReadyBinding for ReadyIntent,
    schema Schema,
    identity "live-whole-result.ready.binding.v1",
    input ReadyInput,
    query ReadyLiveQuery,
    parameters EmptyParamsBinding => |_: &ReadyIntent| ApplicationQueryParameterSet::new(),
    result RowBinding,
    principal PrincipalBinding,
    mapping ExternalMapping,
    principal_entity Principal,
    principal_identity u64,
    identity_binding U64ApplicationValueBinding,
    scope ProjectKey,
    field ProjectKey::at("ProjectKey"),
    value ready_scope,
    limits results 64
);
worth_query_query_binding!(
    pub GovTodayBinding for GovTodayIntent,
    schema Schema,
    identity "live-whole-result.gov-today.binding.v1",
    input GovTodayInput,
    query GovTodayLiveQuery,
    parameters TodayParamsBinding => |i: &GovTodayIntent| ApplicationQueryParameterSet::new().bind(cutoff_param(), date(i.cutoff_days)).expect("cutoff binds"),
    result RowBinding,
    principal PrincipalBinding,
    mapping ExternalMapping,
    principal_entity Principal,
    principal_identity u64,
    identity_binding U64ApplicationValueBinding,
    scope ProjectKey,
    field ProjectKey::at("ProjectKey"),
    value gov_today_scope,
    limits results 64
);
worth_query_query_binding!(
    pub GovReadyBinding for GovReadyIntent,
    schema Schema,
    identity "live-whole-result.gov-ready.binding.v1",
    input GovReadyInput,
    query GovReadyLiveQuery,
    parameters EmptyParamsBinding => |_: &GovReadyIntent| ApplicationQueryParameterSet::new(),
    result RowBinding,
    principal PrincipalBinding,
    mapping ExternalMapping,
    principal_entity Principal,
    principal_identity u64,
    identity_binding U64ApplicationValueBinding,
    scope ProjectKey,
    field ProjectKey::at("ProjectKey"),
    value gov_ready_scope,
    limits results 64
);

pub struct Registered;
impl Registration<Schema> for Registered {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder
            .application_query(today_definition())
            .application_query(ready_definition())
            .application_query(gov_today_definition())
            .application_query(gov_ready_definition())
            .application_query_binding::<TodayBinding>()
            .application_query_binding::<ReadyBinding>()
            .application_query_binding::<GovTodayBinding>()
            .application_query_binding::<GovReadyBinding>())
    }
}
