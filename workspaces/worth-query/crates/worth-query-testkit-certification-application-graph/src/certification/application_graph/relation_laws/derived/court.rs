//! Genuine facade publication judges proposed sources and maintains disposable counts.
pub use super::super::{
    installation::{install, World},
    outcome, read,
};
pub use super::*;
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use std::marker::PhantomData;

pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial,
    },
    application_installation::WorthQueryProgramApplicationRuntime,
    runtime::ExecutionAllocationPolicy,
};

pub fn count(
    application: &WorthQueryProgramApplicationRuntime<
        GraphSchema<Derived>,
        super::super::program::GraphProgram<Derived>,
    >,
    command: u64,
    bad_reference: bool,
) -> Result<
    WorthQueryApplicationMutationOutcome<super::super::operation::GraphDenial, operation::Counted>,
    WorthQueryApplicationRequestMutationDenial,
> {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    application
        .runtime()
        .request(&principal, &scope)
        .on_branch(application.current_world())
        .advanced()
        .mutate(operation::Intent::<Derived> {
            input: operation::Input {
                root: 0,
                bad_reference,
            },
            marker: PhantomData,
        })
        .without_source()
        .idempotency(&command)
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
}
pub fn value(world: &World<Derived>, command: u64, expected: u64) {
    let outcome = count(&world.application, command, false).unwrap();
    assert_eq!(
        outcome.result().map(|result| result.count),
        Some(expected),
        "{outcome:?}"
    );
}
