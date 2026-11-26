//! A Daymark-shaped task list whose title and focus block are Optional fields,
//! indexed by the stock `text_prefix` and `interval_1d` managed indexes.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::{
    application_identity::ApplicationIdentityNamespace,
    application_schema::{fragment::*, *},
    authentication::*,
    worth_query_identity, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_principal_binding, worth_query_structured_value_binding, WorthManagedIndex,
    WorthQuery, WorthQueryAction, WorthQueryRecord, WorthQueryRelation, WorthQueryRow,
    WorthQuerySchema,
};

pub use super::probe::FocusWindow;
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDenial;

pub const INBOX: &str = "inbox";
pub const SPARSE: ApplicationIdentityNamespace = worth_query_identity_namespace!("sparse.dx.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.sparse.dx",
    name = "sparse_dx",
    members(super::program::SparseContribution)
)]
pub struct SparseSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(fragment, name = "sparse_dx", bindings = Codecs, members(Owner, Mapping, Inbox, Task))]
pub struct Records;

worth_query_native_scalar_bindings!(pub Codecs);
impl Scalar<WorthQueryExternalPrincipalIdentity> for Codecs {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Codecs {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(OwnerFacts), reference(SparseSchema, Records))]
pub struct Owner;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(MappingFacts),
    relations(MappingOwner),
    reference(SparseSchema, Records)
)]
pub struct Mapping;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(InboxFacts),
    relations(InboxTasks),
    reference(SparseSchema, Records)
)]
pub struct Inbox;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(TaskFacts), reference(SparseSchema, Records))]
pub struct Task;

#[derive(WorthQueryRelation)]
#[worth_query(to = Owner, from = Mapping, reference(SparseSchema, Records))]
pub struct MappingOwner;

#[derive(WorthQueryRelation)]
#[worth_query(to = Task, from = Inbox, reference(SparseSchema, Records))]
pub struct InboxTasks;

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = OwnerFacts, fragment = Records, entity = Owner)]
pub struct OwnerRow {
    #[worth_query(equality, read_only)]
    pub owner_id: u64,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = MappingFacts, fragment = Records, entity = Mapping)]
pub struct MappingRow {
    #[worth_query(equality)]
    pub external_identity: WorthQueryExternalPrincipalIdentity,
    #[worth_query(equality)]
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = InboxFacts, fragment = Records, entity = Inbox)]
pub struct InboxRow {
    #[worth_query(equality, unique, read_only)]
    pub inbox_id: String,
}

/// Title and focus block are plain `Option<T>` fields: no sentinel values.
#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = TaskFacts, fragment = Records, entity = Task)]
pub struct TaskRow {
    #[worth_query(equality, unique, read_only)]
    pub task_id: u64,
    pub title: Option<String>,
    pub focus_start: Option<CanonicalTimestamp>,
    pub focus_end: Option<CanonicalTimestamp>,
}

worth_query_structured_value_binding!(pub UnitResult for () { identity: worth_query_identity!(SPARSE, result) });
worth_query_structured_value_binding!(pub BodyDenial for WorthQueryOperationBodyDenial {
    identity: worth_query_identity!(SPARSE, denial)
});

worth_query_principal_binding! {
    pub OwnerBinding in SparseSchema, mapping Mapping => Owner {
        identity: MappingFacts::external_identity(),
        status: MappingFacts::mapping_status(),
        target: MappingOwner,
        principal: OwnerId = OwnerFacts::owner_id(),
    }
    context pub Desktop {
        idempotency: u64 = worth_query_identity!(SPARSE, idempotency),
        denial: BodyDenial,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = SparseSchema, root = Task, namespace = SPARSE, member)]
pub struct TaskView {
    #[worth_query(field = TaskId)]
    pub id: u64,
    #[worth_query(field = Title)]
    pub title: Option<String>,
    #[worth_query(field = FocusStart)]
    pub focus_start: Option<CanonicalTimestamp>,
    #[worth_query(field = FocusEnd)]
    pub focus_end: Option<CanonicalTimestamp>,
}

/// The whole Inbox: Source of both managed indexes and itself watchable.
#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = SparseSchema, context = Desktop, namespace = SPARSE, member,
    result = TaskView, root = Task, scope = InboxId, root_path(InboxTasks),
    order_by(TaskView::id(), Ascending), live = whole_result, limits = 64,
)]
pub struct ReadInbox {
    #[worth_query(scope)]
    pub inbox: String,
}

/// Sparse title-prefix index over the Optional `title`.
#[derive(WorthManagedIndex)]
#[worth_index(
    schema = SparseSchema, record = Task, source = ReadInboxBinding, probe = String,
    kind = text_prefix(TaskView::title()), namespace = "sparse.dx.title_prefix", version = 1,
)]
pub struct TitleIndex;

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = SparseSchema, context = Desktop, namespace = SPARSE, member,
    scope = InboxId, managed_index = TitleIndex, lanes = OneShotAndLive,
    order_by(TaskView::id(), Ascending), limits = 64,
)]
pub struct FindTitle {
    #[worth_query(scope)]
    pub inbox: String,
    #[worth_query(probe)]
    pub prefix: String,
}

/// Sparse half-open focus index over the Optional `[focus_start, focus_end)`.
#[derive(WorthManagedIndex)]
#[worth_index(
    schema = SparseSchema, record = Task, source = ReadInboxBinding, probe = FocusWindow,
    kind = interval_1d(TaskView::focus_start(), TaskView::focus_end()),
    namespace = "sparse.dx.focus_overlap", version = 1,
)]
pub struct FocusIndex;

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = SparseSchema, context = Desktop, namespace = SPARSE, member,
    scope = InboxId, managed_index = FocusIndex, lanes = OneShotAndLive,
    order_by(TaskView::id(), Ascending), limits = 64,
)]
pub struct ReadFocus {
    #[worth_query(scope)]
    pub inbox: String,
    #[worth_query(probe)]
    pub window: FocusWindow,
}

#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = SparseSchema, context = Desktop, namespace = SPARSE, member,
    input = AddTaskInput, operation = AddTaskOp, binding = AddTaskBinding, delivery = TaskAdded,
    body, program, reads(Inbox), creates(Task), links(InboxTasks)
)]
pub struct AddTask {
    #[worth_query(scope = InboxId)]
    pub inbox: String,
    pub id: u64,
    pub title: Option<String>,
    pub start: Option<CanonicalTimestamp>,
    pub end: Option<CanonicalTimestamp>,
}

#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = SparseSchema, context = Desktop, namespace = SPARSE, member,
    input = EditTaskInput, operation = EditTaskOp, binding = EditTaskBinding,
    delivery = TaskEdited, body, program, reads(Task)
)]
pub struct EditTask {
    #[worth_query(scope = TaskId)]
    pub id: u64,
    pub title: Option<String>,
    pub start: Option<CanonicalTimestamp>,
    pub end: Option<CanonicalTimestamp>,
}

pub fn add_task_operation() -> ApplicationOperationDefinition<SparseSchema, AddTaskOp, AddTask> {
    AddTaskOp::reference()
        .typed_body()
        .in_scope(Inbox::reference())
        .create_in_scope(Task::reference(), "task")
        .initialize_input(TaskFacts::task_id(), AddTask::id_input())
        .initialize_optional_input(TaskFacts::title(), AddTask::title_input())
        .initialize_optional_input(TaskFacts::focus_start(), AddTask::start_input())
        .initialize_optional_input(TaskFacts::focus_end(), AddTask::end_input())
        .link_from_scope(InboxTasks::reference(), "inbox-task")
        .finish_pure_definition()
        .expect("add task body admits")
}

pub fn edit_task_operation() -> ApplicationOperationDefinition<SparseSchema, EditTaskOp, EditTask> {
    EditTaskOp::reference()
        .typed_body()
        .assign_optional_input(TaskFacts::title(), EditTask::title_input())
        .assign_optional_input(TaskFacts::focus_start(), EditTask::start_input())
        .assign_optional_input(TaskFacts::focus_end(), EditTask::end_input())
        .finish_pure_definition()
        .expect("edit task body admits")
}
