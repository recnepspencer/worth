use worth_proof::TransitionReadiness;
use worth_server::{
    request_context::DiagnosticRichnessProfile,
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerBranchTarget, WorthServerConfig, WorthServerRequestContext,
    WorthServerRequestContextConfig, WorthServerRequestContextDenial,
    WorthServerRequestContextDenialCode, WorthServerRequestContextInput,
    WorthServerResolvedRequestContext, WorthServerSurfaceFamily, WorthServerTransportClass,
};

fn test_server(request_context: WorthServerRequestContextConfig) -> WorthServer {
    WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(request_context)
                .build()
                .expect("server config should validate"),
        )
        .register_surface(WorthNativeSurface::disabled())
        .register_surface(CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

fn input_builder() -> worth_server::WorthServerRequestContextInputBuilder {
    WorthServerRequestContextInput::builder()
        .with_surface_family(WorthServerSurfaceFamily::WorthNative)
        .with_transport_class(WorthServerTransportClass::WorthNativeInProcess)
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
}

fn denied(
    resolution: TransitionReadiness<
        WorthServerResolvedRequestContext,
        WorthServerRequestContextDenial,
        worth_server::WorthServerRequestContextDeferred,
        worth_server::WorthServerRequestContextStale,
        worth_server::WorthServerRequestContextRebindRequired,
        worth_server::WorthServerRequestContextFailure,
    >,
) -> WorthServerRequestContextDenial {
    match resolution {
        TransitionReadiness::Denied(denial) => denial,
        other => panic!("expected denial, got {other:?}"),
    }
}

fn ready_context(
    resolution: TransitionReadiness<
        WorthServerResolvedRequestContext,
        WorthServerRequestContextDenial,
        worth_server::WorthServerRequestContextDeferred,
        worth_server::WorthServerRequestContextStale,
        worth_server::WorthServerRequestContextRebindRequired,
        worth_server::WorthServerRequestContextFailure,
    >,
) -> WorthServerResolvedRequestContext {
    match resolution {
        TransitionReadiness::Ready(context) => context,
        other => panic!("expected ready context, got {other:?}"),
    }
}

#[test]
fn resolve_canonicalizes_trimmed_input_and_default_diagnostics() {
    let server = test_server(
        WorthServerRequestContextConfig::builder()
            .with_default_diagnostics_profile(DiagnosticRichnessProfile::Standard)
            .with_maximum_diagnostics_profile(DiagnosticRichnessProfile::Forensic)
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
            .with_diagnostics_profile(DiagnosticRichnessProfile::Standard)
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
        &WorthServerBranchTarget::Branch {
            branch_id: String::from("branch-9"),
        }
    );
    assert_eq!(
        trimmed.diagnostics_profile(),
        DiagnosticRichnessProfile::Standard
    );
}

#[test]
fn resolve_preserves_semantic_identity_across_surface_families_while_recording_boundary_shape() {
    let server = test_server(
        WorthServerRequestContextConfig::builder()
            .with_branch_targeting_enabled(true)
            .build()
            .expect("request context config should validate"),
    );

    let worth_native = server.request_contexts().resolve(
        input_builder()
            .with_surface_family(WorthServerSurfaceFamily::WorthNative)
            .with_transport_class(WorthServerTransportClass::WorthNativeInProcess)
            .with_branch_id("shared-branch")
            .build()
            .expect("input should validate"),
    );
    let compat_http = server.request_contexts().resolve(
        input_builder()
            .with_surface_family(WorthServerSurfaceFamily::CompatHttp)
            .with_transport_class(WorthServerTransportClass::CompatHttp)
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
        &WorthServerBranchTarget::Branch {
            branch_id: String::from("shared-branch"),
        }
    );
    assert_eq!(
        worth_native.surface_family(),
        WorthServerSurfaceFamily::WorthNative
    );
    assert_eq!(
        compat_http.surface_family(),
        WorthServerSurfaceFamily::CompatHttp
    );
    assert_eq!(
        worth_native.transport_class(),
        WorthServerTransportClass::WorthNativeInProcess
    );
    assert_eq!(
        compat_http.transport_class(),
        WorthServerTransportClass::CompatHttp
    );
}

#[test]
fn resolve_denies_incompatible_surface_transport_binding_before_context_construction() {
    let server = test_server(
        WorthServerRequestContextConfig::builder()
            .build()
            .expect("request context config should validate"),
    );

    let denial = denied(
        server.request_contexts().resolve(
            input_builder()
                .with_surface_family(WorthServerSurfaceFamily::WorthNative)
                .with_transport_class(WorthServerTransportClass::CompatHttp)
                .build()
                .expect("input should validate"),
        ),
    );

    assert_eq!(
        denial.code(),
        WorthServerRequestContextDenialCode::IncompatibleSurfaceTransportBinding
    );
    assert_eq!(
        denial.detail(),
        "surface family WorthNative cannot resolve transport class CompatHttp"
    );
}

#[test]
fn resolve_denies_blank_principal_before_context_construction() {
    let server = test_server(
        WorthServerRequestContextConfig::builder()
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
        WorthServerRequestContextDenialCode::InvalidAuthenticatedPrincipal
    );
    assert_eq!(
        denial.detail(),
        "authenticated principal id must not be empty"
    );
}

#[test]
fn resolve_denies_blank_workspace_target_before_context_construction() {
    let server = test_server(
        WorthServerRequestContextConfig::builder()
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
        WorthServerRequestContextDenialCode::InvalidWorkspaceTarget
    );
    assert_eq!(denial.detail(), "workspace id must not be empty");
}

#[test]
fn resolve_denies_disabled_branch_targeting_with_exact_code_and_detail() {
    let server = test_server(
        WorthServerRequestContextConfig::builder()
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
        WorthServerRequestContextDenialCode::BranchTargetingDisabled
    );
    assert_eq!(
        denial.detail(),
        "branch targeting is disabled by server configuration"
    );
}

#[test]
fn resolve_denies_blank_branch_identifier_before_context_construction() {
    let server = test_server(
        WorthServerRequestContextConfig::builder()
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
        WorthServerRequestContextDenialCode::InvalidBranchTarget
    );
    assert_eq!(denial.detail(), "branch id must not be empty");
}

#[test]
fn resolve_denies_preview_targeting_and_overspecified_diagnostics_strictly() {
    let preview_denial_server = test_server(
        WorthServerRequestContextConfig::builder()
            .with_preview_targeting_enabled(false)
            .build()
            .expect("request context config should validate"),
    );
    let diagnostics_denial_server = test_server(
        WorthServerRequestContextConfig::builder()
            .with_default_diagnostics_profile(DiagnosticRichnessProfile::Standard)
            .with_maximum_diagnostics_profile(DiagnosticRichnessProfile::Standard)
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
                .with_diagnostics_profile(DiagnosticRichnessProfile::Forensic)
                .build()
                .expect("input should validate"),
        ),
    );

    assert_eq!(
        preview_denial.code(),
        WorthServerRequestContextDenialCode::PreviewTargetingDisabled
    );
    assert_eq!(
        preview_denial.detail(),
        "preview targeting is disabled by server configuration"
    );
    assert_eq!(
        diagnostics_denial.code(),
        WorthServerRequestContextDenialCode::DiagnosticsProfileExceedsMaximum
    );
    assert_eq!(
        diagnostics_denial.detail(),
        "requested diagnostics profile Forensic exceeds configured maximum Standard"
    );
}
