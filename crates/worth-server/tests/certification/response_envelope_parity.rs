use worth_foundational::facade::FoundationalBoundaryEvidenceSupportTruthKind;
use worth_server::{
    request_context::DiagnosticRichnessProfile, WorthServerMiddlewareConfig,
    WorthServerOperatorEvidenceConfig, WorthServerQueryHandoffOperation, WorthServerResponseConfig,
    WorthServerResponseTransform, WorthServerSurfaceFamily, WorthServerTransportClass,
};

use crate::shared_support_query_handoff_runtime::TestWorkspaceProvider;
use crate::shared_support_response_fixture::{
    operator_evidence_record, query_handoff_success, shape_success, test_server,
    test_server_with_request_context_and_operator_evidence_config,
    test_server_with_response_config,
};

#[test]
fn shape_preserves_canonical_success_envelope_truth_across_surface_families() {
    let server = test_server(
        TestWorkspaceProvider,
        WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
    );

    let worth_native = shape_success(
        &server,
        query_handoff_success(
            &server,
            WorthServerSurfaceFamily::WorthNative,
            WorthServerTransportClass::WorthNativeInProcess,
            WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
        WorthServerResponseTransform::worth_native(),
    );
    let compat_http = shape_success(
        &server,
        query_handoff_success(
            &server,
            WorthServerSurfaceFamily::CompatHttp,
            WorthServerTransportClass::CompatHttp,
            WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
        WorthServerResponseTransform::compat_http(),
    );

    assert_eq!(
        worth_native.canonical_digest(),
        compat_http.canonical_digest()
    );
    assert_eq!(worth_native.provenance(), compat_http.provenance());
    assert_eq!(
        worth_native.success().expect("success").receipt(),
        compat_http.success().expect("success").receipt()
    );
    assert_eq!(
        worth_native.success().expect("success").payload(),
        compat_http.success().expect("success").payload()
    );
    assert_ne!(worth_native.transform(), compat_http.transform());
}

#[test]
fn shape_preserves_canonical_mutation_success_envelope_truth_across_surface_families() {
    let server = test_server(
        TestWorkspaceProvider,
        WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
    );

    let worth_native = shape_success(
        &server,
        query_handoff_success(
            &server,
            WorthServerSurfaceFamily::WorthNative,
            WorthServerTransportClass::WorthNativeInProcess,
            WorthServerQueryHandoffOperation::query_mutation("users.rename"),
        ),
        WorthServerResponseTransform::worth_native(),
    );
    let compat_http = shape_success(
        &server,
        query_handoff_success(
            &server,
            WorthServerSurfaceFamily::CompatHttp,
            WorthServerTransportClass::CompatHttp,
            WorthServerQueryHandoffOperation::query_mutation("users.rename"),
        ),
        WorthServerResponseTransform::compat_http(),
    );

    assert_eq!(
        worth_native.canonical_digest(),
        compat_http.canonical_digest()
    );
    assert_eq!(worth_native.provenance(), compat_http.provenance());
    assert_eq!(
        worth_native.success().expect("success").payload(),
        compat_http.success().expect("success").payload()
    );
    assert_eq!(
        worth_native
            .success()
            .expect("success")
            .payload()
            .operation(),
        &WorthServerQueryHandoffOperation::query_mutation("users.rename")
    );
}

#[test]
fn shape_changes_success_diagnostics_richness_without_changing_semantic_truth() {
    let low_richness_server = test_server(
        TestWorkspaceProvider,
        WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
    );
    let high_richness_server = test_server_with_response_config(
        TestWorkspaceProvider,
        WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
        WorthServerResponseConfig::builder()
            .with_success_minimum_diagnostics_profile(DiagnosticRichnessProfile::Forensic)
            .build()
            .expect("response config should validate"),
    );

    let low_richness = shape_success(
        &low_richness_server,
        query_handoff_success(
            &low_richness_server,
            WorthServerSurfaceFamily::WorthNative,
            WorthServerTransportClass::WorthNativeInProcess,
            WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
        WorthServerResponseTransform::worth_native(),
    );
    let high_richness = shape_success(
        &high_richness_server,
        query_handoff_success(
            &high_richness_server,
            WorthServerSurfaceFamily::WorthNative,
            WorthServerTransportClass::WorthNativeInProcess,
            WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
        WorthServerResponseTransform::worth_native(),
    );

    assert_eq!(
        low_richness.canonical_digest(),
        high_richness.canonical_digest()
    );
    assert_eq!(low_richness.provenance(), high_richness.provenance());
    assert_eq!(
        low_richness.success().expect("success").receipt(),
        high_richness.success().expect("success").receipt()
    );
    assert_eq!(
        low_richness.success().expect("success").payload(),
        high_richness.success().expect("success").payload()
    );
    assert_eq!(
        low_richness.diagnostics_profile(),
        DiagnosticRichnessProfile::Standard
    );
    assert_eq!(
        high_richness.diagnostics_profile(),
        DiagnosticRichnessProfile::Forensic
    );
}

#[test]
fn operator_evidence_keeps_exact_success_counters_and_richness_policy_distinct() {
    let low_richness_server = test_server(
        TestWorkspaceProvider,
        WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
    );
    let high_richness_server = test_server_with_response_config(
        TestWorkspaceProvider,
        WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
        WorthServerResponseConfig::builder()
            .with_success_minimum_diagnostics_profile(DiagnosticRichnessProfile::Forensic)
            .build()
            .expect("response config should validate"),
    );

    let low_response = shape_success(
        &low_richness_server,
        query_handoff_success(
            &low_richness_server,
            WorthServerSurfaceFamily::WorthNative,
            WorthServerTransportClass::WorthNativeInProcess,
            WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
        WorthServerResponseTransform::worth_native(),
    );
    let high_response = shape_success(
        &high_richness_server,
        query_handoff_success(
            &high_richness_server,
            WorthServerSurfaceFamily::WorthNative,
            WorthServerTransportClass::WorthNativeInProcess,
            WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
        WorthServerResponseTransform::worth_native(),
    );

    let low_evidence = operator_evidence_record(&low_richness_server, low_response);
    let high_evidence = operator_evidence_record(&high_richness_server, high_response);

    assert_eq!(
        low_evidence.response_digest(),
        high_evidence.response_digest()
    );
    assert_eq!(
        low_evidence.classification(),
        high_evidence.classification()
    );
    assert_eq!(
        low_evidence
            .counter_receipt()
            .counter("response.success.count")
            .expect("success counter")
            .exact_value(),
        1
    );
    assert_eq!(
        low_evidence
            .counter_receipt()
            .counter("response.query_read_success.count")
            .expect("read success counter")
            .exact_value(),
        1
    );
    assert_eq!(
        low_evidence
            .counter_receipt()
            .counter("response.denial.count")
            .expect("denial counter")
            .exact_value(),
        0
    );
    assert_eq!(
        high_evidence
            .counter_receipt()
            .counter("response.success.count")
            .expect("success counter")
            .exact_value(),
        1
    );
    assert!(low_evidence
        .materialized_attachment_bundle()
        .support()
        .is_some());
    assert!(high_evidence
        .materialized_attachment_bundle()
        .support()
        .is_some());
}

#[test]
fn operator_evidence_preserves_support_truth_when_operational_minimal_elides_support_attachment() {
    let server = test_server_with_request_context_and_operator_evidence_config(
        TestWorkspaceProvider,
        WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
        worth_server::WorthServerRequestContextConfig::builder()
            .with_default_diagnostics_profile(DiagnosticRichnessProfile::OperationalMinimal)
            .build()
            .expect("request context config should validate"),
        WorthServerResponseConfig::builder()
            .with_success_minimum_diagnostics_profile(DiagnosticRichnessProfile::OperationalMinimal)
            .build()
            .expect("response config should validate"),
        WorthServerOperatorEvidenceConfig::builder()
            .with_minimum_diagnostics_profile(DiagnosticRichnessProfile::OperationalMinimal)
            .build()
            .expect("operator evidence config should validate"),
    );

    let response = shape_success(
        &server,
        query_handoff_success(
            &server,
            WorthServerSurfaceFamily::WorthNative,
            WorthServerTransportClass::WorthNativeInProcess,
            WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
        WorthServerResponseTransform::worth_native(),
    );
    let evidence = operator_evidence_record(&server, response);

    assert_eq!(
        evidence.support_truth_kind(),
        FoundationalBoundaryEvidenceSupportTruthKind::EvidenceBundle
    );
    assert!(evidence
        .materialized_attachment_bundle()
        .support()
        .is_none());
}
