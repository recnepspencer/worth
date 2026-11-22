//! Two derived body actions and one written with the declaration macros,
//! side by side in one feature.
use super::schema::*;
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_schema::*, worth_query_identity,
    worth_query_identity_namespace, worth_query_mutation_binding, worth_query_operation,
    worth_query_structured_value_binding, WorthQueryAction,
};
use worth_query_host::facade::primary_graph::*;

pub const RENAME: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.tasks.rename.v1");
pub const RESCHEDULE: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.tasks.reschedule.v1");
pub const COMPLETE: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.tasks.complete.v1");

#[derive(Clone, Debug, WorthValue, WorthQueryAction)]
#[worth_query(schema = Schema, context = BoardContext, namespace = RENAME,
    input = RenameInput, operation = RenameTask, binding = RenameBinding, delivery = Renamed,
    result = BodyResultBinding, decision = WorthQueryOperationBodyDecision<Schema, Task>,
    body, program, access(read_fields: [TaskKey::at("TaskKey")];), reads(Task),
    validate = titled)]
pub struct Rename {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub title: String,
}

/// Refuses a blank title when the rename's input is captured.
pub fn titled(rename: &Rename) -> Result<(), &'static str> {
    (!rename.title.trim().is_empty())
        .then_some(())
        .ok_or("a task needs a title")
}

/// A native optional date: `Some` sets the due date, `None` clears it.
/// `CanonicalDate` is captured through its `Serialize` carrier.
#[derive(Clone, Debug, serde::Serialize, WorthQueryAction)]
#[worth_query(schema = Schema, context = BoardContext, namespace = RESCHEDULE,
    input = RescheduleInput, operation = RescheduleTask, binding = RescheduleBinding,
    delivery = Rescheduled, result = BodyResultBinding,
    decision = WorthQueryOperationBodyDecision<Schema, Task>, body, program,
    access(read_fields: [TaskKey::at("TaskKey")];), reads(Task))]
pub struct Reschedule {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub due: Option<CanonicalDate>,
}

/// The same declaration written out with the declaration macros.
#[derive(Clone, Debug, WorthValue)]
pub struct Complete {
    pub task: u64,
    pub done: bool,
}
worth_query_structured_value_binding!(pub CompleteInput for Complete {
    identity: worth_query_identity!(COMPLETE, input),
    capture_input: true
});
worth_query_operation!(pub CompleteTask for Schema, input CompleteInput;
    access { read_fields: [TaskKey::at("TaskKey")]; };
    reads [Task]
);
fn complete_scope(input: &Complete) -> u64 {
    input.task
}
worth_query_mutation_binding!(pub CompleteBinding for Complete, schema Schema,
    context BoardContext, identity worth_query_identity!(COMPLETE, binding),
    input CompleteInput, operation CompleteTask, result BodyResultBinding,
    decision WorthQueryOperationBodyDecision<Schema, Task>,
    handler identity worth_query_identity!(COMPLETE, body), program required,
    outputs NoApplicationMutationOutputs,
    scope TaskKey, field TaskKey::at("TaskKey"), value complete_scope,
);
pub struct Completed;
impl ApplicationActionDelivery<Schema, CompleteBinding> for Completed {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = worth_query_identity!(COMPLETE, completion);
}

pub fn rename_definition() -> ApplicationOperationDefinition<Schema, RenameTask, Rename> {
    let body = RenameTask::reference()
        .body()
        .assign_input(Title::at("Title"), "title")
        .expect("the column is writable")
        .finish()
        .expect("the rename body is complete");
    RenameTask::reference()
        .definition()
        .no_external_effect()
        .no_aftermath()
        .finish()
        .with_body(body)
        .expect("the body belongs to its operation")
}

pub fn reschedule_definition() -> ApplicationOperationDefinition<Schema, RescheduleTask, Reschedule>
{
    let body = RescheduleTask::reference()
        .body()
        .assign_optional_input(Due::at("Due"), "due")
        .expect("the column is writable")
        .finish()
        .expect("the reschedule body is complete");
    RescheduleTask::reference()
        .definition()
        .no_external_effect()
        .no_aftermath()
        .finish()
        .with_body(body)
        .expect("the body belongs to its operation")
}

pub fn complete_definition() -> ApplicationOperationDefinition<Schema, CompleteTask, Complete> {
    let body = CompleteTask::reference()
        .body()
        .assign_input(Done::at("Done"), "done")
        .expect("the column is writable")
        .finish()
        .expect("the complete body is complete");
    CompleteTask::reference()
        .definition()
        .no_external_effect()
        .no_aftermath()
        .finish()
        .with_body(body)
        .expect("the body belongs to its operation")
}
