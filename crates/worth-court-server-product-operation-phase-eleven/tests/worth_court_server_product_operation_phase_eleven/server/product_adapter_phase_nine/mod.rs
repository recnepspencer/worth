#[allow(unused_imports)]
use crate::product_adapter_phase_nine::ordinary_context::*;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy;

use serde_json::json;

#[allow(unused_imports)]
use worth_testkit_server::shared_support_product_adapter_phase_nine_fixture as fixture;

use {
    worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::build_server,
    worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::completed,
    worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::direct_session,
    worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::editor_registration,
    worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::open_mutation_product_session,
    worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::prepared_mutation_request,
    worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::prepared_read_request,
    worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::RequireTitleValidator,
};

#[test]
fn product_adapter_operation_parity_across_direct_and_http_surfaces() {
    let server = build_server(vec![editor_registration(None, None)]);
    let payload = worth_server::WorthServerProductOperationPayload::json(
        "product-editor.render.v1",
        json!({ "document": "doc-7" }),
    );

    let direct = completed(
        direct_session(&server).product_operations().execute(
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.render",
                payload.clone(),
            )
            .with_basis_digest("basis-7"),
        ),
    );
    let compat = completed(server.compat_http().product_operations().execute(
        &prepared_read_request(&server, "product_editor.render", Some("basis-7")),
        worth_server::WorthServerProductOperationInput::new("product_editor.render", payload),
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
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_server(vec![editor_registration(Some(calls.clone()), None)]);
    let payload = worth_server::WorthServerProductOperationPayload::json(
        "product-editor.apply.v1",
        json!({ "title": "Renamed" }),
    );

    let denial = direct_session(&server).product_operations().execute(
        worth_server::WorthServerProductOperationInput::new("product_editor.apply", payload)
            .with_basis_digest("basis-7"),
    );

    let denial = denial.expect_err("missing product session should deny before adapter execution");
    assert_eq!(
        denial.code(),
        worth_server::WorthServerProductOperationSurfaceDenialCode::AdmissionDenied
    );
    assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 0);
}

#[test]
fn product_denials_preserve_product_reason_keys_inside_server_envelopes() {
    let server = build_server(vec![editor_registration(None, None)]);
    let session = direct_session(&server);
    let product_session =
        open_mutation_product_session(&session, "product_editor.finalize", "basis-9");
    let payload = worth_server::WorthServerProductOperationPayload::json(
        "product-editor.finalize.v1",
        json!({ "title": "Ready", "deny_reason": "product.finalize.rebase_required" }),
    );

    let completed = completed(
        session.product_operations().execute(
            worth_server::WorthServerProductOperationInput::new("product_editor.finalize", payload)
                .with_basis_digest("basis-9")
                .with_product_session_identity(product_session.identity().as_str()),
        ),
    );

    let denial = match completed.outcome() {
        worth_server::WorthServerProductOperationOutcome::Denied(denial) => denial,
        other => panic!("expected product denial, got {other:?}"),
    };
    assert_eq!(denial.reason_key(), "product.finalize.rebase_required");
    assert_eq!(
        denial
            .facts()
            .expect("product semantic denial should expose server denial facts")
            .code(),
        worth_server::WorthServerProductOperationDenialCode::ProductSemantic
    );
    assert_eq!(
        completed.envelope().kind(),
        worth_server::WorthServerProductOperationEnvelopeKind::Denial
    );
}

#[test]
fn product_payload_schema_validation_is_adapter_declared_not_route_owned() {
    let validator = std::sync::Arc::new(RequireTitleValidator);
    let server = build_server(vec![editor_registration(None, Some(validator))]);
    let direct = direct_session(&server);
    let product_session = open_mutation_product_session(&direct, "product_editor.apply", "basis-7");
    let payload = worth_server::WorthServerProductOperationPayload::json(
        "product-editor.apply.v1",
        json!({ "body": "missing-title" }),
    );

    let direct = completed(
        direct.product_operations().execute(
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.apply",
                payload.clone(),
            )
            .with_basis_digest("basis-7")
            .with_product_session_identity(product_session.identity().as_str()),
        ),
    );
    let compat = completed(
        server.compat_http().product_operations().execute(
            &prepared_mutation_request(&server, "product_editor.apply", Some("basis-7")),
            worth_server::WorthServerProductOperationInput::new("product_editor.apply", payload)
                .with_product_session_identity(product_session.identity().as_str()),
        ),
    );

    for outcome in [direct.outcome(), compat.outcome()] {
        match outcome {
            worth_server::WorthServerProductOperationOutcome::Denied(denial) => {
                assert_eq!(denial.reason_key(), "missing_title");
                assert_eq!(
                    denial
                        .facts()
                        .expect("validator denial should expose server denial facts")
                        .code(),
                    worth_server::WorthServerProductOperationDenialCode::DeclaredPayloadValidator
                );
            }
            other => panic!("expected validator denial, got {other:?}"),
        }
    }
}

#[test]
fn malformed_product_payload_schema_is_classified_before_adapter_semantics() {
    let server = build_server(vec![editor_registration(None, None)]);
    let payload = worth_server::WorthServerProductOperationPayload::json(
        "product-editor.render.v999",
        json!({ "document": "doc-7" }),
    );

    let completed = completed(
        direct_session(&server).product_operations().execute(
            worth_server::WorthServerProductOperationInput::new("product_editor.render", payload)
                .with_basis_digest("basis-7"),
        ),
    );

    let denial = match completed.outcome() {
        worth_server::WorthServerProductOperationOutcome::Denied(denial) => denial,
        other => panic!("expected payload schema denial, got {other:?}"),
    };
    assert_eq!(denial.reason_key(), "invalid_payload_schema");
    assert_eq!(
        denial
            .facts()
            .expect("schema mismatch should expose server denial facts")
            .code(),
        worth_server::WorthServerProductOperationDenialCode::PayloadSchemaMismatch
    );
    assert!(completed.plan().is_none());
    assert!(completed.scheduler_admission().is_none());
}

#[test]
fn unsupported_product_support_is_denied_at_readiness_with_typed_surface_facts() {
    let server = build_server(vec![unsupported_render_registration()]);
    let payload = worth_server::WorthServerProductOperationPayload::json(
        "product-editor.render.v1",
        json!({ "document": "doc-9" }),
    );

    let denial = direct_session(&server)
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new("product_editor.render", payload)
                .with_basis_digest("basis-9"),
        )
        .expect_err("unsupported shared-read product support should deny at readiness");

    assert_eq!(
        denial.code(),
        worth_server::WorthServerProductOperationSurfaceDenialCode::ReadinessDenied
    );
    assert_eq!(
        denial.facts().and_then(|facts| facts.readiness_code()),
        Some(worth_server::WorthServerOperationReadinessDenialCode::UnsupportedProductSupport)
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
        let mut input =
            worth_server::WorthServerProductOperationInput::new(operation_name, payload)
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
