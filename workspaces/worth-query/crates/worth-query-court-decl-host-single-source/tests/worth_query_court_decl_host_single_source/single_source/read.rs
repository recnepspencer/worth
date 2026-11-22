//! One task read whose request, result and slot identities share a namespace
//! and whose principal clauses come from the board context.
use super::schema::*;
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::{
    application_query::*, worth_query_application_query,
    worth_query_identity, worth_query_identity_namespace, worth_query_portable_type,
    worth_query_query_binding, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::*;

pub const READ: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("single-source.tasks.read.v1");

#[derive(Clone, Debug, WorthValue)]
pub struct ReadTask {
    pub task: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
pub struct TaskRow {
    pub key: u64,
    pub title: String,
    pub archived: bool,
    pub pinned: bool,
}
worth_query_structured_value_binding!(pub RequestBinding for ReadTask {
    identity: worth_query_identity!(READ, request)
});
worth_query_structured_value_binding!(pub ParametersBinding for () {
    identity: worth_query_identity!(READ, parameters)
});
worth_query_structured_value_binding!(pub RowBinding for TaskRow {
    identity: worth_query_identity!(READ, result)
});
worth_query_application_query!(pub TaskQuery for Schema, identity worth_query_identity!(READ, query),
    parameters ParametersBinding, result RowBinding, scope Task => "Task", name "read_task");

pub struct KeySlot;
pub struct TitleSlot;
pub struct ArchivedSlot;
pub struct PinnedSlot;
worth_query_portable_type!(KeySlot => worth_query_identity!(READ, slot "key"));
worth_query_portable_type!(TitleSlot => worth_query_identity!(READ, slot "title"));
worth_query_portable_type!(ArchivedSlot => worth_query_identity!(READ, slot "archived"));
worth_query_portable_type!(PinnedSlot => worth_query_identity!(READ, slot "pinned"));

fn key() -> ApplicationQueryResultFieldRef<TaskQuery, KeySlot, Schema, TaskKey> {
    ApplicationQueryResultFieldRef::new("key", TaskKey::at("TaskKey"))
}
fn title() -> ApplicationQueryResultFieldRef<TaskQuery, TitleSlot, Schema, Title> {
    ApplicationQueryResultFieldRef::new("title", Title::at("Title"))
}
fn archived() -> ApplicationQueryResultFieldRef<TaskQuery, ArchivedSlot, Schema, Archived> {
    ApplicationQueryResultFieldRef::new("archived", Archived::at("Archived"))
}
fn pinned() -> ApplicationQueryResultFieldRef<TaskQuery, PinnedSlot, Schema, Pinned> {
    ApplicationQueryResultFieldRef::new("pinned", Pinned::at("Pinned"))
}

pub fn definition() -> ApplicationQueryDefinitionBuilder<Schema, TaskQuery, (), TaskRow, Task> {
    ApplicationQueryDefinitionBuilder::declare(TaskQuery::reference())
        .root(Task::reference())
        .scope(Task::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::new(Task::reference())
                .field(key())
                .field(title())
                .field(archived())
                .field(pinned())
                .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}
impl WorthQueryApplicationProjection<Schema, TaskQuery> for TaskRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, TaskQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key())?,
            title: row.field(title())?,
            archived: row.field(archived())?,
            pinned: row.field(pinned())?,
        })
    }
}

fn read_scope(input: ReadTask) -> u64 {
    input.task
}
worth_query_query_binding!(pub ReadBinding for ReadTask, schema Schema, context BoardContext,
    identity worth_query_identity!(READ, binding), input RequestBinding, query TaskQuery,
    parameters ParametersBinding => |_: &ReadTask| ApplicationQueryParameterSet::new(),
    result RowBinding,
    scope TaskKey, field TaskKey::at("TaskKey"), value read_scope, limits results 4
);
