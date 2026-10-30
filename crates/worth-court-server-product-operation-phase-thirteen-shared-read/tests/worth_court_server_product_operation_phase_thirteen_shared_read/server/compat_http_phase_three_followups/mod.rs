#[allow(unused_imports)]
use crate::compat_http_phase_three_followups::ordinary_context::*;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_compat_http_phase_three_runtime as compat_http_phase_three_runtime;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy;

use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};

use serde_json::json;
use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use {
    worth_server::WorthServerQueryHandoffDenialCode,
    worth_server::WorthServerQueryWorkspaceBindingError,
    worth_server::WorthServerQueryWorkspaceBindingRequest,
    worth_server::WorthServerQueryWorkspaceProvider,
};

use {
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::build_phase_three_server,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::build_phase_three_server_with_workspace_provider,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::compat_mutation_denied,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::compat_mutation_success,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::mutation_input,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::prepared_mutation_request,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::single_insert_body,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::StatefulCountingMutationWorkspaceProvider,
};

#[test]
fn compat_http_mutation_request_denials_preserve_requested_diagnostics_profile() {
    let server = build_phase_three_server();
    let denial = compat_mutation_denied(server.compat_http().mutate(
        worth_server::WorthServerCompatibilityMutationExecutionInput::new(
            prepared_mutation_request(
                &server,
                mutation_input("tasks.delete")
                    .with_diagnostics_profile(
                        worth_server::request_context::DiagnosticRichnessProfile::OperationalMinimal,
                    )
                    .build()
                    .expect("diagnostic-profile mutation input should validate structurally"),
            ),
            "tasks.delete",
            json!({
                "command": {
                    "family": "delete",
                    "entity_identity": "task-1",
                    "touched_aspect_paths": ["title.value", 7]
                }
            }),
        ),
    ));

    assert_eq!(
        denial.diagnostics_profile(),
        worth_server::request_context::DiagnosticRichnessProfile::OperationalMinimal
    );
    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid
    );
}

#[test]
fn compat_http_mutation_rejects_stale_validator_after_authoritative_write() {
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_phase_three_server_with_workspace_provider(
        worth_testkit_server::shared_support_compat_http_phase_three_runtime::StatefulCountingMutationWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            attempted_writes.clone(),
        ),
    );
    let first = compat_mutation_success(
        server.compat_http().mutate(
            worth_server::WorthServerCompatibilityMutationExecutionInput::new(
                prepared_mutation_request(
                    &server,
                    mutation_input("tasks.insert")
                        .build()
                        .expect("first stateful mutation input should validate structurally"),
                ),
                "tasks.insert",
                single_insert_body("task-1"),
            ),
        ),
    );
    let stale_validator_denial = compat_mutation_denied(
        server.compat_http().mutate(
            worth_server::WorthServerCompatibilityMutationExecutionInput::new(
                prepared_mutation_request(
                    &server,
                    mutation_input("tasks.insert")
                        .with_header("if-match", first.precondition().validator())
                        .build()
                        .expect("stale-validator mutation input should validate structurally"),
                ),
                "tasks.insert",
                single_insert_body("task-2"),
            ),
        ),
    );

    assert_eq!(
        stale_validator_denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityMutationPreconditionFailed
    );
    assert_eq!(
        stale_validator_denial
            .facts()
            .and_then(|facts| facts.expected_validator()),
        Some(first.precondition().validator())
    );
    assert_ne!(
        stale_validator_denial
            .facts()
            .and_then(|facts| facts.observed_validator()),
        stale_validator_denial
            .facts()
            .and_then(|facts| facts.expected_validator())
    );
    assert_eq!(
        attempted_writes.load(std::sync::atomic::Ordering::Relaxed),
        1,
        "stale validator denial must happen before any second write executes"
    );
}

#[test]
fn compat_http_mutation_planning_reuses_precondition_workspace_binding() {
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let bind_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_phase_three_server_with_workspace_provider(
        crate::compat_http_phase_three_followups::ordinary_context::BindCountingMutationWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            attempted_writes.clone(),
            bind_count.clone(),
        ),
    );

    let mutation = compat_mutation_success(
        server.compat_http().mutate(
            worth_server::WorthServerCompatibilityMutationExecutionInput::new(
                prepared_mutation_request(
                    &server,
                    mutation_input("tasks.insert")
                        .build()
                        .expect("bind-counted mutation input should validate structurally"),
                ),
                "tasks.insert",
                single_insert_body("task-bind-count"),
            ),
        ),
    );

    assert_eq!(
        bind_count.load(std::sync::atomic::Ordering::Relaxed),
        1,
        "compat mutation planning must reuse the workspace it already bound for precondition evaluation"
    );
    assert_eq!(
        attempted_writes.load(std::sync::atomic::Ordering::Relaxed),
        1,
        "the successful mutation should still execute exactly one write"
    );
    assert_eq!(
        mutation.operation_request().identity().operation_name(),
        "tasks.insert"
    );
}
