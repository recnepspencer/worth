#[allow(unused_imports)]
use crate::shared_support_test_bootstrap_policy as test_bootstrap_policy;
use serde_json::json;
use worth_proof::TransitionOutcome;
use worth_server::{
    WorthServerOperationAuthorityDeclaration, WorthServerOperationAuthorizationPolicy,
    WorthServerOperationFamily, WorthServerOperationInputEnvelope,
    WorthServerOperationReadinessDenialCode, WorthServerOperationRegistration,
    WorthServerOperationRequestInput, WorthServerPipelineInput, WorthServerPipelineIntent,
    WorthServerProductSupportPosture, WorthServerSharedReadBasisKind, WorthServerSurfaceFamily,
};

#[allow(unused_imports)]
use crate::shared_support_operation_request_runtime as operation_request_runtime;
#[allow(unused_imports)]
use crate::shared_support_query_handoff_fixture as query_handoff_fixture;
#[allow(unused_imports)]
use crate::shared_support_query_handoff_runtime as query_handoff_runtime;

use operation_request_runtime::{
    operation_request_test_server, operation_request_test_server_with_operations,
    worth_native_resolved_context,
};
fn query_read_admission(server: &worth_server::WorthServer) -> worth_server::WorthServerAdmission {
    match server.middleware().admit(WorthServerPipelineInput::new(
        worth_native_resolved_context(server, None),
        WorthServerPipelineIntent::query_read("users.profile"),
    )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected query read admission, got {other:?}"),
    }
}

fn product_read_server(
    basis_kind: WorthServerSharedReadBasisKind,
    support_posture: WorthServerProductSupportPosture,
) -> worth_server::WorthServer {
    let mut registrations = WorthServerOperationRegistration::phase_two_defaults();
    registrations.retain(|registration| {
        registration.family() != WorthServerOperationFamily::ProductApplicationRead
    });
    registrations.push(
        WorthServerOperationRegistration::enabled(
            WorthServerOperationFamily::ProductApplicationRead,
        )
        .exposed_on([
            WorthServerSurfaceFamily::WorthNative,
            WorthServerSurfaceFamily::CompatHttp,
        ])
        .admit_operation_names(["editor.render"])
        .with_authority_declaration(
            WorthServerOperationAuthorityDeclaration::product_shared_read_with_support_posture(
                basis_kind,
                support_posture,
            ),
        )
        .with_authorization_policy(WorthServerOperationAuthorizationPolicy::allow_authenticated()),
    );
    operation_request_test_server_with_operations(registrations)
}

fn admitted_product_read_posture(
    server: &worth_server::WorthServer,
) -> worth_server::WorthServerOperationAdmissionPosture {
    let admission = query_read_admission(server);
    let request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::ProductApplicationRead)
                .with_operation_name("editor.render")
                .with_basis_digest("basis-users-profile")
                .build(),
        )
        .expect("product read request should admit");
    server
        .operation_admissions()
        .admit_declared(&admission, &request)
        .expect("product read posture should admit")
}

#[test]
fn query_and_product_support_compose_without_meaning_merge() {
    let server = operation_request_test_server();
    let query_read_admission = query_read_admission(&server);
    let query_read_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &query_read_admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::QueryDirectRead)
                .with_operation_name("users.profile")
                .with_basis_digest("basis-users-profile")
                .build(),
        )
        .expect("query read request should admit");
    let query_read_posture = server
        .operation_admissions()
        .admit_declared(&query_read_admission, &query_read_request)
        .expect("query read posture should admit");

    let query_denial = server
        .operation_readiness()
        .compose_support(&query_read_posture, None)
        .expect_err("query-dependent operations must not plan without query support");
    assert_eq!(
        query_denial.code(),
        WorthServerOperationReadinessDenialCode::MissingQuerySupport
    );

    let product_read_posture = admitted_product_read_posture(&product_read_server(
        WorthServerSharedReadBasisKind::ProductSessionDerived,
        WorthServerProductSupportPosture::ProductionAdmitted,
    ));
    let product_read_support = server
        .operation_readiness()
        .compose_support(&product_read_posture, None)
        .expect("query-independent product read should not require query support");

    assert!(product_read_support.query_support_posture().is_none());
    assert_eq!(
        product_read_support
            .composition_receipt()
            .dependency_relation(),
        "product-independent"
    );
    assert_eq!(
        product_read_support
            .composition_receipt()
            .query_rows_consulted()
            .len(),
        0
    );

    for (basis_kind, support_posture, expected_code) in [
        (
            WorthServerSharedReadBasisKind::ProductSessionDerived,
            WorthServerProductSupportPosture::Unsupported,
            WorthServerOperationReadinessDenialCode::UnsupportedProductSupport,
        ),
        (
            WorthServerSharedReadBasisKind::ProductSessionDerived,
            WorthServerProductSupportPosture::Unknown,
            WorthServerOperationReadinessDenialCode::UnknownProductSupport,
        ),
        (
            WorthServerSharedReadBasisKind::FixtureOnly,
            WorthServerProductSupportPosture::ProductionAdmitted,
            WorthServerOperationReadinessDenialCode::FixtureOnlyProductSupport,
        ),
        (
            WorthServerSharedReadBasisKind::ProductSessionDerived,
            WorthServerProductSupportPosture::IncompatibleBasis,
            WorthServerOperationReadinessDenialCode::IncompatibleSupportBasis,
        ),
    ] {
        let product_server = product_read_server(basis_kind, support_posture);
        let denial = product_server
            .operation_readiness()
            .compose_support(&admitted_product_read_posture(&product_server), None)
            .expect_err("non-production product support posture must deny during readiness");
        assert_eq!(denial.code(), expected_code);
    }
}

#[test]
fn concurrency_class_requires_support_and_precondition_closure() {
    let server = operation_request_test_server();
    let read_posture = admitted_product_read_posture(&product_read_server(
        WorthServerSharedReadBasisKind::ProductSessionDerived,
        WorthServerProductSupportPosture::ProductionAdmitted,
    ));
    let read_closure = server
        .operation_readiness()
        .close_readiness(&read_posture, None, None)
        .expect("comparable shared read should close");

    assert_eq!(
        read_closure.concurrency_class(),
        worth_server::WorthServerOperationConcurrencyClass::ConcurrentSharedRead
    );

    let mutation_admission = match server.middleware().admit(WorthServerPipelineInput::new(
        worth_native_resolved_context(&server, None),
        WorthServerPipelineIntent::query_mutation("editor.apply"),
    )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected mutation admission, got {other:?}"),
    };
    let mutation_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &mutation_admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::ProductApplicationMutation)
                .with_operation_name("editor.apply")
                .with_basis_digest("basis-editor")
                .with_idempotency_key("idem-editor")
                .with_product_session_identity("session-1")
                .with_payload_envelope(WorthServerOperationInputEnvelope::json(
                    "editor-apply.v1",
                    &json!({"command": "apply"}),
                ))
                .build(),
        )
        .expect("product mutation request should admit");
    let mutation_posture = server
        .operation_admissions()
        .admit_declared(&mutation_admission, &mutation_request)
        .expect("product mutation posture should admit");
    let mutation_closure = server
        .operation_readiness()
        .close_readiness(&mutation_posture, None, None)
        .expect("product mutation should close");

    assert_eq!(
        mutation_closure.concurrency_class(),
        worth_server::WorthServerOperationConcurrencyClass::SerializeDeterministically
    );

    let coordination_admission = match server.middleware().admit(WorthServerPipelineInput::new(
        worth_native_resolved_context(&server, None),
        WorthServerPipelineIntent::worth_native_session("editor.apply"),
    )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected Worth-native session admission, got {other:?}"),
    };
    let coordination_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &coordination_admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::ProductSessionCoordination)
                .with_operation_name("editor.apply")
                .with_product_session_identity("session-1")
                .build(),
        )
        .expect("session coordination request should admit");
    let coordination_posture = server
        .operation_admissions()
        .admit_declared(&coordination_admission, &coordination_request)
        .expect("session coordination posture should admit");
    assert_eq!(
        server
            .operation_readiness()
            .close_readiness(&coordination_posture, None, None)
            .expect("session coordination should close")
            .concurrency_class(),
        worth_server::WorthServerOperationConcurrencyClass::SerializeDeterministically
    );

    let unsupported_read_server = product_read_server(
        WorthServerSharedReadBasisKind::ProductSessionDerived,
        WorthServerProductSupportPosture::Unsupported,
    );
    let unsupported_denial = unsupported_read_server
        .operation_readiness()
        .close_readiness(
            &admitted_product_read_posture(&unsupported_read_server),
            None,
            None,
        )
        .expect_err("unsupported product support should deny before concurrency classification");
    assert_eq!(
        unsupported_denial.code(),
        WorthServerOperationReadinessDenialCode::UnsupportedProductSupport
    );
}
