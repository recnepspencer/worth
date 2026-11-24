//! An authored inverse of a recorded board completion, derived via
//! `#[derive(WorthQueryCorrection)]` and typed body authoring.
use super::{actions::*, reads::*, schema::*};
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::{
    application_aftermath::*, application_identity::ApplicationIdentityNamespace,
    application_query::*, application_schema::*, worth_query_identity_namespace, WorthQuery,
    WorthQueryAction, WorthQueryCorrection,
};

const CORRECTION: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.tasks.correct-completion.v1");
const DETAILS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.tasks.correction-details.v1");
const RESTORE: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.tasks.restore-done.v1");

#[derive(Clone, Debug, PartialEq, Eq, WorthValue, serde::Deserialize, WorthQueryAction)]
#[worth_query(schema = Schema, context = BoardContext, namespace = RESTORE,
    input = RestoreDoneInput, operation = RestoreDone, binding = RestoreDoneBinding,
    delivery = RestoredDone, body, program, reads(Task))]
pub struct RestoreCompletion {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub expected: bool,
    pub done: bool,
}

pub fn restore_done() -> ApplicationOperationDefinition<Schema, RestoreDone, RestoreCompletion> {
    RestoreDone::reference()
        .typed_body()
        .expect_input(TaskFacts::done(), RestoreCompletion::expected_input())
        .assign_input(TaskFacts::done(), RestoreCompletion::done_input())
        .finish_pure_definition()
        .expect("restore_done body admits")
}

#[derive(WorthQueryCorrection)]
#[worth_query(
    schema = Schema,
    namespace = CORRECTION,
    source = CompleteBinding,
    inverse = RestoreCompletion,
    disclosure = DetailsBinding,
    source_body = complete_body,
    preimage_fields(TaskFacts::done()),
    preimage_bytes = BoolApplicationValueBinding::encode(&true).unwrap().semantic_byte_width(),
    derive = derive_restore_completion,
)]
pub struct CorrectCompletion;

fn derive_restore_completion(
    original: &Complete,
    preimage: &dyn ApplicationCorrectionPreImage<Schema>,
) -> Result<RestoreCompletion, ApplicationCorrectionDerivationDenial> {
    Ok(RestoreCompletion {
        task: original.task,
        expected: original.done,
        done: decode_preimage_field(preimage, TaskFacts::done())?,
    })
}

#[derive(Clone, serde::Serialize, WorthQuery)]
#[worth_query(
    schema = Schema,
    context = BoardContext,
    namespace = DETAILS,
    query = DetailsQuery,
    input = ReadDetailsInput,
    binding = DetailsBinding,
    result = RowBinding,
    root = Task,
    cardinality = ExactlyOne,
    limits = 1,
    name = "correction_task",
)]
pub struct ReadDetails {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
}

pub fn task_details() -> ApplicationQueryDefinitionBuilder<Schema, DetailsQuery, (), TaskRow, Task>
{
    DetailsQuery::definition()
}
