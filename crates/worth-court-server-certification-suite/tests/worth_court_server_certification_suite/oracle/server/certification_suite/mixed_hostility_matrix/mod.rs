pub(crate) use {worth_server::request_context::DiagnosticRichnessProfile, worth_server::WorthServerResponseTransform, worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass};

pub(crate) use {crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField as Field};
pub(crate) use {crate::certification_suite::ordinary_context::certification_counter_assertions::assert_counter_exact, crate::certification_suite::ordinary_context::certification_counter_assertions::assert_counters_zero};
pub(crate) use {crate::certification_suite::ordinary_context::certification_digest_assertions::assert_equal_on, crate::certification_suite::ordinary_context::certification_digest_assertions::assert_not_equal_on};
pub(crate) use {crate::certification_suite::ordinary_context::certification_fixture::certification_server, crate::certification_suite::ordinary_context::certification_fixture::durable_resume_denial_bundle, crate::certification_suite::ordinary_context::certification_fixture::malformed_identity_bundle, crate::certification_suite::ordinary_context::certification_fixture::middleware_preview_authorization_denial_bundle, crate::certification_suite::ordinary_context::certification_fixture::preview_branch_denial_bundle, crate::certification_suite::ordinary_context::certification_fixture::read_success_bundle, crate::certification_suite::ordinary_context::certification_fixture::read_success_bundle_for_workspace};

#[test]
fn mixed_hostility_matrix_preserves_forced_entry_truth_and_typed_denial_boundaries() {
    let standard_server = certification_server(
        worth_server::request_context::DiagnosticRichnessProfile::Standard,
        worth_server::request_context::DiagnosticRichnessProfile::Standard,
        worth_server::request_context::DiagnosticRichnessProfile::Standard,
    );
    let forensic_server = certification_server(
        worth_server::request_context::DiagnosticRichnessProfile::Standard,
        worth_server::request_context::DiagnosticRichnessProfile::Forensic,
        worth_server::request_context::DiagnosticRichnessProfile::Forensic,
    );

    let control_lane = read_success_bundle(
        &standard_server,
        worth_server::WorthServerSurfaceFamily::WorthNative,
        worth_server::WorthServerTransportClass::WorthNativeInProcess,
        worth_server::WorthServerResponseTransform::worth_native(),
    );
    let equivalent_lane = read_success_bundle(
        &standard_server,
        worth_server::WorthServerSurfaceFamily::CompatHttp,
        worth_server::WorthServerTransportClass::CompatHttp,
        worth_server::WorthServerResponseTransform::compat_http(),
    );
    let diagnostics_lane = read_success_bundle(
        &forensic_server,
        worth_server::WorthServerSurfaceFamily::WorthNative,
        worth_server::WorthServerTransportClass::WorthNativeInProcess,
        worth_server::WorthServerResponseTransform::worth_native(),
    );
    let tenant_divergence_lane = read_success_bundle_for_workspace(
        &standard_server,
        worth_server::WorthServerSurfaceFamily::WorthNative,
        worth_server::WorthServerTransportClass::WorthNativeInProcess,
        worth_server::WorthServerResponseTransform::worth_native(),
        "tenant-b",
        "workspace-42",
    );
    let malformed_identity_lane = malformed_identity_bundle(&standard_server);
    let branch_hostility_lane = preview_branch_denial_bundle(&standard_server);
    let authorization_pressure_lane = middleware_preview_authorization_denial_bundle();
    let unsupported_capability_lane = durable_resume_denial_bundle(&standard_server);

    assert_equal_on(
        &control_lane,
        &equivalent_lane,
        &[
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::RequestContextDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ProvenanceDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::CounterSnapshot,
        ],
    );
    assert_equal_on(
        &control_lane,
        &diagnostics_lane,
        &[
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::RequestContextDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ProvenanceDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::CounterSnapshot,
        ],
    );
    assert_counter_exact(&control_lane, "response.success.count", 1);
    assert_counter_exact(&equivalent_lane, "response.success.count", 1);
    assert_counter_exact(&diagnostics_lane, "response.success.count", 1);
    assert_not_equal_on(
        &control_lane,
        &tenant_divergence_lane,
        &[crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::RequestContextDigest, crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest],
    );

    assert_not_equal_on(
        &control_lane,
        &malformed_identity_lane,
        &[crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest, crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::FailureDigest],
    );
    assert_not_equal_on(
        &control_lane,
        &branch_hostility_lane,
        &[crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest, crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::FailureDigest],
    );
    assert_not_equal_on(
        &control_lane,
        &authorization_pressure_lane,
        &[crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest, crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::FailureDigest],
    );
    assert_not_equal_on(
        &control_lane,
        &unsupported_capability_lane,
        &[crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest, crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::FailureDigest],
    );

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
            "response.unsupported_capability.count",
        ],
    );

    assert_counter_exact(
        &branch_hostility_lane,
        "response.request_context_denial.count",
        1,
    );
    assert_counters_zero(
        &branch_hostility_lane,
        &[
            "response.success.count",
            "response.query_read_success.count",
            "response.unsupported_capability.count",
        ],
    );

    assert_counter_exact(
        &authorization_pressure_lane,
        "response.middleware_denial.count",
        1,
    );
    assert_counters_zero(
        &authorization_pressure_lane,
        &[
            "response.success.count",
            "response.query_read_success.count",
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
        ],
    );
}
