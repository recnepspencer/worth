//! An authored-body correction over an optional field: the pre-image records
//! `None` when the task was undated, and the inverse guards the due date with
//! `expect_optional_input` before restoring it.
use super::program::{DaymarkProgram, EditTaskDetailsAction, RestoreTaskDetailsAction};
use super::queries::ReadTaskDetailsBinding;
use super::schema::*;
use super::title_correction::APP;
use worth_foundational::facade::CanonicalDate;
use worth_query_decl::facade::application_aftermath::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{WorthQueryAction, WorthQueryCorrection};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, WorthQueryAction)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = APP,
    member,
    input = EditTaskDetailsInput,
    operation = EditTaskDetailsOperation,
    binding = EditTaskDetailsBinding,
    delivery = EditTaskDetailsDelivery,
    body,
    program,
    reads(Task),
)]
pub struct EditTaskDetails {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub title: String,
    pub due_date: Option<CanonicalDate>,
}

pub fn edit_task_details_body(
) -> ApplicationOperationBody<DaymarkSchema, EditTaskDetailsOperation, EditTaskDetails> {
    EditTaskDetailsOperation::reference()
        .typed_body()
        .assign_input(TaskFacts::title(), EditTaskDetails::title_input())
        .assign_optional_input(TaskFacts::due_date(), EditTaskDetails::due_date_input())
        .finish()
        .expect("edit_task_details body admits")
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, WorthQueryAction)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = APP,
    member,
    input = RestoreTaskDetailsInput,
    operation = RestoreTaskDetailsOperation,
    binding = RestoreTaskDetailsBinding,
    delivery = RestoreTaskDetailsDelivery,
    body,
    program,
    reads(Task),
)]
pub struct RestoreTaskDetails {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub expected_title: String,
    pub expected_due_date: Option<CanonicalDate>,
    pub restored_title: String,
    pub restored_due_date: Option<CanonicalDate>,
}

pub fn restore_task_details_definition(
) -> ApplicationOperationDefinition<DaymarkSchema, RestoreTaskDetailsOperation, RestoreTaskDetails>
{
    RestoreTaskDetailsOperation::reference()
        .typed_body()
        .expect_input(
            TaskFacts::title(),
            RestoreTaskDetails::expected_title_input(),
        )
        .expect_optional_input(
            TaskFacts::due_date(),
            RestoreTaskDetails::expected_due_date_input(),
        )
        .assign_input(
            TaskFacts::title(),
            RestoreTaskDetails::restored_title_input(),
        )
        .assign_optional_input(
            TaskFacts::due_date(),
            RestoreTaskDetails::restored_due_date_input(),
        )
        .finish_pure_definition()
        .expect("restore_task_details body admits")
}

#[derive(WorthQueryCorrection)]
#[worth_query(
    schema = DaymarkSchema,
    namespace = APP,
    member,
    mode = authored_body,
    source = EditTaskDetailsBinding,
    inverse = RestoreTaskDetails,
    disclosure = ReadTaskDetailsBinding,
    source_body = edit_task_details_body,
    preimage_fields(TaskFacts::title(), TaskFacts::due_date()),
    preimage_bytes = 256,
    derive = derive_restore_task_details,
    reapply,
    program = DaymarkProgram,
    inverse_action = RestoreTaskDetailsAction,
    source_action = EditTaskDetailsAction,
)]
pub struct CorrectTaskDetails;

fn derive_restore_task_details(
    original: &EditTaskDetails,
    preimage: &dyn ApplicationCorrectionPreImage<DaymarkSchema>,
) -> Result<RestoreTaskDetails, ApplicationCorrectionDerivationDenial> {
    Ok(RestoreTaskDetails {
        task: original.task,
        expected_title: original.title.clone(),
        expected_due_date: original.due_date,
        restored_title: decode_preimage_field(preimage, TaskFacts::title())?,
        restored_due_date: decode_optional_preimage_field(preimage, TaskFacts::due_date())?,
    })
}
