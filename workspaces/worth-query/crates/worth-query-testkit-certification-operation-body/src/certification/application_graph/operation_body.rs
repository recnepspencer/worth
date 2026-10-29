//! Native literal-body execution. Static fixture resources are not NCR create acceptance.
#[path = "operation_body/advanced_commit.rs"]
pub mod advanced_commit;
#[path = "operation_body/audit_query.rs"]
pub mod audit_query;
#[path = "operation_body/binding.rs"]
pub mod binding;
#[path = "operation_body/body_only.rs"]
pub mod body_only;
#[path = "operation_body/conditional_requirement.rs"]
pub mod conditional_requirement;
#[path = "operation_body/creation.rs"]
pub mod creation;
#[path = "operation_body/creation_link.rs"]
pub mod creation_link;
#[path = "operation_body/draft.rs"]
pub mod draft;
#[path = "operation_body/draft_query.rs"]
pub mod draft_query;
#[path = "operation_body/genesis.rs"]
pub mod genesis;
#[path = "operation_body/input.rs"]
pub mod input;
#[path = "operation_body/input_expectation.rs"]
pub mod input_expectation;
#[path = "operation_body/input_selection.rs"]
pub mod input_selection;
#[path = "operation_body/installation.rs"]
pub mod installation;
#[path = "operation_body/optional_input.rs"]
pub mod optional_input;
#[path = "operation_body/output.rs"]
pub mod output;
#[path = "operation_body/persistence.rs"]
pub mod persistence;
#[path = "operation_body/principal.rs"]
pub mod principal;
#[path = "operation_body/program.rs"]
pub mod program;
#[path = "operation_body/query.rs"]
pub mod query;
#[path = "operation_body/replacement.rs"]
pub mod replacement;
#[path = "operation_body/requirement.rs"]
pub mod requirement;
#[path = "operation_body/requirement_control.rs"]
pub mod requirement_control;
#[path = "operation_body/retention_body.rs"]
pub mod retention_body;
#[path = "operation_body/retirement.rs"]
pub mod retirement;
#[path = "operation_body/schema.rs"]
pub mod schema;
pub use installation::{install, install_with_projection};

pub use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::host_limits,
    operator_identity::{block_on, operator_identity, request_scope, OperatorAdapter},
    retention_entry::DOCUMENT_IDENTITY,
};
pub use schema::*;
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
    },
    application_installation::{in_memory_program, WorthQueryProgramApplicationRuntime},
    declaration::authentication::WorthQueryPrincipalMappingStatus,
    primary_graph::{
        WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
        WorthQueryApplicationPrincipalKey, WorthQueryApplicationRelationSeed,
        WorthQueryOperationBodyDenial,
    },
    runtime::ExecutionAllocationPolicy,
};

pub fn invoke<const DESTINATION: u64>(
    app: &WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >,
    key: u64,
) -> Outcome<WorthQueryOperationBodyDenial, ()> {
    invoke_input(
        app,
        key,
        SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: 99,
            ..Default::default()
        },
    )
}

pub fn invoke_input<const DESTINATION: u64>(
    app: &WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >,
    key: u64,
    input: SetRetentionInput,
) -> Outcome<WorthQueryOperationBodyDenial, ()> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .advanced()
        .mutate(binding::BodyIntent(input))
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}













pub fn authenticate_operator<Schema: BodySchemaBinding>(
    schema: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<Schema>,
    scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
) -> worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>{
    use worth_query_host::facade::admission::authenticated_principal::*;
    let adapter = admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("document-retention-host").unwrap(),
            WorthQueryAuthenticationMethod::new("certification").unwrap(),
        ),
        OperatorAdapter,
    )
    .unwrap();
    block_on(adapter.authenticate((), scope)).unwrap()
}

pub fn read_retention<const DESTINATION: u64>(
    app: &WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >,
) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .query(query::DocumentRetentionRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .retention_days
}
