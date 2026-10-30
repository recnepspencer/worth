//! Business entities start at the ordinary entry, never in bootstrap.
#[path = "genesis/binding.rs"]
pub mod binding;
#[path = "genesis/declaration.rs"]
pub mod declaration;
#[path = "genesis/portable.rs"]
pub mod portable;
pub use super::*;
pub use binding::{CreateInput, CreateIntent};

pub type Application =
    WorthQueryProgramApplicationRuntime<DocumentRetentionSchema<30>, program::BodyProgram<30>>;
pub fn input(tenant: u64) -> CreateInput {
    CreateInput {
        tenant,
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: 7,
        note: None,
    }
}
pub fn create(
    app: &Application,
    key: u64,
    input: CreateInput,
) -> Result<
    Outcome<WorthQueryOperationBodyDenial, ()>,
    worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial,
> {
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .advanced()
        .mutate(CreateIntent(input))
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
}
