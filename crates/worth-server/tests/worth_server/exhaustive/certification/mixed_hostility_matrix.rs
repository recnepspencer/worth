pub(crate) use worth_server::{
    request_context::DiagnosticRichnessProfile, WorthServerResponseTransform,
    WorthServerSurfaceFamily, WorthServerTransportClass,
};

pub(crate) use super::certification_bundle::WorthServerCertificationField as Field;
pub(crate) use super::certification_counter_assertions::{
    assert_counter_exact, assert_counters_zero,
};
pub(crate) use super::certification_digest_assertions::{assert_equal_on, assert_not_equal_on};
pub(crate) use super::certification_fixture::{
    certification_server, durable_resume_denial_bundle, malformed_identity_bundle,
    middleware_preview_authorization_denial_bundle, preview_branch_denial_bundle,
    read_success_bundle, read_success_bundle_for_workspace,
};

#[test]
fn mixed_hostility_matrix_preserves_forced_entry_truth_and_typed_denial_boundaries() {
    let standard_server = certification_server(
        DiagnosticRichnessProfile::Standard,
        DiagnosticRichnessProfile::Standard,
        DiagnosticRichnessProfile::Standard,
    );
    let forensic_server = certification_server(
        DiagnosticRichnessProfile::Standard,
        DiagnosticRichnessProfile::Forensic,
        DiagnosticRichnessProfile::Forensic,
    );

    let control_lane = read_success_bundle(
        &standard_server,
        WorthServerSurfaceFamily::WorthNative,
        WorthServerTransportClass::WorthNativeInProcess,
        WorthServerResponseTransform::worth_native(),
    );
    let equivalent_lane = read_success_bundle(
        &standard_server,
        WorthServerSurfaceFamily::CompatHttp,
        WorthServerTransportClass::CompatHttp,
        WorthServerResponseTransform::compat_http(),
    );
    let diagnostics_lane = read_success_bundle(
        &forensic_server,
        WorthServerSurfaceFamily::WorthNative,
        WorthServerTransportClass::WorthNativeInProcess,
        WorthServerResponseTransform::worth_native(),
    );
    let tenant_divergence_lane = read_success_bundle_for_workspace(
        &standard_server,
        WorthServerSurfaceFamily::WorthNative,
        WorthServerTransportClass::WorthNativeInProcess,
        WorthServerResponseTransform::worth_native(),
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
            Field::RequestContextDigest,
            Field::ResponseDigest,
            Field::ProvenanceDigest,
            Field::CounterSnapshot,
        ],
    );
    assert_equal_on(
        &control_lane,
        &diagnostics_lane,
        &[
            Field::RequestContextDigest,
            Field::ResponseDigest,
            Field::ProvenanceDigest,
            Field::CounterSnapshot,
        ],
    );
    assert_counter_exact(&control_lane, "response.success.count", 1);
    assert_counter_exact(&equivalent_lane, "response.success.count", 1);
    assert_counter_exact(&diagnostics_lane, "response.success.count", 1);
    assert_not_equal_on(
        &control_lane,
        &tenant_divergence_lane,
        &[Field::RequestContextDigest, Field::ResponseDigest],
    );

    assert_not_equal_on(
        &control_lane,
        &malformed_identity_lane,
        &[Field::ResponseDigest, Field::FailureDigest],
    );
    assert_not_equal_on(
        &control_lane,
        &branch_hostility_lane,
        &[Field::ResponseDigest, Field::FailureDigest],
    );
    assert_not_equal_on(
        &control_lane,
        &authorization_pressure_lane,
        &[Field::ResponseDigest, Field::FailureDigest],
    );
    assert_not_equal_on(
        &control_lane,
        &unsupported_capability_lane,
        &[Field::ResponseDigest, Field::FailureDigest],
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
