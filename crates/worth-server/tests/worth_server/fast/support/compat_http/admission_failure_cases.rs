use worth_server::{
    WorthServerCompatHttpRouteFamily, WorthServerCompatibilityDenialCode,
    WorthServerCompatibilityRequestInput,
};

use super::compat_http_admission_runtime::{compat_http_admission_test_server, compat_http_denial};

#[test]
fn compat_http_malformed_request_contract_denies_before_request_context_resolution() {
    let denial = compat_http_denial(
        &compat_http_admission_test_server(),
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_preview_id("preview-9")
            .with_route_family(WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("users/123")
            .build()
            .expect("input should validate"),
    );
    assert_eq!(
        denial.code(),
        WorthServerCompatibilityDenialCode::InvalidPath
    );
}

#[test]
fn compat_http_denies_blank_query_key_before_request_context_resolution() {
    let denial = compat_http_denial(
        &compat_http_admission_test_server(),
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_preview_id("preview-9")
            .with_route_family(WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/v1/users/123")
            .with_query_pair("   ", "value")
            .build()
            .expect("input should validate"),
    );
    assert_eq!(
        denial.code(),
        WorthServerCompatibilityDenialCode::InvalidQueryPair
    );
}

#[test]
fn compat_http_denies_unsupported_accept_before_request_context_resolution() {
    let denial = compat_http_denial(
        &compat_http_admission_test_server(),
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_preview_id("preview-9")
            .with_route_family(WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/v1/users/123")
            .with_header("accept", "text/plain")
            .build()
            .expect("input should validate"),
    );
    assert_eq!(
        denial.code(),
        WorthServerCompatibilityDenialCode::UnsupportedRepresentation
    );
}
