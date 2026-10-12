#[path = "support/compat_http/phase_three_runtime.rs"]
mod compat_http_phase_three_runtime;
#[path = "support/query_handoff/runtime.rs"]
mod query_handoff_runtime;
#[path = "support/test_execution_policy.rs"]
mod test_execution_policy;
#[path = "support/worth_native/assertions.rs"]
mod worth_native_assertions;
#[path = "support/worth_native/runtime.rs"]
mod worth_native_runtime;

use std::sync::atomic::Ordering;

use serde_json::json;
use worth_query::facade::runtime::{
    WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupport, WorthQueryRuntimeSupportProfile,
};
use worth_server::{WorthServerQueryHandoffDenialCode, WorthServerSuccessKind};

use compat_http_phase_three_runtime::{
    build_phase_three_server, build_phase_three_server_with_workspace_provider,
    compat_mutation_denied, compat_mutation_execution_input, compat_mutation_success,
    direct_mutation_success, insert_task, mutation_input, mutation_request_input_for_workspace,
    prepared_mutation_request, single_insert_body,
};
use query_handoff_runtime::ProfiledCountingTestWorkspaceProvider;
use worth_native_assertions::{
    family_contract_digest, response_provenance_digest, worth_native_session,
};

#[test]
fn compat_http_single_insert_matches_worth_native_mutation_on_shared_query_artifacts() {
    let server = build_phase_three_server();
    let body = single_insert_body("task-1");
    let compat = compat_mutation_success(server.compat_http().mutate(
        compat_mutation_execution_input(&server, "tasks.insert", body),
    ));
    let direct = direct_mutation_success(worth_native_session(&server).direct().mutate(
        &worth_server::WorthServerQueryOperation::single_mutation(
            "tasks.insert",
            insert_task("task-1"),
        ),
    ));

    assert_eq!(
        compat
            .envelope()
            .response_envelope()
            .success()
            .expect("compat response should succeed")
            .payload()
            .kind(),
        WorthServerSuccessKind::QueryMutation
    );
    assert_eq!(
        family_contract_digest(compat.envelope().support_posture()),
        family_contract_digest(direct.support_posture())
    );
    assert_eq!(
        compat.mutation_result().result_digest(),
        direct.mutation_result().result_digest()
    );
    assert_eq!(
        compat.mutation_result().inspection_digest(),
        direct.mutation_result().inspection_digest()
    );
    assert_eq!(
        response_provenance_digest(compat.envelope().response_envelope()),
        response_provenance_digest(direct.response_envelope())
    );
    assert!(!compat.envelope().retry_receipt().is_previously_completed());
}

#[test]
fn compat_http_mutation_denies_forbidden_and_unsupported_families() {
    let server = build_phase_three_server();
    let forbidden =
        compat_mutation_denied(server.compat_http().mutate(compat_mutation_execution_input(
            &server,
            "tasks.verify-existing",
            json!({
                "command": {
                    "family": "verify_existing",
                    "authoritative_identity": "authority:task-1",
                    "resolved_entity_identity": "task-1",
                    "target_collection": "Task",
                    "asserted_aspects": { "title.value": "Expected title" }
                }
            }),
        )));
    let unsupported =
        compat_mutation_denied(server.compat_http().mutate(compat_mutation_execution_input(
            &server,
            "tasks.custom",
            json!({
                "command": {
                    "family": "merge",
                    "entity_identity": "task-1",
                    "aspects": { "title.value": "Unexpected" }
                }
            }),
        )));

    assert_eq!(
        forbidden.code(),
        WorthServerQueryHandoffDenialCode::CompatibilityMutationFamilyForbidden
    );
    assert!(forbidden
        .detail()
        .contains("forbidden at the external server boundary"));
    assert_eq!(
        unsupported.code(),
        WorthServerQueryHandoffDenialCode::CompatibilityMutationFamilyUnsupported
    );
    assert!(unsupported.detail().contains("`merge` is not supported"));
}

#[test]
fn compat_http_mutation_preconditions_deny_before_any_write_attempt() {
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_phase_three_server_with_workspace_provider(
        ProfiledCountingTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            attempted_writes.clone(),
        ),
    );
    let mismatch_basis = worth_server::WorthServerCompatibilityMutationExecutionInput::new(
        prepared_mutation_request(
            &server,
            mutation_input("tasks.insert")
                .with_query_pair("basis", "basis:drifted")
                .build()
                .expect("basis mutation input should validate structurally"),
        ),
        "tasks.insert",
        single_insert_body("task-1"),
    );
    let mismatch_validator = worth_server::WorthServerCompatibilityMutationExecutionInput::new(
        prepared_mutation_request(
            &server,
            mutation_input("tasks.insert")
                .with_header("if-match", "\"validator:wrong\"")
                .build()
                .expect("validator mutation input should validate structurally"),
        ),
        "tasks.insert",
        single_insert_body("task-2"),
    );

    let basis_denial = compat_mutation_denied(server.compat_http().mutate(mismatch_basis));
    let validator_denial = compat_mutation_denied(server.compat_http().mutate(mismatch_validator));

    assert_eq!(
        basis_denial.code(),
        WorthServerQueryHandoffDenialCode::CompatibilityMutationPreconditionFailed
    );
    assert_eq!(
        basis_denial
            .facts()
            .and_then(|facts| facts.expected_basis_digest()),
        Some("basis:drifted")
    );
    assert_ne!(
        basis_denial
            .facts()
            .and_then(|facts| facts.observed_basis_digest()),
        basis_denial
            .facts()
            .and_then(|facts| facts.expected_basis_digest())
    );
    assert_eq!(
        validator_denial.code(),
        WorthServerQueryHandoffDenialCode::CompatibilityMutationPreconditionFailed
    );
    assert_eq!(
        validator_denial
            .facts()
            .and_then(|facts| facts.expected_validator()),
        Some("\"validator:wrong\"")
    );
    assert_ne!(
        validator_denial
            .facts()
            .and_then(|facts| facts.observed_validator()),
        validator_denial
            .facts()
            .and_then(|facts| facts.expected_validator())
    );
    assert_eq!(
        attempted_writes.load(Ordering::Relaxed),
        0,
        "compat mutation precondition failures must deny before any write executes"
    );
}

#[test]
fn compat_http_idempotency_resolves_identical_retries_and_denies_conflicts() {
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_phase_three_server_with_workspace_provider(
        ProfiledCountingTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            attempted_writes.clone(),
        ),
    );
    let first = compat_mutation_success(
        server.compat_http().mutate(
            worth_server::WorthServerCompatibilityMutationExecutionInput::new(
                prepared_mutation_request(
                    &server,
                    mutation_input("tasks.insert")
                        .with_header("idempotency-key", "idem-1")
                        .build()
                        .expect("first idempotent mutation input should validate structurally"),
                ),
                "tasks.insert",
                single_insert_body("task-1"),
            ),
        ),
    );
    let retry = compat_mutation_success(
        server.compat_http().mutate(
            worth_server::WorthServerCompatibilityMutationExecutionInput::new(
                prepared_mutation_request(
                    &server,
                    mutation_input("tasks.insert")
                        .with_header("idempotency-key", "idem-1")
                        .build()
                        .expect("retried idempotent mutation input should validate structurally"),
                ),
                "tasks.insert",
                single_insert_body("task-1"),
            ),
        ),
    );
    let conflict = compat_mutation_denied(
        server.compat_http().mutate(
            worth_server::WorthServerCompatibilityMutationExecutionInput::new(
                prepared_mutation_request(
                    &server,
                    mutation_input("tasks.insert")
                        .with_header("idempotency-key", "idem-1")
                        .build()
                        .expect(
                            "conflicting idempotent mutation input should validate structurally",
                        ),
                ),
                "tasks.insert",
                single_insert_body("task-2"),
            ),
        ),
    );

    assert!(!first.envelope().retry_receipt().is_previously_completed());
    assert!(retry.envelope().retry_receipt().is_previously_completed());
    assert_eq!(
        first.mutation_result().result_digest(),
        retry.mutation_result().result_digest()
    );
    assert_eq!(
        attempted_writes.load(Ordering::Relaxed),
        1,
        "identical idempotent retries must not execute a second write"
    );
    assert_eq!(
        conflict.code(),
        WorthServerQueryHandoffDenialCode::CompatibilityIdempotencyConflict
    );
    assert_eq!(
        conflict.facts().and_then(|facts| facts.idempotency_key()),
        Some("idem-1")
    );
    assert_ne!(
        conflict
            .facts()
            .and_then(|facts| facts.conflicting_request_digest()),
        conflict
            .facts()
            .and_then(|facts| facts.bound_request_digest())
    );
}

#[test]
fn compat_http_idempotency_scope_isolated_by_workspace_target() {
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_phase_three_server_with_workspace_provider(
        ProfiledCountingTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            attempted_writes.clone(),
        ),
    );
    let first = compat_mutation_success(
        server.compat_http().mutate(
            worth_server::WorthServerCompatibilityMutationExecutionInput::new(
                prepared_mutation_request(
                    &server,
                    mutation_input("tasks.insert")
                        .with_header("idempotency-key", "shared-key")
                        .build()
                        .expect("first scoped idempotent mutation input should validate"),
                ),
                "tasks.insert",
                single_insert_body("task-1"),
            ),
        ),
    );
    let second = compat_mutation_success(
        server.compat_http().mutate(
            worth_server::WorthServerCompatibilityMutationExecutionInput::new(
                prepared_mutation_request(
                    &server,
                    mutation_request_input_for_workspace("tasks.insert", "workspace-77")
                        .with_header("accept", "application/json")
                        .with_header("idempotency-key", "shared-key")
                        .build()
                        .expect("second scoped idempotent mutation input should validate"),
                ),
                "tasks.insert",
                single_insert_body("task-2"),
            ),
        ),
    );

    assert!(!first.envelope().retry_receipt().is_previously_completed());
    assert!(!second.envelope().retry_receipt().is_previously_completed());
    assert_eq!(
        attempted_writes.load(Ordering::Relaxed),
        2,
        "the same idempotency key must not collide across workspace scopes"
    );
}

#[test]
fn compat_http_mutation_denies_before_write_when_inspect_family_is_unavailable() {
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_phase_three_server_with_workspace_provider(
        ProfiledCountingTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
                WorthQueryRuntimeFamilySupport::unsupported(
                    WorthQueryRuntimeFacadeFamily::Inspect,
                    "inspect is intentionally denied in this hostile compat mutation profile",
                ),
            ),
            attempted_writes.clone(),
        ),
    );

    let denial = compat_mutation_denied(server.compat_http().mutate(
        compat_mutation_execution_input(&server, "tasks.insert", single_insert_body("task-1")),
    ));

    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
    );
    assert!(denial
        .detail()
        .contains("does not admit `inspect` facade family"));
    assert_eq!(
        attempted_writes.load(Ordering::Relaxed),
        0,
        "inspect family denial must happen before any compatibility mutation write"
    );
}
