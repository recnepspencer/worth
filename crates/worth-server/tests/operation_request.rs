#[allow(unused_imports)]
use crate::shared_support_operation_request_runtime as operation_request_runtime;

use serde_json::json;
use worth_server::{
    request_context::DiagnosticRichnessProfile, WorthServerCompatHttpRouteFamily,
    WorthServerOperationFamily, WorthServerOperationInputEnvelope,
    WorthServerOperationRequestDenialCode, WorthServerOperationRequestInput,
};

use operation_request_runtime::{
    compat_prepared_request, operation_request_test_server, worth_native_admission,
    worth_native_resolved_context,
};

#[test]
fn equivalent_surface_inputs_lower_to_identical_operation_identity() {
    let server = operation_request_test_server();
    let payload = WorthServerOperationInputEnvelope::json(
        "task-command.v1",
        &json!({
            "command": {
                "family": "insert",
                "aspects": {
                    "title.value": "Task",
                    "identity.id": "task-1"
                }
            }
        }),
    );
    let compat = server
        .operation_requests()
        .admit_from_compat_http(
            &compat_prepared_request(
                &server,
                Some(DiagnosticRichnessProfile::Standard),
                WorthServerCompatHttpRouteFamily::Mutation,
                "POST",
                "/compat/mutations/tasks.insert",
                Some("basis:abc123"),
                Some("idem-42"),
            ),
            WorthServerOperationFamily::QueryDirectSubmission,
            "tasks.insert",
            Some(payload.clone()),
        )
        .expect("compat operation request should admit");
    let direct = server
        .operation_requests()
        .admit(
            worth_native_resolved_context(&server, Some(DiagnosticRichnessProfile::Standard)),
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::QueryDirectSubmission)
                .with_operation_name("tasks.insert")
                .with_basis_digest("basis:abc123")
                .with_idempotency_key("idem-42")
                .with_payload_envelope(payload)
                .build(),
        )
        .expect("Worth-native operation request should admit");

    assert_eq!(
        compat.identity().canonical_digest(),
        direct.identity().canonical_digest()
    );
    assert_ne!(
        compat.receipt().canonical_digest(),
        direct.receipt().canonical_digest()
    );
    assert_eq!(
        compat
            .payload_envelope()
            .expect("compat payload envelope")
            .canonical_digest(),
        direct
            .payload_envelope()
            .expect("direct payload envelope")
            .canonical_digest()
    );
}

#[test]
fn malformed_operation_request_denies_before_planning() {
    let server = operation_request_test_server();

    let missing_family = server.operation_requests().admit(
        worth_native_resolved_context(&server, None),
        WorthServerOperationRequestInput::builder()
            .with_operation_name("tasks.insert")
            .build(),
    );
    let invalid_idempotency = server.operation_requests().admit(
        worth_native_resolved_context(&server, None),
        WorthServerOperationRequestInput::builder()
            .with_operation_family(WorthServerOperationFamily::QueryDirectSubmission)
            .with_operation_name("tasks.insert")
            .with_idempotency_key("   ")
            .build(),
    );
    let invalid_payload_schema = server.operation_requests().admit(
        worth_native_resolved_context(&server, None),
        WorthServerOperationRequestInput::builder()
            .with_operation_family(WorthServerOperationFamily::QueryDirectSubmission)
            .with_operation_name("tasks.insert")
            .with_payload_envelope(WorthServerOperationInputEnvelope::json(
                "   ",
                &json!({"command": {"family": "insert"}}),
            ))
            .build(),
    );
    let binding_mismatch = server.operation_requests().admit_from_compat_http(
        &compat_prepared_request(
            &server,
            None,
            WorthServerCompatHttpRouteFamily::Read,
            "GET",
            "/compat/reads/users.profile",
            None,
            None,
        ),
        WorthServerOperationFamily::QueryDirectRead,
        "users.other",
        None,
    );

    assert_eq!(
        denied_code(missing_family),
        WorthServerOperationRequestDenialCode::MissingOperationFamily
    );
    assert_eq!(
        denied_code(invalid_idempotency),
        WorthServerOperationRequestDenialCode::InvalidIdempotencyKey
    );
    assert_eq!(
        denied_code(invalid_payload_schema),
        WorthServerOperationRequestDenialCode::InvalidDeclaredSchemaIdentity
    );
    assert_eq!(
        denied_code(binding_mismatch),
        WorthServerOperationRequestDenialCode::CompatibilityBindingInvalid
    );
}

#[test]
fn diagnostics_policy_does_not_change_semantic_operation_identity() {
    let server = operation_request_test_server();
    let input = WorthServerOperationRequestInput::builder()
        .with_operation_family(WorthServerOperationFamily::QueryDirectRead)
        .with_operation_name("users.profile")
        .build();
    let standard = server
        .operation_requests()
        .admit(
            worth_native_resolved_context(&server, Some(DiagnosticRichnessProfile::Standard)),
            input.clone(),
        )
        .expect("standard diagnostics request should admit");
    let minimal = server
        .operation_requests()
        .admit(
            worth_native_resolved_context(
                &server,
                Some(DiagnosticRichnessProfile::OperationalMinimal),
            ),
            input,
        )
        .expect("minimal diagnostics request should admit");

    assert_eq!(
        standard.identity().canonical_digest(),
        minimal.identity().canonical_digest()
    );
    assert_ne!(
        standard.receipt().canonical_digest(),
        minimal.receipt().canonical_digest()
    );
}

#[test]
fn operation_request_identity_rejects_display_string_ordering() {
    let server = operation_request_test_server();
    let left = WorthServerOperationInputEnvelope::json(
        "task-command.v1",
        &json!({
            "command": {
                "z": 1,
                "a": 2
            }
        }),
    );
    let right = WorthServerOperationInputEnvelope::json(
        "task-command.v1",
        &json!({
            "command": {
                "a": 2,
                "z": 1
            }
        }),
    );
    let left_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &worth_native_admission(&server, None, "tasks.insert"),
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::QueryDirectSubmission)
                .with_operation_name("Tasks.Insert")
                .with_payload_envelope(left)
                .build(),
        )
        .expect("left request should admit");
    let right_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &worth_native_admission(&server, None, "tasks.insert"),
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::QueryDirectSubmission)
                .with_operation_name("tasks.insert")
                .with_payload_envelope(right)
                .build(),
        )
        .expect("right request should admit");

    assert_eq!(
        left_request
            .payload_envelope()
            .expect("left payload envelope")
            .canonical_digest(),
        right_request
            .payload_envelope()
            .expect("right payload envelope")
            .canonical_digest()
    );
    assert_eq!(
        left_request.identity().canonical_digest(),
        right_request.identity().canonical_digest()
    );
}

fn denied_code(
    result: Result<
        worth_server::WorthServerOperationRequest,
        worth_server::WorthServerOperationRequestDenial,
    >,
) -> WorthServerOperationRequestDenialCode {
    match result {
        Ok(value) => panic!("expected denied operation request, got {value:?}"),
        Err(denial) => denial.code(),
    }
}
