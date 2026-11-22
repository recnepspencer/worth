//! A concrete-schema feature: one authored-body action and one read,
//! each identity derived from one namespace and each binding from the context.
use super::read;
use super::schema::*;
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_schema::*,
    worth_query_identity, worth_query_identity_namespace, worth_query_mutation_binding,
    worth_query_operation, worth_query_structured_value_binding,
};
use worth_query_host::facade::{primary_graph::*, worth_query_application_contribution};

pub const RENAME: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("single-source.tasks.rename.v1");

#[derive(Clone, Debug, WorthValue)]
pub struct Rename {
    pub task: u64,
    pub title: String,
}
worth_query_structured_value_binding!(pub RenameInputBinding for Rename {
    identity: worth_query_identity!(RENAME, input),
    capture_input: true
});
worth_query_operation!(pub RenameTask for Schema, input RenameInputBinding;
    access { read_fields: [TaskKey::at("TaskKey")]; };
    reads [Task]
);
fn rename_scope(input: &Rename) -> u64 {
    input.task
}
worth_query_mutation_binding!(pub RenameBinding for Rename, schema Schema, context BoardContext,
    identity worth_query_identity!(RENAME, binding), input RenameInputBinding,
    operation RenameTask, result BodyResultBinding,
    decision WorthQueryOperationBodyDecision<Schema, Task>,
    handler identity worth_query_identity!(RENAME, body), program required,
    outputs NoApplicationMutationOutputs,
    scope TaskKey, field TaskKey::at("TaskKey"), value rename_scope,
);
pub struct Renamed;
impl ApplicationActionDelivery<Schema, RenameBinding> for Renamed {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = worth_query_identity!(RENAME, completion);
}

thread_local! {
    /// When set, the rename operation is declared without its authored body.
    pub static OMIT_BODY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub fn rename_definition() -> ApplicationOperationDefinition<Schema, RenameTask, Rename> {
    let definition = RenameTask::reference()
        .definition()
        .no_external_effect()
        .no_aftermath()
        .finish();
    if OMIT_BODY.with(std::cell::Cell::get) {
        return definition;
    }
    let body = RenameTask::reference()
        .body()
        .assign_input(Title::at("Title"), "title")
        .expect("the title is writable")
        .finish()
        .expect("the rename body is complete");
    definition
        .with_body(body)
        .expect("the body belongs to its operation")
}

pub struct TaskFeature;
impl ApplicationFeature<Schema> for TaskFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "single-source.tasks.feature.v1";
}

worth_query_application_contribution! {
    pub feature TasksDefinition<Schema> for TaskFeature => TasksRuntime {
        schema: |schema| {
            schema
                .application_query(read::definition())
                .application_query_binding::<read::ReadBinding>()
        };
        body_action(RenameBinding, Renamed) {
            operation: rename_definition();
        };
    }
}
