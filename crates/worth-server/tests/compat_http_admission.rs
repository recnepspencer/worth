#[path = "support/compat_http/admission_failure_cases.rs"]
mod compat_http_admission_failure_cases;
#[path = "support/compat_http/admission_runtime.rs"]
mod compat_http_admission_runtime;

use worth_server::{
    request_context::DiagnosticRichnessProfile, WorthServerCompatHttpRouteFamily,
    WorthServerCompatibilityDenialCode, WorthServerCompatibilityRequestInput,
};

use compat_http_admission_runtime::{
    compat_http_admission_test_server, compat_http_denial, compat_http_request,
};

#[test]
fn compat_http_canonicalizes_equivalent_requests_to_the_same_digest() {
    let server = compat_http_admission_test_server();

    let left = compat_http_request(
        &server,
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Read)
            .with_method("get")
            .with_path(" /v1/users/123 ")
            .with_query_pair("view", "detail")
            .with_query_pair("page", "1")
            .with_header("accept", "application/json")
            .with_header("x-Worth-api-version", "1")
            .build()
            .expect("left input should validate"),
    );
    let right = compat_http_request(
        &server,
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/v1/users/123")
            .with_query_pair("page", "1")
            .with_query_pair("view", "detail")
            .with_header("Accept", "application/json")
            .with_header("X-Worth-Api-Version", "1")
            .build()
            .expect("right input should validate"),
    );

    assert_eq!(
        left.request_contract().canonical_digest(),
        right.request_contract().canonical_digest()
    );
}

#[test]
fn compat_http_canonicalizes_body_content_type_and_diagnostics_in_request_contract_identity() {
    let server = compat_http_admission_test_server();

    let left = compat_http_request(
        &server,
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Upload)
            .with_method("POST")
            .with_path("/v1/uploads")
            .with_diagnostics_profile(DiagnosticRichnessProfile::Standard)
            .with_body_present(true)
            .with_body_content_type(" Application/Json ")
            .build()
            .expect("left input should validate"),
    );
    let right = compat_http_request(
        &server,
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Upload)
            .with_method("POST")
            .with_path("/v1/uploads")
            .with_diagnostics_profile(DiagnosticRichnessProfile::Standard)
            .with_body_present(true)
            .with_body_content_type("application/json")
            .build()
            .expect("right input should validate"),
    );
    let richer = compat_http_request(
        &server,
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Upload)
            .with_method("POST")
            .with_path("/v1/uploads")
            .with_diagnostics_profile(DiagnosticRichnessProfile::OperationalMinimal)
            .with_body_present(true)
            .with_body_content_type("application/json")
            .build()
            .expect("richer input should validate"),
    );

    assert_eq!(
        left.request_contract().canonical_digest(),
        right.request_contract().canonical_digest()
    );
    assert_ne!(
        left.request_contract().canonical_digest(),
        richer.request_contract().canonical_digest()
    );
}

#[test]
fn compat_http_accepts_head_reads_without_widening_route_family() {
    let request = compat_http_request(
        &compat_http_admission_test_server(),
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Read)
            .with_method("HEAD")
            .with_path("/v1/users/123")
            .with_header("accept", "application/json, */*")
            .build()
            .expect("input should validate"),
    );

    assert_eq!(
        request.request_contract().route_family(),
        WorthServerCompatHttpRouteFamily::Read
    );
}

#[test]
fn compat_http_denies_ambiguous_forwarding_headers_before_request_context_lowering() {
    let denial = compat_http_denial(
        &compat_http_admission_test_server(),
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/v1/users/123")
            .with_header("x-forwarded-proto", "https")
            .with_header("x-forwarded-proto", "http")
            .build()
            .expect("input should validate"),
    );

    assert_eq!(
        denial.code(),
        WorthServerCompatibilityDenialCode::AmbiguousForwardingHeaders
    );
}

#[test]
fn compat_http_denies_conflicting_api_version_headers_before_request_context_lowering() {
    let denial = compat_http_denial(
        &compat_http_admission_test_server(),
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/v1/users/123")
            .with_header("x-Worth-api-version", "1")
            .with_header("x-Worth-api-version", "2")
            .build()
            .expect("input should validate"),
    );

    assert_eq!(
        denial.code(),
        WorthServerCompatibilityDenialCode::UnsupportedApiVersion
    );
}

#[test]
fn compat_http_denies_body_metadata_without_body_before_request_context_lowering() {
    let denial = compat_http_denial(
        &compat_http_admission_test_server(),
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Upload)
            .with_method("POST")
            .with_path("/v1/uploads")
            .with_body_content_type("application/octet-stream")
            .build()
            .expect("input should validate"),
    );

    assert_eq!(
        denial.code(),
        WorthServerCompatibilityDenialCode::BodyMetadataWithoutBody
    );
}

#[test]
fn compat_http_denies_blank_body_content_type_before_request_context_lowering() {
    let denial = compat_http_denial(
        &compat_http_admission_test_server(),
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Upload)
            .with_method("POST")
            .with_path("/v1/uploads")
            .with_body_present(true)
            .with_body_content_type("   ")
            .build()
            .expect("input should validate"),
    );

    assert_eq!(
        denial.code(),
        WorthServerCompatibilityDenialCode::InvalidBodyContentType
    );
}

#[test]
fn compat_http_denies_body_on_head_before_request_context_lowering() {
    let denial = compat_http_denial(
        &compat_http_admission_test_server(),
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Read)
            .with_method("HEAD")
            .with_path("/v1/users/123")
            .with_body_present(true)
            .build()
            .expect("input should validate"),
    );

    assert_eq!(
        denial.code(),
        WorthServerCompatibilityDenialCode::UnexpectedRequestBody
    );
}

#[test]
fn compat_http_denies_incompatible_method_for_route_family() {
    let denial = compat_http_denial(
        &compat_http_admission_test_server(),
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Read)
            .with_method("POST")
            .with_path("/v1/users/123")
            .build()
            .expect("input should validate"),
    );

    assert_eq!(
        denial.code(),
        WorthServerCompatibilityDenialCode::IncompatibleMethodForRouteFamily
    );
}

#[test]
fn compat_http_admits_options_preflight_as_a_distinct_route_family() {
    let request = compat_http_request(
        &compat_http_admission_test_server(),
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Preflight)
            .with_method("OPTIONS")
            .with_path("/v1/users/123")
            .with_header("origin", "https://example.com")
            .build()
            .expect("input should validate"),
    );

    assert_eq!(
        request.request_contract().route_family(),
        WorthServerCompatHttpRouteFamily::Preflight
    );
}

#[test]
fn compat_http_denies_ambiguous_forwarded_host_before_request_context_lowering() {
    let denial = compat_http_denial(
        &compat_http_admission_test_server(),
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/v1/users/123")
            .with_header("x-forwarded-host", "api.one.test")
            .with_header("x-forwarded-host", "api.two.test")
            .build()
            .expect("input should validate"),
    );

    assert_eq!(
        denial.code(),
        WorthServerCompatibilityDenialCode::AmbiguousForwardingHeaders
    );
}

#[test]
fn compat_http_canonicalizes_repeated_query_keys_to_one_identity() {
    let server = compat_http_admission_test_server();
    let left = compat_http_request(
        &server,
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/v1/users/123")
            .with_query_pair("tag", "alpha")
            .with_query_pair("tag", "beta")
            .build()
            .expect("left input should validate"),
    );
    let right = compat_http_request(
        &server,
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_route_family(WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/v1/users/123")
            .with_query_pair("tag", "beta")
            .with_query_pair("tag", "alpha")
            .build()
            .expect("right input should validate"),
    );

    assert_eq!(
        left.request_contract().canonical_digest(),
        right.request_contract().canonical_digest()
    );
}
