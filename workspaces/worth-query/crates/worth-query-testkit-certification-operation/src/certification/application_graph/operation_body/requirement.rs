//! One installed presence rule drives descriptive guidance and direct admission.
pub use super::{draft::*, program, request_scope};
pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::{
        WorthQueryApplicationUncommitted, WorthQueryOperationBodyDenial,
        WorthQueryOperationBodyRequirementStatus as Status,
    },
    runtime::ExecutionAllocationPolicy,
};

pub type App<const DESTINATION: u64> =
    worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >;
pub type Projection<const DESTINATION: u64> =
    worth_query_host::facade::primary_graph::WorthQueryApplicationInvariantProjectionAuthority<
        DocumentRetentionSchema<DESTINATION>,
    >;

pub fn guidance<const DESTINATION: u64>(
    app: &App<DESTINATION>,
    projection: &Projection<DESTINATION>,
) -> Vec<Status> {
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    let assessed = app
        .request(&principal, &scope)
        .advanced()
        .mutate(DraftIntent::<DESTINATION, RequireDraftNote>::new(""))
        .assess_current_body_requirements(app, projection)
        .expect("authorized native assessment");
    assessed
        .findings()
        .iter()
        .map(|finding| {
            assert_eq!(
                finding.field().expect("presence target").field(),
                "DocumentDraftNoteField"
            );
            finding.status()
        })
        .collect()
}

pub fn invoke<Operation: DraftOperation<DocumentRetentionSchema<20>>>(
    app: &App<20>,
    key: u64,
    note: &str,
) -> Outcome<WorthQueryOperationBodyDenial, ()> {
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .advanced()
        .mutate(DraftIntent::<20, Operation>::new(note))
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}

pub fn invoke_requirement<const DESTINATION: u64>(
    app: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >,
    key: u64,
) -> Outcome<WorthQueryOperationBodyDenial, ()> {
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .advanced()
        .mutate(DraftIntent::<DESTINATION, RequireDraftNote>::new(""))
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}
