#[allow(unused_imports)]
use crate::request_context_resolution::ordinary_context::*;
use worth_server::WorthServerRequestContext;

#[test]
fn resolve_canonicalizes_trimmed_input_and_default_diagnostics() {
    let server = test_server(
        worth_server::WorthServerRequestContextConfig::builder()
            .with_default_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::Standard,
            )
            .with_maximum_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::Forensic,
            )
            .build()
            .expect("request context config should validate"),
    );

    let trimmed = server.request_contexts().resolve(
        input_builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .build()
            .expect("input should validate"),
    );
    let padded = server.request_contexts().resolve(
        input_builder()
            .with_authenticated_principal_id(" principal-7 ")
            .with_tenant_id(" tenant-a ")
            .with_workspace_id(" workspace-42 ")
            .with_branch_id(" branch-9 ")
            .with_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::Standard,
            )
            .build()
            .expect("input should validate"),
    );

    let trimmed = ready_context(trimmed);
    let padded = ready_context(padded);

    assert_eq!(trimmed, padded);
    let trimmed = trimmed.request_context();
    assert_eq!(
        trimmed.authenticated_principal().principal_id(),
        "principal-7"
    );
    assert_eq!(trimmed.workspace_target().tenant_id(), "tenant-a");
    assert_eq!(trimmed.workspace_target().workspace_id(), "workspace-42");
    assert_eq!(
        trimmed.branch_target(),
        &worth_server::WorthServerBranchTarget::Branch {
            branch_id: String::from("branch-9"),
        }
    );
    assert_eq!(
        trimmed.diagnostics_profile(),
        worth_server::request_context::DiagnosticRichnessProfile::Standard
    );
}

#[test]
fn resolve_preserves_semantic_identity_across_surface_families_while_recording_boundary_shape() {
    let server = test_server(
        worth_server::WorthServerRequestContextConfig::builder()
            .with_branch_targeting_enabled(true)
            .build()
            .expect("request context config should validate"),
    );

    let worth_native = server.request_contexts().resolve(
        input_builder()
            .with_surface_family(worth_server::WorthServerSurfaceFamily::WorthNative)
            .with_transport_class(worth_server::WorthServerTransportClass::WorthNativeInProcess)
            .with_branch_id("shared-branch")
            .build()
            .expect("input should validate"),
    );
    let compat_http = server.request_contexts().resolve(
        input_builder()
            .with_surface_family(worth_server::WorthServerSurfaceFamily::CompatHttp)
            .with_transport_class(worth_server::WorthServerTransportClass::CompatHttp)
            .with_branch_id("shared-branch")
            .build()
            .expect("input should validate"),
    );

    let worth_native = ready_context(worth_native);
    let compat_http = ready_context(compat_http);

    assert_eq!(
        worth_native.request_context(),
        compat_http.request_context()
    );
    let semantic_context: &WorthServerRequestContext = worth_native.request_context();
    assert_eq!(
        semantic_context.branch_target(),
        &worth_server::WorthServerBranchTarget::Branch {
            branch_id: String::from("shared-branch"),
        }
    );
    assert_eq!(
        worth_native.surface_family(),
        worth_server::WorthServerSurfaceFamily::WorthNative
    );
    assert_eq!(
        compat_http.surface_family(),
        worth_server::WorthServerSurfaceFamily::CompatHttp
    );
    assert_eq!(
        worth_native.transport_class(),
        worth_server::WorthServerTransportClass::WorthNativeInProcess
    );
    assert_eq!(
        compat_http.transport_class(),
        worth_server::WorthServerTransportClass::CompatHttp
    );
}

#[test]
fn resolve_denies_incompatible_surface_transport_binding_before_context_construction() {
    let server = test_server(
        worth_server::WorthServerRequestContextConfig::builder()
            .build()
            .expect("request context config should validate"),
    );

    let denial = denied(
        server.request_contexts().resolve(
            input_builder()
                .with_surface_family(worth_server::WorthServerSurfaceFamily::WorthNative)
                .with_transport_class(worth_server::WorthServerTransportClass::CompatHttp)
                .build()
                .expect("input should validate"),
        ),
    );

    assert_eq!(
        denial.code(),
        worth_server::WorthServerRequestContextDenialCode::IncompatibleSurfaceTransportBinding
    );
    assert_eq!(
        denial.detail(),
        "surface family WorthNative cannot resolve transport class CompatHttp"
    );
}

#[test]
fn resolve_denies_blank_principal_before_context_construction() {
    let server = test_server(
        worth_server::WorthServerRequestContextConfig::builder()
            .build()
            .expect("request context config should validate"),
    );

    let denial = denied(
        server.request_contexts().resolve(
            input_builder()
                .with_authenticated_principal_id("   ")
                .build()
                .expect("input should validate"),
        ),
    );

    assert_eq!(
        denial.code(),
        worth_server::WorthServerRequestContextDenialCode::InvalidAuthenticatedPrincipal
    );
    assert_eq!(
        denial.detail(),
        "authenticated principal id must not be empty"
    );
}

#[test]
fn resolve_denies_blank_workspace_target_before_context_construction() {
    let server = test_server(
        worth_server::WorthServerRequestContextConfig::builder()
            .build()
            .expect("request context config should validate"),
    );

    let denial = denied(
        server.request_contexts().resolve(
            input_builder()
                .with_tenant_id("tenant-a")
                .with_workspace_id("   ")
                .build()
                .expect("input should validate"),
        ),
    );

    assert_eq!(
        denial.code(),
        worth_server::WorthServerRequestContextDenialCode::InvalidWorkspaceTarget
    );
    assert_eq!(denial.detail(), "workspace id must not be empty");
}

#[test]
fn resolve_denies_disabled_branch_targeting_with_exact_code_and_detail() {
    let server = test_server(
        worth_server::WorthServerRequestContextConfig::builder()
            .with_branch_targeting_enabled(false)
            .build()
            .expect("request context config should validate"),
    );

    let denial = denied(
        server.request_contexts().resolve(
            input_builder()
                .with_branch_id("branch-1")
                .build()
                .expect("input should validate"),
        ),
    );

    assert_eq!(
        denial.code(),
        worth_server::WorthServerRequestContextDenialCode::BranchTargetingDisabled
    );
    assert_eq!(
        denial.detail(),
        "branch targeting is disabled by server configuration"
    );
}

#[test]
fn resolve_denies_blank_branch_identifier_before_context_construction() {
    let server = test_server(
        worth_server::WorthServerRequestContextConfig::builder()
            .with_branch_targeting_enabled(true)
            .build()
            .expect("request context config should validate"),
    );

    let denial = denied(
        server.request_contexts().resolve(
            input_builder()
                .with_branch_id("   ")
                .build()
                .expect("input should validate"),
        ),
    );

    assert_eq!(
        denial.code(),
        worth_server::WorthServerRequestContextDenialCode::InvalidBranchTarget
    );
    assert_eq!(denial.detail(), "branch id must not be empty");
}

#[test]
fn resolve_denies_preview_targeting_and_overspecified_diagnostics_strictly() {
    let preview_denial_server = test_server(
        worth_server::WorthServerRequestContextConfig::builder()
            .with_preview_targeting_enabled(false)
            .build()
            .expect("request context config should validate"),
    );
    let diagnostics_denial_server = test_server(
        worth_server::WorthServerRequestContextConfig::builder()
            .with_default_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::Standard,
            )
            .with_maximum_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::Standard,
            )
            .build()
            .expect("request context config should validate"),
    );

    let preview_denial = denied(
        preview_denial_server.request_contexts().resolve(
            input_builder()
                .with_preview_id("preview-unsafe")
                .build()
                .expect("input should validate"),
        ),
    );
    let diagnostics_denial = denied(
        diagnostics_denial_server.request_contexts().resolve(
            input_builder()
                .with_diagnostics_profile(
                    worth_server::request_context::DiagnosticRichnessProfile::Forensic,
                )
                .build()
                .expect("input should validate"),
        ),
    );

    assert_eq!(
        preview_denial.code(),
        worth_server::WorthServerRequestContextDenialCode::PreviewTargetingDisabled
    );
    assert_eq!(
        preview_denial.detail(),
        "preview targeting is disabled by server configuration"
    );
    assert_eq!(
        diagnostics_denial.code(),
        worth_server::WorthServerRequestContextDenialCode::DiagnosticsProfileExceedsMaximum
    );
    assert_eq!(
        diagnostics_denial.detail(),
        "requested diagnostics profile Forensic exceeds configured maximum Standard"
    );
}
