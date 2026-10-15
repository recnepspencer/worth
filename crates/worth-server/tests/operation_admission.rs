use serde_json::json;
use worth_proof::TransitionOutcome;
use worth_server::{
    WorthServerOperationAdmissionDenialCode, WorthServerOperationAuthorityDeclaration,
    WorthServerOperationAuthorityKind, WorthServerOperationAuthorizationPolicy,
    WorthServerOperationConcurrencyClass, WorthServerOperationConcurrencyDenialCode,
    WorthServerOperationConcurrencyFacade, WorthServerOperationFamily,
    WorthServerOperationInputEnvelope, WorthServerOperationRegistration,
    WorthServerOperationRequestInput, WorthServerOperationScope, WorthServerPipelineInput,
    WorthServerPipelineIntent, WorthServerProductSessionCoordinationTarget,
    WorthServerSharedReadBasisKind, WorthServerSurfaceFamily,
};

#[allow(unused_imports)]
use crate::shared_support_operation_request_runtime as operation_request_runtime;

use operation_request_runtime::{
    operation_request_test_server, operation_request_test_server_with_operations,
    worth_native_resolved_context, worth_native_resolved_context_for_principal,
};

#[path = "operation_admission/session_creation.rs"]
mod session_creation;

#[test]
fn declared_shared_read_footprints_admit_concurrent_planning() {
    let server = operation_request_test_server();
    let resolved = worth_native_resolved_context(&server, None);
    let admission = match server.middleware().admit(WorthServerPipelineInput::new(
        resolved,
        WorthServerPipelineIntent::query_read("users.profile"),
    )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected read admission, got {other:?}"),
    };

    let first = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::QueryDirectRead)
                .with_operation_name("users.profile")
                .with_basis_digest("basis-users-profile")
                .build(),
        )
        .expect("first operation request should admit");
    let second = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::QueryDirectRead)
                .with_operation_name("users.profile")
                .with_basis_digest("basis-users-profile")
                .build(),
        )
        .expect("second operation request should admit");

    let first_posture = server
        .operation_admissions()
        .admit_declared(&admission, &first)
        .expect("first posture should admit");
    let second_posture = server
        .operation_admissions()
        .admit_declared(&admission, &second)
        .expect("second posture should admit");
    let concurrency = WorthServerOperationConcurrencyFacade
        .classify_pair(&first_posture, &second_posture)
        .expect("equivalent shared reads should admit concurrent classification");

    assert_eq!(
        first_posture.authority_footprint().authority_kind(),
        WorthServerOperationAuthorityKind::SharedReadOnly
    );
    assert_eq!(
        concurrency,
        WorthServerOperationConcurrencyClass::ConcurrentSharedRead
    );
    assert_eq!(
        first_posture.authority_footprint().canonical_digest(),
        second_posture.authority_footprint().canonical_digest()
    );
    assert_eq!(
        first_posture.footprint_receipt().canonical_digest(),
        second_posture.footprint_receipt().canonical_digest()
    );
}

#[test]
fn authorization_proof_is_required_after_footprint_classification() {
    let server =
        operation_request_test_server_with_operations([WorthServerOperationRegistration::enabled(
            WorthServerOperationFamily::QueryDirectRead,
        )
        .exposed_on([
            WorthServerSurfaceFamily::WorthNative,
            WorthServerSurfaceFamily::CompatHttp,
        ])
        .admit_operation_names(["users.profile"])
        .with_authority_declaration(WorthServerOperationAuthorityDeclaration::query_shared_read())
        .with_authorization_policy(
            WorthServerOperationAuthorizationPolicy::require_principal("principal-9"),
        )]);
    let resolved = worth_native_resolved_context_for_principal(&server, None, "principal-7");
    let read_admission = match server.middleware().admit(WorthServerPipelineInput::new(
        resolved,
        WorthServerPipelineIntent::query_read("users.profile"),
    )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected read admission, got {other:?}"),
    };
    let operation_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &read_admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::QueryDirectRead)
                .with_operation_name("users.profile")
                .with_basis_digest("basis-users-profile")
                .build(),
        )
        .expect("read request should admit");

    let denial = server
        .operation_admissions()
        .admit_declared(&read_admission, &operation_request)
        .expect_err("unauthorized principal should deny after valid footprint classification");

    assert_eq!(
        denial.code(),
        WorthServerOperationAdmissionDenialCode::AuthorizationDenied
    );
    assert!(denial.detail().contains("principal `principal-7`"));
}

#[test]
fn ambiguous_operation_authority_cannot_fall_back_to_global_lock() {
    let server = operation_request_test_server();
    let resolved = worth_native_resolved_context(&server, None);
    let admission = match server.middleware().admit(WorthServerPipelineInput::new(
        resolved,
        WorthServerPipelineIntent::query_read("users.profile"),
    )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected read admission, got {other:?}"),
    };
    let operation_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::QueryDirectRead)
                .with_operation_name("users.profile")
                .with_basis_digest("basis-users-profile")
                .build(),
        )
        .expect("read operation request should admit");

    let denial = server
        .operation_admissions()
        .admit(
            &admission,
            &operation_request,
            worth_server::WorthServerOperationAuthorityMetadata::deterministic_submission(
                "query-write",
                "query-write-review",
                "caller-basis-bound",
                "best-effort",
            ),
        )
        .expect_err("mismatched authority metadata should deny");

    assert_eq!(
        denial.code(),
        WorthServerOperationAdmissionDenialCode::AuthorityDenied
    );
    assert!(denial
        .detail()
        .contains("does not admit the declared authority metadata"));
}

#[test]
fn fixture_only_product_read_basis_is_admitted_as_declared_authority_fact() {
    let server = worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    worth_server::WorthServerRequestContextConfig::builder()
                        .with_default_diagnostics_profile(
                            worth_server::request_context::DiagnosticRichnessProfile::Standard,
                        )
                        .with_preview_targeting_enabled(true)
                        .build()
                        .expect("request context config should validate"),
                )
                .with_middleware_config(
                    worth_server::WorthServerMiddlewareConfig::builder()
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operation(
            WorthServerOperationRegistration::enabled(
                WorthServerOperationFamily::ProductApplicationRead,
            )
            .exposed_on([WorthServerSurfaceFamily::WorthNative])
            .admit_operation_names(["editor.render"])
            .with_authority_declaration(
                WorthServerOperationAuthorityDeclaration::product_shared_read(
                    WorthServerSharedReadBasisKind::FixtureOnly,
                ),
            ),
        )
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("fixture-only product read registration should build for readiness-owned denial");

    assert_eq!(server.surface_inventory().registered_families.len(), 2);
}

#[test]
fn shared_read_metadata_must_match_admitted_operation_basis() {
    let server = operation_request_test_server();
    let resolved = worth_native_resolved_context(&server, None);
    let admission = match server.middleware().admit(WorthServerPipelineInput::new(
        resolved,
        WorthServerPipelineIntent::query_read("users.profile"),
    )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected read admission, got {other:?}"),
    };
    let operation_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::QueryDirectRead)
                .with_operation_name("users.profile")
                .with_basis_digest("basis-users-profile")
                .build(),
        )
        .expect("read operation request should admit");

    let denial = server
        .operation_admissions()
        .admit(
            &admission,
            &operation_request,
            worth_server::WorthServerOperationAuthorityMetadata::shared_read(
                "query-shared-read-basis",
                "basis-users-profile-drifted",
                "users.profile",
            ),
        )
        .expect_err("drifted metadata basis should deny");

    assert_eq!(
        denial.code(),
        WorthServerOperationAdmissionDenialCode::AuthorityDenied
    );
    assert!(denial
        .detail()
        .contains("does not match the admitted operation basis"));
}

#[test]
fn authorization_proof_distinguishes_payload_identity() {
    let server = operation_request_test_server();
    let resolved = worth_native_resolved_context(&server, None);
    let admission = match server.middleware().admit(WorthServerPipelineInput::new(
        resolved,
        WorthServerPipelineIntent::query_mutation("tasks.insert"),
    )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected mutation admission, got {other:?}"),
    };
    let first = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::QueryDirectSubmission)
                .with_operation_name("tasks.insert")
                .with_payload_envelope(WorthServerOperationInputEnvelope::json(
                    "task-command.v1",
                    &json!({"command": {"family": "insert", "id": "task-1"}}),
                ))
                .build(),
        )
        .expect("first mutation request should admit");
    let second = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::QueryDirectSubmission)
                .with_operation_name("tasks.insert")
                .with_payload_envelope(WorthServerOperationInputEnvelope::json(
                    "task-command.v1",
                    &json!({"command": {"family": "insert", "id": "task-2"}}),
                ))
                .build(),
        )
        .expect("second mutation request should admit");

    let first_posture = server
        .operation_admissions()
        .admit_declared(&admission, &first)
        .expect("first mutation posture should admit");
    let second_posture = server
        .operation_admissions()
        .admit_declared(&admission, &second)
        .expect("second mutation posture should admit");

    assert_ne!(
        first_posture.authorization_proof().canonical_digest(),
        second_posture.authorization_proof().canonical_digest()
    );
}

#[test]
fn conflicting_product_draft_footprints_fail_before_execution() {
    let server = operation_request_test_server();
    let resolved = worth_native_resolved_context(&server, None);
    let admission = match server.middleware().admit(WorthServerPipelineInput::new(
        resolved,
        WorthServerPipelineIntent::query_mutation("editor.apply"),
    )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected mutation admission, got {other:?}"),
    };
    let first = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::ProductApplicationMutation)
                .with_operation_name("editor.apply")
                .with_basis_digest("basis-editor-1")
                .with_idempotency_key("idem-editor-1")
                .with_product_session_identity("session-1")
                .build(),
        )
        .expect("first product mutation should admit");
    let second = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::ProductApplicationMutation)
                .with_operation_name("editor.apply")
                .with_basis_digest("basis-editor-2")
                .with_idempotency_key("idem-editor-2")
                .with_product_session_identity("session-1")
                .build(),
        )
        .expect("second product mutation should admit");

    let first_posture = server
        .operation_admissions()
        .admit_declared(&admission, &first)
        .expect("first product posture should admit");
    let second_posture = server
        .operation_admissions()
        .admit_declared(&admission, &second)
        .expect("second product posture should admit");

    let denial = WorthServerOperationConcurrencyFacade
        .classify_pair(&first_posture, &second_posture)
        .expect_err("same product draft scope should not admit concurrent mutation");

    assert_eq!(
        denial.code(),
        WorthServerOperationConcurrencyDenialCode::ConflictingMutableAuthority
    );
    assert!(denial.detail().contains("product draft scope"));
}
