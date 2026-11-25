//! A second correctable action whose derives share one application namespace
//! through `member`, as an application with a single `APP` namespace declares them.
use super::program::{DaymarkProgram, RenameTaskAction, RestoreTaskTitleAction};
use super::queries::ReadTaskDetailsBinding;
use super::schema::*;
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::application_aftermath::*;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_identity_namespace, WorthQueryAction, WorthQueryCorrection,
};

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

pub fn rename_task_body() -> ApplicationOperationBody<DaymarkSchema, RenameTaskOperation, RenameTask>
{
    RenameTaskOperation::reference()
        .typed_body()
        .assign_input(TaskFacts::title(), RenameTask::title_input())
        .finish()
        .expect("rename_task body admits")
}

#[derive(Clone, Debug, PartialEq, Eq, WorthValue, serde::Deserialize, WorthQueryAction)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = APP,
    member,
    input = RestoreTaskTitleInput,
    operation = RestoreTaskTitleOperation,
    binding = RestoreTaskTitleBinding,
    delivery = RestoreTaskTitleDelivery,
    body,
    program,
    reads(Task),
)]
pub struct RestoreTaskTitle {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub expected_title: String,
    pub restored_title: String,
}

pub fn restore_task_title_definition(
) -> ApplicationOperationDefinition<DaymarkSchema, RestoreTaskTitleOperation, RestoreTaskTitle> {
    RestoreTaskTitleOperation::reference()
        .typed_body()
        .expect_input(TaskFacts::title(), RestoreTaskTitle::expected_title_input())
        .assign_input(TaskFacts::title(), RestoreTaskTitle::restored_title_input())
        .finish_pure_definition()
        .expect("restore_task_title body admits")
}

/// `member` narrows `APP` to `daymark.unified.app.v1.correct_task_title`.
#[derive(WorthQueryCorrection)]
#[worth_query(
    schema = DaymarkSchema,
    namespace = APP,
    member,
    mode = authored_body,
    source = RenameTaskBinding,
    inverse = RestoreTaskTitle,
    disclosure = ReadTaskDetailsBinding,
    source_body = rename_task_body,
    preimage_fields(TaskFacts::title()),
    preimage_bytes = 64,
    derive = derive_restore_task_title,
    reapply,
    program = DaymarkProgram,
    inverse_action = RestoreTaskTitleAction,
    source_action = RenameTaskAction,
)]
pub struct CorrectTaskTitle;

fn derive_restore_task_title(
    original: &RenameTask,
    preimage: &dyn ApplicationCorrectionPreImage<DaymarkSchema>,
) -> Result<RestoreTaskTitle, ApplicationCorrectionDerivationDenial> {
    Ok(RestoreTaskTitle {
        task: original.task,
        expected_title: original.title.clone(),
        restored_title: decode_preimage_field(preimage, TaskFacts::title())?,
    })
}
