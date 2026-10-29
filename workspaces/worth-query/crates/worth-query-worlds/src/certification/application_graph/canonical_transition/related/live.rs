//! Actual retained-effect delivery crosses the changed-codec installation boundary.
pub use super::*;
pub use crate::facade::certification::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::*,
    declaration::application_schema::ApplicationScalarValueBinding,
    primary_graph::*,
};
pub fn publish_old(host: &host::Host, id: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let result = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.runtime().current_world())
        .advanced()
        .mutate(publish::Publish {
            changed: cause::Changed {
                identity: "derived-report".into(),
            },
        })
        .without_source()
        .idempotency(&id)
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        result,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}
pub fn publish_target(host: &successor::host::Host, id: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let result = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.runtime().current_world())
        .advanced()
        .mutate(successor::publish::Publish {
            changed: successor::cause::Changed {
                identity: "derived-report".into(),
            },
        })
        .without_source()
        .idempotency(&id)
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        result,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}
