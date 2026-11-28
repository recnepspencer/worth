//! Task rename declared under the application namespace.
use super::schema::*;
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{worth_query_identity_namespace, WorthQueryAction};

pub const APP: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.app.v1");

#[derive(Clone, Debug, PartialEq, Eq, WorthValue, serde::Deserialize, WorthQueryAction)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = APP,
    member,
    input = RenameTaskInput,
    operation = RenameTaskOperation,
    binding = RenameTaskBinding,
    delivery = RenameTaskDelivery,
    body,
    program,
    reads(Task),
)]
pub struct RenameTask {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub title: String,
}

pub fn rename_task_definition(
) -> ApplicationOperationDefinition<DaymarkSchema, RenameTaskOperation, RenameTask> {
    RenameTaskOperation::reference()
        .typed_body()
        .assign_input(TaskFacts::title(), RenameTask::title_input())
        .finish_pure_definition()
        .expect("rename_task body admits")
}
