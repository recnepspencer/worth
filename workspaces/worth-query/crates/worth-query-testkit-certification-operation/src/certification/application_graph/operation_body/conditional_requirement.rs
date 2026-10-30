//! Authored Foundation expressions drive collection guidance and actual admission.
pub use super::{
    draft::{DraftInput, DraftIntent},
    program, request_scope,
    requirement_control::UnlinkIntent,
    schema::*,
};
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
pub type App<const MODE: u64> =
    worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<MODE>,
        program::BodyProgram<MODE>,
    >;
pub type Projection<const MODE: u64> =
    worth_query_host::facade::primary_graph::WorthQueryApplicationInvariantProjectionAuthority<
        DocumentRetentionSchema<MODE>,
    >;

pub fn guidance<const MODE: u64>(app: &App<MODE>, projection: &Projection<MODE>) -> Status {
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    let assessed = app
        .request(&principal, &scope)
        .advanced()
        .mutate(DraftIntent::<MODE, RequireDraftNote>::new(""))
        .assess_current_body_requirements(app, projection)
        .expect("authorized native assessment");
    assert_eq!(assessed.findings().len(), 1);
    let finding = &assessed.findings()[0];
    assert_eq!(
        finding.identity(),
        Some(if MODE == 28 {
            "nonempty-note"
        } else {
            "members-minimum"
        })
    );
    assert_eq!(finding.field().is_none(), MODE != 28);
    finding.status()
}

pub fn invoke<const MODE: u64>(
    app: &App<MODE>,
    key: u64,
) -> Outcome<WorthQueryOperationBodyDenial, ()> {
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .advanced()
        .mutate(DraftIntent::<MODE, RequireDraftNote>::new(""))
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}

pub fn unlink<const MODE: u64>(app: &App<MODE>, key: u64) {
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    assert!(matches!(
        app.request(&principal, &scope)
            .advanced()
            .mutate(UnlinkIntent::<MODE>(DraftInput {
                identity: super::DOCUMENT_IDENTITY.to_owned(),
                note: String::new()
            }))
            .without_source()
            .idempotency(&key)
            .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        Outcome::Committed { .. }
    ));
}

#[path = "conditional_requirement/portable.rs"]
pub mod portable;
