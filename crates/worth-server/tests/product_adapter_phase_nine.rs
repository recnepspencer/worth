#[allow(unused_imports)]
use crate::shared_support_test_bootstrap_policy as test_bootstrap_policy;

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use serde_json::json;
use worth_server::{
    WorthServerOperationReadinessDenialCode, WorthServerProductApplicationAdapterRegistration,
    WorthServerProductOperationBasisKind, WorthServerProductOperationDeclaration,
    WorthServerProductOperationDenialCode, WorthServerProductOperationErrorMaps,
    WorthServerProductOperationInput, WorthServerProductOperationOutcome,
    WorthServerProductOperationPayload, WorthServerProductOperationSupportSnapshot,
    WorthServerProductOperationSurfaceDenialCode,
};

#[allow(unused_imports)]
use crate::shared_support_product_adapter_phase_nine_fixture as fixture;

use fixture::{
    build_server, completed, direct_session, editor_registration, open_mutation_product_session,
    prepared_mutation_request, prepared_read_request, result_contract, EditorAdapter,
    RequireTitleValidator,
};

#[test]
fn product_adapter_operation_parity_across_direct_and_http_surfaces() {
    let server = build_server(vec![editor_registration(None, None)]);
    let payload = WorthServerProductOperationPayload::json(
        "product-editor.render.v1",
        json!({ "document": "doc-7" }),
    );

    let direct = completed(
        direct_session(&server).product_operations().execute(
            WorthServerProductOperationInput::new("product_editor.render", payload.clone())
                .with_basis_digest("basis-7"),
        ),
    );
    let compat = completed(server.compat_http().product_operations().execute(
        &prepared_read_request(&server, "product_editor.render", Some("basis-7")),
        WorthServerProductOperationInput::new("product_editor.render", payload),
    ));

    assert_eq!(direct.outcome(), compat.outcome());
    assert_eq!(direct.envelope(), compat.envelope());
    assert_eq!(
        direct
            .plan()
            .expect("direct product execution should expose plan")
            .canonical_digest(),
        compat
            .plan()
            .expect("compat product execution should expose plan")
            .canonical_digest()
    );
    assert_eq!(
        direct
            .scheduler_admission()
            .expect("direct product execution should expose scheduler admission")
            .scheduler_lane(),
        compat
            .scheduler_admission()
            .expect("compat product execution should expose scheduler admission")
            .scheduler_lane()
    );
    assert_eq!(
        direct
            .support_posture()
            .expect("direct support posture")
            .canonical_digest(),
        compat
            .support_posture()
            .expect("compat support posture")
            .canonical_digest()
    );
    assert_eq!(
        direct
            .precondition_posture()
            .expect("direct precondition posture")
            .canonical_digest(),
        compat
            .precondition_posture()
            .expect("compat precondition posture")
            .canonical_digest()
    );
}

#[test]
fn product_adapter_cannot_bypass_server_operation_runtime() {
    let calls = Arc::new(AtomicUsize::new(0));
    let server = build_server(vec![editor_registration(Some(calls.clone()), None)]);
    let payload = WorthServerProductOperationPayload::json(
        "product-editor.apply.v1",
        json!({ "title": "Renamed" }),
    );

    let denial = direct_session(&server).product_operations().execute(
        WorthServerProductOperationInput::new("product_editor.apply", payload)
            .with_basis_digest("basis-7"),
    );

    let denial = denial.expect_err("missing product session should deny before adapter execution");
    assert_eq!(
        denial.code(),
        WorthServerProductOperationSurfaceDenialCode::AdmissionDenied
    );
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}

#[test]
fn product_denials_preserve_product_reason_keys_inside_server_envelopes() {
    let server = build_server(vec![editor_registration(None, None)]);
    let session = direct_session(&server);
    let product_session =
        open_mutation_product_session(&session, "product_editor.finalize", "basis-9");
    let payload = WorthServerProductOperationPayload::json(
        "product-editor.finalize.v1",
        json!({ "title": "Ready", "deny_reason": "product.finalize.rebase_required" }),
    );

    let completed = completed(
        session.product_operations().execute(
            WorthServerProductOperationInput::new("product_editor.finalize", payload)
                .with_basis_digest("basis-9")
                .with_product_session_identity(product_session.identity().as_str()),
        ),
    );

    let denial = match completed.outcome() {
        WorthServerProductOperationOutcome::Denied(denial) => denial,
        other => panic!("expected product denial, got {other:?}"),
    };
    assert_eq!(denial.reason_key(), "product.finalize.rebase_required");
    assert_eq!(
        denial
            .facts()
            .expect("product semantic denial should expose server denial facts")
            .code(),
        WorthServerProductOperationDenialCode::ProductSemantic
    );
    assert_eq!(
        completed.envelope().kind(),
        worth_server::WorthServerProductOperationEnvelopeKind::Denial
    );
}

#[test]
fn product_payload_schema_validation_is_adapter_declared_not_route_owned() {
    let validator = Arc::new(RequireTitleValidator);
    let server = build_server(vec![editor_registration(None, Some(validator))]);
    let direct = direct_session(&server);
    let product_session = open_mutation_product_session(&direct, "product_editor.apply", "basis-7");
    let payload = WorthServerProductOperationPayload::json(
        "product-editor.apply.v1",
        json!({ "body": "missing-title" }),
    );

    let direct = completed(
        direct.product_operations().execute(
            WorthServerProductOperationInput::new("product_editor.apply", payload.clone())
                .with_basis_digest("basis-7")
                .with_product_session_identity(product_session.identity().as_str()),
        ),
    );
    let compat = completed(
        server.compat_http().product_operations().execute(
            &prepared_mutation_request(&server, "product_editor.apply", Some("basis-7")),
            WorthServerProductOperationInput::new("product_editor.apply", payload)
                .with_product_session_identity(product_session.identity().as_str()),
        ),
    );

    for outcome in [direct.outcome(), compat.outcome()] {
        match outcome {
            WorthServerProductOperationOutcome::Denied(denial) => {
                assert_eq!(denial.reason_key(), "missing_title");
                assert_eq!(
                    denial
                        .facts()
                        .expect("validator denial should expose server denial facts")
                        .code(),
                    WorthServerProductOperationDenialCode::DeclaredPayloadValidator
                );
            }
            other => panic!("expected validator denial, got {other:?}"),
        }
    }
}

#[test]
fn malformed_product_payload_schema_is_classified_before_adapter_semantics() {
    let server = build_server(vec![editor_registration(None, None)]);
    let payload = WorthServerProductOperationPayload::json(
        "product-editor.render.v999",
        json!({ "document": "doc-7" }),
    );

    let completed = completed(
        direct_session(&server).product_operations().execute(
            WorthServerProductOperationInput::new("product_editor.render", payload)
                .with_basis_digest("basis-7"),
        ),
    );

    let denial = match completed.outcome() {
        WorthServerProductOperationOutcome::Denied(denial) => denial,
        other => panic!("expected payload schema denial, got {other:?}"),
    };
    assert_eq!(denial.reason_key(), "invalid_payload_schema");
    assert_eq!(
        denial
            .facts()
            .expect("schema mismatch should expose server denial facts")
            .code(),
        WorthServerProductOperationDenialCode::PayloadSchemaMismatch
    );
    assert!(completed.plan().is_none());
    assert!(completed.scheduler_admission().is_none());
}

#[test]
fn unsupported_product_support_is_denied_at_readiness_with_typed_surface_facts() {
    let server = build_server(vec![unsupported_render_registration()]);
    let payload = WorthServerProductOperationPayload::json(
        "product-editor.render.v1",
        json!({ "document": "doc-9" }),
    );

    let denial = direct_session(&server)
        .product_operations()
        .execute(
            WorthServerProductOperationInput::new("product_editor.render", payload)
                .with_basis_digest("basis-9"),
        )
        .expect_err("unsupported shared-read product support should deny at readiness");

    assert_eq!(
        denial.code(),
        WorthServerProductOperationSurfaceDenialCode::ReadinessDenied
    );
    assert_eq!(
        denial.facts().and_then(|facts| facts.readiness_code()),
        Some(WorthServerOperationReadinessDenialCode::UnsupportedProductSupport)
    );
}

#[test]
fn product_editor_shaped_operations_register_without_server_semantics() {
    let server = build_server(vec![editor_registration(None, None)]);
    let session = direct_session(&server);
    let product_session =
        open_mutation_product_session(&session, "product_editor.apply", "basis-9");
    let receipt = &server.product_adapter_inventory()[0];

    assert_eq!(receipt.adapter_label(), "editor-adapter");
    assert!(receipt
        .operation_names()
        .iter()
        .any(|name| name == "product_editor.render"));
    assert!(receipt
        .operation_names()
        .iter()
        .any(|name| name == "product_editor.finalize"));

    for (operation_name, payload) in editor_execution_cases() {
        let mut input = WorthServerProductOperationInput::new(operation_name, payload)
            .with_basis_digest("basis-9");
        if operation_name.contains("apply") || operation_name.contains("finalize") {
            input = input.with_product_session_identity(product_session.identity().as_str());
        }
        let completed = completed(session.product_operations().execute(input));
        assert_eq!(completed.envelope().operation_name(), operation_name);
        assert_eq!(
            completed
                .scheduler_admission()
                .expect("registered editor-shaped operation should expose scheduler proof")
                .scheduler_lane(),
            expected_scheduler_lane(operation_name, product_session.identity().as_str())
        );
    }
}

fn editor_execution_cases() -> [(&'static str, WorthServerProductOperationPayload); 5] {
    [
        (
            "product_editor.render",
            WorthServerProductOperationPayload::json(
                "product-editor.render.v1",
                json!({ "document": "doc-1" }),
            ),
        ),
        (
            "product_editor.select",
            WorthServerProductOperationPayload::json(
                "product-editor.select.v1",
                json!({ "node": "node-1" }),
            ),
        ),
        (
            "product_editor.available_actions",
            WorthServerProductOperationPayload::json(
                "product-editor.actions.v1",
                json!({ "node": "node-1" }),
            ),
        ),
        (
            "product_editor.apply",
            WorthServerProductOperationPayload::json(
                "product-editor.apply.v1",
                json!({ "title": "Rename" }),
            ),
        ),
        (
            "product_editor.finalize",
            WorthServerProductOperationPayload::json(
                "product-editor.finalize.v1",
                json!({ "title": "Finalize" }),
            ),
        ),
    ]
}

fn expected_scheduler_lane(operation_name: &str, product_session_identity: &str) -> String {
    if operation_name.contains("apply") || operation_name.contains("finalize") {
        format!("product-draft:{product_session_identity}:draft")
    } else {
        "shared-read".to_string()
    }
}

fn unsupported_render_registration() -> WorthServerProductApplicationAdapterRegistration {
    WorthServerProductApplicationAdapterRegistration::new(
        "unsupported-render",
        Arc::new(EditorAdapter::default()),
    )
    .with_operation(
        WorthServerProductOperationDeclaration::product_read(
            "product_editor.render",
            "product-editor.render.v1",
            result_contract("product-editor.render.result.v1"),
            WorthServerProductOperationBasisKind::DurableProductDerived,
            WorthServerProductOperationSupportSnapshot::unsupported("render-unsupported"),
        )
        .with_error_map(WorthServerProductOperationErrorMaps::passthrough()),
    )
}
