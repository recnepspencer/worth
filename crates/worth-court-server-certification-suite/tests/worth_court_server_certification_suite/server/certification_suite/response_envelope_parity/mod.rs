pub(crate) use worth_foundational::facade::FoundationalBoundaryEvidenceSupportTruthKind;
pub(crate) use {
    worth_server::request_context::DiagnosticRichnessProfile,
    worth_server::WorthServerMiddlewareConfig, worth_server::WorthServerOperatorEvidenceConfig,
    worth_server::WorthServerQueryHandoffOperation, worth_server::WorthServerResponseConfig,
    worth_server::WorthServerResponseTransform, worth_server::WorthServerSurfaceFamily,
    worth_server::WorthServerTransportClass,
};

pub(crate) use worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::TestWorkspaceProvider;
pub(crate) use {
    worth_testkit_server::shared_support_response_fixture::operator_evidence_record,
    worth_testkit_server::shared_support_response_fixture::query_handoff_success,
    worth_testkit_server::shared_support_response_fixture::shape_success,
    worth_testkit_server::shared_support_response_fixture::test_server,
    worth_testkit_server::shared_support_response_fixture::test_server_with_request_context_and_operator_evidence_config,
    worth_testkit_server::shared_support_response_fixture::test_server_with_response_config,
};

#[test]
fn shape_preserves_canonical_success_envelope_truth_across_surface_families() {
    let server = test_server(
        TestWorkspaceProvider,
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
    );

    let worth_native = shape_success(
        &server,
        query_handoff_success(
            &server,
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerTransportClass::WorthNativeInProcess,
            worth_server::WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
        worth_server::WorthServerResponseTransform::worth_native(),
    );
    let compat_http = shape_success(
        &server,
        query_handoff_success(
            &server,
            worth_server::WorthServerSurfaceFamily::CompatHttp,
            worth_server::WorthServerTransportClass::CompatHttp,
            worth_server::WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
        worth_server::WorthServerResponseTransform::compat_http(),
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
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
    );

    let worth_native = shape_success(
        &server,
        query_handoff_success(
            &server,
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerTransportClass::WorthNativeInProcess,
            worth_server::WorthServerQueryHandoffOperation::query_mutation("users.rename"),
        ),
        worth_server::WorthServerResponseTransform::worth_native(),
    );
    let compat_http = shape_success(
        &server,
        query_handoff_success(
            &server,
            worth_server::WorthServerSurfaceFamily::CompatHttp,
            worth_server::WorthServerTransportClass::CompatHttp,
            worth_server::WorthServerQueryHandoffOperation::query_mutation("users.rename"),
        ),
        worth_server::WorthServerResponseTransform::compat_http(),
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
        &worth_server::WorthServerQueryHandoffOperation::query_mutation("users.rename")
    );
}

#[test]
fn shape_changes_success_diagnostics_richness_without_changing_semantic_truth() {
    let low_richness_server = test_server(
        TestWorkspaceProvider,
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
    );
    let high_richness_server = test_server_with_response_config(
        TestWorkspaceProvider,
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
        worth_server::WorthServerResponseConfig::builder()
            .with_success_minimum_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::Forensic,
            )
            .build()
            .expect("response config should validate"),
    );

    let low_richness = shape_success(
        &low_richness_server,
        query_handoff_success(
            &low_richness_server,
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerTransportClass::WorthNativeInProcess,
            worth_server::WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
        worth_server::WorthServerResponseTransform::worth_native(),
    );
    let high_richness = shape_success(
        &high_richness_server,
        query_handoff_success(
            &high_richness_server,
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerTransportClass::WorthNativeInProcess,
            worth_server::WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
        worth_server::WorthServerResponseTransform::worth_native(),
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
        worth_server::request_context::DiagnosticRichnessProfile::Standard
    );
    assert_eq!(
        high_richness.diagnostics_profile(),
        worth_server::request_context::DiagnosticRichnessProfile::Forensic
    );
}

#[test]
fn operator_evidence_keeps_exact_success_counters_and_richness_policy_distinct() {
    let low_richness_server = test_server(
        TestWorkspaceProvider,
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
    );
    let high_richness_server = test_server_with_response_config(
        TestWorkspaceProvider,
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
        worth_server::WorthServerResponseConfig::builder()
            .with_success_minimum_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::Forensic,
            )
            .build()
            .expect("response config should validate"),
    );

    let low_response = shape_success(
        &low_richness_server,
        query_handoff_success(
            &low_richness_server,
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerTransportClass::WorthNativeInProcess,
            worth_server::WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
        worth_server::WorthServerResponseTransform::worth_native(),
    );
    let high_response = shape_success(
        &high_richness_server,
        query_handoff_success(
            &high_richness_server,
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerTransportClass::WorthNativeInProcess,
            worth_server::WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
        worth_server::WorthServerResponseTransform::worth_native(),
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
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
        worth_server::WorthServerRequestContextConfig::builder()
            .with_default_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::OperationalMinimal,
            )
            .build()
            .expect("request context config should validate"),
        worth_server::WorthServerResponseConfig::builder()
            .with_success_minimum_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::OperationalMinimal,
            )
            .build()
            .expect("response config should validate"),
        worth_server::WorthServerOperatorEvidenceConfig::builder()
            .with_minimum_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::OperationalMinimal,
            )
            .build()
            .expect("operator evidence config should validate"),
    );

    let response = shape_success(
        &server,
        query_handoff_success(
            &server,
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerTransportClass::WorthNativeInProcess,
            worth_server::WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
        worth_server::WorthServerResponseTransform::worth_native(),
    );
    let evidence = operator_evidence_record(&server, response);

    assert_eq!(
        evidence.support_truth_kind(),
        worth_foundational::facade::FoundationalBoundaryEvidenceSupportTruthKind::EvidenceBundle
    );
    assert!(evidence
        .materialized_attachment_bundle()
        .support()
        .is_none());
}
