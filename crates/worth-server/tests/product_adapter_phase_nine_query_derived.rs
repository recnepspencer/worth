#[allow(unused_imports)]
use crate::shared_support_test_bootstrap_policy as test_bootstrap_policy;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use serde_json::json;
use worth_server::{
    WorthServerOperationReadinessDenialCode, WorthServerProductOperationInput,
    WorthServerProductOperationPayload, WorthServerProductOperationSurfaceDenialCode,
};

#[allow(unused_imports)]
use crate::shared_support_product_adapter_phase_nine_fixture as fixture;

use fixture::{
    build_server, completed, direct_session, open_mutation_product_session,
    query_derived_editor_registration,
};

#[test]
fn query_derived_product_read_exposes_real_query_support_and_basis_proof() {
    let server = build_server(vec![query_derived_editor_registration(None)]);
    let payload = WorthServerProductOperationPayload::json(
        "product-editor.render.v1",
        json!({ "document": "doc-query-derived" }),
    );

    let denial = direct_session(&server)
        .product_operations()
        .execute(
            WorthServerProductOperationInput::new("product_editor.render", payload.clone())
                .with_basis_digest("basis:stale"),
        )
        .expect_err("stale caller basis should be denied before query-derived planning");

    assert_eq!(
        denial.code(),
        WorthServerProductOperationSurfaceDenialCode::ReadinessDenied
    );
    let denial_facts = denial
        .facts()
        .expect("query-derived basis denial should expose typed surface facts");
    assert_eq!(
        denial_facts.readiness_code(),
        Some(WorthServerOperationReadinessDenialCode::PreconditionFailed)
    );
    assert_eq!(denial_facts.expected_basis_digest(), Some("basis:stale"));
    let observed_basis_digest = denial_facts
        .observed_basis_digest()
        .expect("query-derived basis denial should expose observed workspace basis")
        .to_string();

    let completed = completed(
        direct_session(&server).product_operations().execute(
            WorthServerProductOperationInput::new("product_editor.render", payload)
                .with_basis_digest(&observed_basis_digest),
        ),
    );
    let support_posture = completed
        .support_posture()
        .expect("query-derived success should expose support proof");
    assert!(support_posture.query_support_posture().is_some());
    assert_eq!(
        support_posture.composition_receipt().dependency_relation(),
        "query-dependent"
    );

    let precondition = completed
        .precondition_posture()
        .and_then(|posture| posture.product_basis())
        .expect("query-derived success should expose product basis precondition proof");
    assert_eq!(precondition.operation_name(), "product_editor.render");
    assert_eq!(
        precondition.requested_basis_digest(),
        Some(observed_basis_digest.as_str())
    );
    assert_eq!(precondition.observed_basis_digest(), observed_basis_digest);
    assert_eq!(
        completed
            .scheduler_admission()
            .expect("query-derived success should expose scheduler proof")
            .scheduler_lane(),
        "shared-read"
    );
}

#[test]
fn query_derived_product_mutation_denies_stale_basis_before_adapter_execution() {
    let calls = Arc::new(AtomicUsize::new(0));
    let server = build_server(vec![query_derived_editor_registration(Some(calls.clone()))]);
    let session = direct_session(&server);
    let product_session =
        open_mutation_product_session(&session, "product_editor.apply", "basis:observed");
    let payload = WorthServerProductOperationPayload::json(
        "product-editor.apply.v1",
        json!({ "title": "Rename from query-derived product mutation" }),
    );

    let denial = session
        .product_operations()
        .execute(
            WorthServerProductOperationInput::new("product_editor.apply", payload)
                .with_basis_digest("basis:stale")
                .with_product_session_identity(product_session.identity().as_str()),
        )
        .expect_err("stale query-derived mutation basis should stop before adapter execution");

    assert_eq!(
        denial.code(),
        WorthServerProductOperationSurfaceDenialCode::AdmissionDenied
    );
    assert!(denial.detail().contains("explicit rebind"));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}
