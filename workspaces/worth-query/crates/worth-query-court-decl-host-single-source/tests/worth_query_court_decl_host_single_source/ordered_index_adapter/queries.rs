//! Queries, result bindings, and binding context for ordered-index courts.
use super::schema::*;
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::{
    application_schema::*, worth_query_application_query, worth_query_portable_type,
    worth_query_query_binding, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::*;

pub struct Params;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskRow {
    pub key: u64,
    pub title: String,
    pub start_min: i64,
    pub end_min: i64,
}
#[derive(Clone, Debug)]
pub struct ReadTasks {
    pub workspace: String,
}
#[derive(Clone, Debug)]
pub struct FindTitlePrefix {
    pub workspace: String,
    pub prefix: String,
}
#[derive(Clone, Debug)]
pub struct FindFocusOverlap {
    pub workspace: String,
    pub window: FocusWindow,
}

worth_query_structured_value_binding!(pub ParamsBinding for Params { identity: "ordered-index-adapter.params.v1" });
worth_query_structured_value_binding!(pub TaskRowBinding for TaskRow { identity: "ordered-index-adapter.row.v1" });
worth_query_structured_value_binding!(pub ReadTasksInput for ReadTasks { identity: "ordered-index-adapter.source-input.v1" });
worth_query_structured_value_binding!(pub FindTitlePrefixInput for FindTitlePrefix { identity: "ordered-index-adapter.find-prefix-input.v1" });
worth_query_structured_value_binding!(pub FindFocusOverlapInput for FindFocusOverlap { identity: "ordered-index-adapter.find-focus-input.v1" });

pub struct KeySlot;
pub struct TitleSlot;
pub struct StartSlot;
pub struct EndSlot;
pub struct PrefixProbeSlot;
pub struct FocusProbeSlot;
worth_query_portable_type!(KeySlot => "ordered-index-adapter.key-slot.v1");
worth_query_portable_type!(TitleSlot => "ordered-index-adapter.title-slot.v1");
worth_query_portable_type!(StartSlot => "ordered-index-adapter.start-slot.v1");
worth_query_portable_type!(EndSlot => "ordered-index-adapter.end-slot.v1");

worth_query_application_query!(pub SourceQuery for DaymarkSchema,
    identity "ordered-index-adapter.source-query.v1", parameters ParamsBinding, result TaskRowBinding,
    scope Workspace => "Workspace", name "daymark_tasks_source");
worth_query_application_query!(pub FindTitlePrefixQuery for DaymarkSchema,
    identity "ordered-index-adapter.find-title-prefix.v1", parameters ParamsBinding, result TaskRowBinding,
    scope Workspace => "Workspace", name "daymark_find_title_prefix");
worth_query_application_query!(pub FindFocusOverlapQuery for DaymarkSchema,
    identity "ordered-index-adapter.find-focus-overlap.v1", parameters ParamsBinding, result TaskRowBinding,
    scope Workspace => "Workspace", name "daymark_find_focus_overlap");

pub fn task_key<Q>() -> ApplicationQueryResultFieldRef<Q, KeySlot, DaymarkSchema, TaskKeyField> {
    ApplicationQueryResultFieldRef::new("key", TaskKeyField::at("TaskKey"))
}
pub fn task_title<Q>() -> ApplicationQueryResultFieldRef<Q, TitleSlot, DaymarkSchema, TaskTitleField>
{
    ApplicationQueryResultFieldRef::new("title", TaskTitleField::at("TaskTitle"))
}
pub fn start_min<Q>() -> ApplicationQueryResultFieldRef<Q, StartSlot, DaymarkSchema, StartMinField>
{
    ApplicationQueryResultFieldRef::new("start_min", StartMinField::at("StartMin"))
}
pub fn end_min<Q>() -> ApplicationQueryResultFieldRef<Q, EndSlot, DaymarkSchema, EndMinField> {
    ApplicationQueryResultFieldRef::new("end_min", EndMinField::at("EndMin"))
}
fn prefix_probe() -> ApplicationQueryParameterRef<
    FindTitlePrefixQuery,
    PrefixProbeSlot,
    StringApplicationValueBinding,
> {
    ApplicationQueryParameterRef::from_query_identifier("prefix")
}
fn focus_probe(
) -> ApplicationQueryParameterRef<FindFocusOverlapQuery, FocusProbeSlot, FocusWindowProbeBinding> {
    ApplicationQueryParameterRef::from_query_identifier("window")
}

fn base_query<Q: ApplicationQueryMarkerIdentity<DaymarkSchema, ResultBinding = TaskRowBinding>>(
    reference: ApplicationQueryReference<DaymarkSchema, Q, Params, TaskRow, Workspace>,
    lanes: ApplicationQueryLaneEligibility,
) -> ApplicationQueryDefinitionBuilder<DaymarkSchema, Q, Params, TaskRow, Workspace> {
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(TaskItem::reference())
        .scope(Workspace::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<DaymarkSchema, Q, TaskItem, TaskRow, TaskRowBinding>::new(
                TaskItem::reference(),
            )
            .field(task_key())
            .field(task_title())
            .field(start_min())
            .field(end_min())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(lanes)
        .public()
}

pub fn source_query(
) -> ApplicationQueryDefinitionBuilder<DaymarkSchema, SourceQuery, Params, TaskRow, Workspace> {
    base_query(
        SourceQuery::reference(),
        ApplicationQueryLaneEligibility::one_shot(),
    )
    .root_path(
        ApplicationQueryRootPath::from(Workspace::reference()).forward(ContainsTask::reference()),
    )
    .order_by(task_key(), ApplicationQueryOrderingDirection::Ascending)
}
pub fn find_title_prefix_query() -> ApplicationQueryDefinitionBuilder<
    DaymarkSchema,
    FindTitlePrefixQuery,
    Params,
    TaskRow,
    Workspace,
> {
    base_query(
        FindTitlePrefixQuery::reference(),
        ApplicationQueryLaneEligibility::one_shot().with_live(),
    )
    .managed_index(TitlePrefixIndex::reference(), prefix_probe())
    .order_by(task_key(), ApplicationQueryOrderingDirection::Ascending)
}
pub fn find_focus_overlap_query() -> ApplicationQueryDefinitionBuilder<
    DaymarkSchema,
    FindFocusOverlapQuery,
    Params,
    TaskRow,
    Workspace,
> {
    base_query(
        FindFocusOverlapQuery::reference(),
        ApplicationQueryLaneEligibility::one_shot().with_live(),
    )
    .managed_index(FocusIntervalIndex::reference(), focus_probe())
    .order_by(task_key(), ApplicationQueryOrderingDirection::Ascending)
}

impl<Q: ApplicationQueryMarkerIdentity<DaymarkSchema>>
    WorthQueryApplicationProjection<DaymarkSchema, Q> for TaskRow
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, DaymarkSchema, Q>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(task_key())?,
            title: row.field(task_title())?,
            start_min: row.field(start_min())?,
            end_min: row.field(end_min())?,
        })
    }
}

fn source_ws(input: ReadTasks) -> String {
    input.workspace
}
fn prefix_ws(input: FindTitlePrefix) -> String {
    input.workspace
}
fn focus_ws(input: FindFocusOverlap) -> String {
    input.workspace
}

worth_query_query_binding!(pub SourceBinding for ReadTasks, schema DaymarkSchema,
    identity "ordered-index-adapter.source-binding.v1", input ReadTasksInput,
    query SourceQuery, parameters ParamsBinding => |_| ApplicationQueryParameterSet::new(),
    result TaskRowBinding, principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope WorkspaceKeyField, field WorkspaceKeyField::at("WorkspaceKey"), value source_ws, limits results 64);
worth_query_query_binding!(pub FindTitlePrefixBinding for FindTitlePrefix, schema DaymarkSchema,
    identity "ordered-index-adapter.find-prefix-binding.v1", input FindTitlePrefixInput,
    query FindTitlePrefixQuery, parameters ParamsBinding => |i: &FindTitlePrefix| ApplicationQueryParameterSet::new().bind(prefix_probe(), i.prefix.clone()).unwrap(),
    result TaskRowBinding, principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope WorkspaceKeyField, field WorkspaceKeyField::at("WorkspaceKey"), value prefix_ws, limits results 64);
worth_query_query_binding!(pub FindFocusOverlapBinding for FindFocusOverlap, schema DaymarkSchema,
    identity "ordered-index-adapter.find-focus-binding.v1", input FindFocusOverlapInput,
    query FindFocusOverlapQuery, parameters ParamsBinding => |i: &FindFocusOverlap| ApplicationQueryParameterSet::new().bind(focus_probe(), i.window).unwrap(),
    result TaskRowBinding, principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope WorkspaceKeyField, field WorkspaceKeyField::at("WorkspaceKey"), value focus_ws, limits results 64);

worth_query_structured_value_binding!(pub BodyResultBinding for () { identity: "ordered-index-adapter.body-result.v1" });
worth_query_structured_value_binding!(pub BodyDenialBinding for WorthQueryOperationBodyDenial { identity: "ordered-index-adapter.body-denial.v1" });

pub struct DaymarkContext;
impl ApplicationBindingContext<DaymarkSchema> for DaymarkContext {
    type PrincipalBinding = PrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type IdempotencyKey = u64;
    type DenialBinding = BodyDenialBinding;
    const IDEMPOTENCY_IDENTITY: &'static str = "ordered-index-adapter.idempotency.v1";
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DaymarkSchema,
        PrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        PrincipalBinding::reference()
    }
}
