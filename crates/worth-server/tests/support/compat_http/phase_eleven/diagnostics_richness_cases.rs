use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::{
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerConfig, WorthServerMiddlewareConfig, WorthServerOperatorEvidenceConfig,
    WorthServerQueryHandoffConfig, WorthServerRequestContextConfig,
};

use crate::shared_support_compat_http_phase_ten_runtime::{
    build_phase_ten_server_with_workspace_provider, compat_read_execution_input,
    compat_read_success,
};
use crate::shared_support_query_handoff_runtime::ProfiledTestWorkspaceProvider;

#[test]
fn compat_http_phase_eleven_reduced_diagnostics_trim_detail_without_changing_counter_or_policy_truth(
) {
    let server =
        build_phase_ten_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));

    let forensic = compat_read_success(server.compat_http().read(compat_read_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-11",
        "users.profile",
        DiagnosticRichnessProfile::Forensic,
    )));
    let minimal = compat_read_success(server.compat_http().read(compat_read_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-11",
        "users.profile",
        DiagnosticRichnessProfile::OperationalMinimal,
    )));

    let forensic_certification = forensic.certification_bundle();
    let minimal_certification = minimal.certification_bundle();

    assert_eq!(
        forensic_certification.support_posture_label(),
        minimal_certification.support_posture_label()
    );
    assert_eq!(
        forensic_certification.policy_digest(),
        minimal_certification.policy_digest()
    );
    assert_eq!(
        forensic_certification.provenance_digest(),
        minimal_certification.provenance_digest()
    );
    assert_eq!(
        forensic_certification
            .external_counters()
            .canonical_digest(),
        minimal_certification.external_counters().canonical_digest()
    );
    assert_eq!(
        forensic_certification
            .operator_evidence_record()
            .classification_label(),
        minimal_certification
            .operator_evidence_record()
            .classification_label()
    );
    assert_eq!(
        forensic_certification
            .operator_evidence_record()
            .support_truth_kind(),
        minimal_certification
            .operator_evidence_record()
            .support_truth_kind()
    );
    assert_ne!(
        forensic_certification
            .operator_evidence_record()
            .diagnostics_profile(),
        minimal_certification
            .operator_evidence_record()
            .diagnostics_profile()
    );
}

#[test]
fn compat_http_phase_eleven_certification_bundle_honors_server_operator_evidence_minimum_profile() {
    let server = WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    WorthServerRequestContextConfig::builder()
                        .with_preview_targeting_enabled(true)
                        .with_default_diagnostics_profile(DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("request context config should validate"),
                )
                .with_middleware_config(
                    WorthServerMiddlewareConfig::builder()
                        .with_compat_http_maximum_diagnostics_profile(
                            DiagnosticRichnessProfile::Forensic,
                        )
                        .build()
                        .expect("middleware config should validate"),
                )
                .with_query_handoff_config(
                    WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(ProfiledTestWorkspaceProvider::new(
                            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
                        ))
                        .build()
                        .expect("query handoff config should validate"),
                )
                .with_operator_evidence_config(
                    WorthServerOperatorEvidenceConfig::builder()
                        .with_minimum_diagnostics_profile(DiagnosticRichnessProfile::Forensic)
                        .build()
                        .expect("operator evidence config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(WorthNativeSurface::enabled())
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("server should build");

    let read = compat_read_success(server.compat_http().read(compat_read_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-11",
        "users.profile",
        DiagnosticRichnessProfile::OperationalMinimal,
    )));

    assert_eq!(
        read.certification_bundle()
            .operator_evidence_record()
            .diagnostics_profile(),
        DiagnosticRichnessProfile::Forensic
    );
}
