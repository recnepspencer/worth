pub(crate) use worth_foundational::facade::FoundationalBoundaryEvidenceSupportTruthKind;

pub(crate) use {
    crate::certification_suite::ordinary_context::certification_counter_assertions::assert_counter_exact,
    crate::certification_suite::ordinary_context::certification_counter_assertions::assert_counters_zero,
};
pub(crate) use {
    crate::certification_suite::ordinary_context::certification_fixture::certification_server,
    crate::certification_suite::ordinary_context::certification_fixture::durable_resume_denial_bundle,
    crate::certification_suite::ordinary_context::certification_fixture::malformed_identity_bundle,
};

#[test]
fn structural_sabotage_suite_proves_non_bypass_and_forbidden_zero_artifacts() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-server/assets/ui/construction/private_middleware_admission_constructor.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-server/assets/ui/construction/query_handoff/private_prepared_query_handoff_constructor.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-server/assets/ui/construction/query_handoff/private_query_handoff_constructor.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-server/assets/ui/construction/response/private_evidence_record_constructor.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-server/assets/ui/cross_family/compat_http_cannot_import_worth_native_internal.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-testkit-server/assets/ui/cross_family/worth_native_cannot_import_compat_http_internal.rs")).expect("prebuilt compiler denial");

    let server = certification_server(
        worth_server::request_context::DiagnosticRichnessProfile::OperationalMinimal,
        worth_server::request_context::DiagnosticRichnessProfile::OperationalMinimal,
        worth_server::request_context::DiagnosticRichnessProfile::OperationalMinimal,
    );
    let malformed_identity_lane = malformed_identity_bundle(&server);
    let unsupported_capability_lane = durable_resume_denial_bundle(&server);

    assert_eq!(
        malformed_identity_lane.support_truth_kind(),
        worth_foundational::facade::FoundationalBoundaryEvidenceSupportTruthKind::DegradedRecoveryReport
    );
    assert!(!malformed_identity_lane.support_attachment_present());
    assert_eq!(
        unsupported_capability_lane.support_truth_kind(),
        worth_foundational::facade::FoundationalBoundaryEvidenceSupportTruthKind::DegradedRecoveryReport
    );
    assert!(!unsupported_capability_lane.support_attachment_present());

    assert_counter_exact(
        &malformed_identity_lane,
        "response.request_context_denial.count",
        1,
    );
    assert_counters_zero(
        &malformed_identity_lane,
        &[
            "response.success.count",
            "response.query_read_success.count",
            "response.query_mutation_success.count",
            "response.downstream_delivery_success.count",
            "response.unsupported_capability.count",
        ],
    );

    assert_counter_exact(
        &unsupported_capability_lane,
        "response.query_handoff_denial.count",
        1,
    );
    assert_counter_exact(
        &unsupported_capability_lane,
        "response.unsupported_capability.count",
        1,
    );
    assert_counters_zero(
        &unsupported_capability_lane,
        &[
            "response.success.count",
            "response.query_read_success.count",
            "response.query_mutation_success.count",
            "response.downstream_delivery_success.count",
        ],
    );
}
