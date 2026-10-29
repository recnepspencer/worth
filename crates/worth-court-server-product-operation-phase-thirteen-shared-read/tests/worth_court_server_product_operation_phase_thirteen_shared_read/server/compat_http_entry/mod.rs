

use {worth_proof::TransitionOutcome};
use {worth_server::surfaces::CompatHttpSurface, worth_server::WorthServer, worth_server::WorthServerCompatHttpRouteFamily, worth_server::WorthServerCompatibilityDenialCode, worth_server::WorthServerCompatibilityRequestInput, worth_server::WorthServerConfig};

use {crate::compat_http_entry::ordinary_context::compat_http_entry_runtime::compat_http_entry_test_server, crate::compat_http_entry::ordinary_context::compat_http_entry_runtime::compat_http_prepared_request, crate::compat_http_entry::ordinary_context::compat_http_entry_runtime::ready_context, crate::compat_http_entry::ordinary_context::compat_http_entry_runtime::worth_native_request_context_input};

#[test]
fn compat_http_surface_root_exposes_phase_one_route_families() {
    let compat_http = compat_http_entry_test_server().surfaces().compat_http();

    assert!(compat_http.capabilities().is_registered());
    assert!(compat_http
        .route_families()
        .contains(worth_server::WorthServerCompatHttpRouteFamily::Read));
    assert!(compat_http
        .route_families()
        .contains(worth_server::WorthServerCompatHttpRouteFamily::Query));
    assert!(compat_http
        .route_families()
        .contains(worth_server::WorthServerCompatHttpRouteFamily::Mutation));
    assert!(compat_http
        .route_families()
        .contains(worth_server::WorthServerCompatHttpRouteFamily::Streaming));
    assert!(compat_http
        .route_families()
        .contains(worth_server::WorthServerCompatHttpRouteFamily::Upload));
    assert!(compat_http
        .route_families()
        .contains(worth_server::WorthServerCompatHttpRouteFamily::Download));
    assert!(compat_http
        .route_families()
        .contains(worth_server::WorthServerCompatHttpRouteFamily::Preflight));
}

#[test]
fn compat_http_request_preserves_request_context_semantics_for_equivalent_identity() {
    let server = compat_http_entry_test_server();
    let worth_native_context = ready_context(
        server
            .request_contexts()
            .resolve(worth_native_request_context_input()),
    );

    let prepared_request = compat_http_prepared_request(
        &server,
        worth_server::WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id(" principal-7 ")
            .with_tenant_id(" tenant-a ")
            .with_workspace_id(" workspace-42 ")
            .with_branch_id(" branch-9 ")
            .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/v1/users/123")
            .with_query_pair("view", "detail")
            .with_header("accept", "application/json")
            .build()
            .expect("compat input should validate"),
    );

    assert_eq!(
        worth_native_context.request_context(),
        prepared_request.admission().request_context()
    );
    assert_eq!(
        prepared_request.request_contract().route_family(),
        worth_server::WorthServerCompatHttpRouteFamily::Read
    );
}

#[test]
fn compat_http_denies_unregistered_route_family_before_request_context_resolution() {
    let server = worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .build()
                .expect("server config should validate"),
        )
        .register_surface(worth_server::surfaces::CompatHttpSurface::enabled(
            worth_server::WorthServerCompatHttpRouteFamilies::new([
                worth_server::WorthServerCompatHttpRouteFamily::Read,
            ]),
        ))
        .build()
        .expect("server should build");

    let denial: worth_server::WorthServerCompatibilityDenial = match server.compat_http().request(
        worth_server::WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Mutation)
            .with_method("POST")
            .with_path("/v1/users/123")
            .build()
            .expect("compat input should validate"),
    ) {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied compat request, got {other:?}"),
    };

    assert_eq!(
        denial.code(),
        worth_server::WorthServerCompatibilityDenialCode::UnsupportedRouteFamily
    );
}

#[test]
fn compat_http_head_read_preserves_request_context_identity_against_get() {
    let server = compat_http_entry_test_server();
    let get_request = compat_http_prepared_request(
        &server,
        worth_server::WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/v1/users/123")
            .build()
            .expect("GET input should validate"),
    );
    let head_request = compat_http_prepared_request(
        &server,
        worth_server::WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Read)
            .with_method("HEAD")
            .with_path("/v1/users/123")
            .build()
            .expect("HEAD input should validate"),
    );

    assert_eq!(
        get_request.request_context_digest(),
        head_request.request_context_digest()
    );
    assert_eq!(
        get_request.request_contract().route_family(),
        head_request.request_contract().route_family()
    );
}
