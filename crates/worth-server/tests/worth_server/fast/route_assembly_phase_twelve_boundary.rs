#[allow(unused_imports)]
use crate::shared_support_test_bootstrap_policy as test_bootstrap_policy;

use axum::http::StatusCode;

#[allow(unused_imports)]
use crate::shared_support_route_assembly_phase_twelve_fixture as fixture;
#[allow(unused_imports)]
use crate::shared_support_route_assembly_phase_twelve_request_driver as request_driver;

use fixture::build_server;
use request_driver::WorthServerRouteHttpTestDriver;

#[tokio::test]
async fn transport_errors_shape_without_product_semantics() {
    let server = build_server();
    let driver = WorthServerRouteHttpTestDriver::new(&server);

    let malformed = driver
        .post_bytes(
            "/compat/mutations/product_editor.apply?basis=basis:7",
            &[
                ("x-principal-id", "principal-7"),
                ("x-tenant-id", "tenant-a"),
                ("x-workspace-id", "workspace-42"),
                ("x-branch-id", "branch-9"),
                ("x-product-session-id", "product-session:test"),
            ],
            Some("application/json"),
            br#"{"title":"Rename""#.to_vec(),
        )
        .await;
    assert_eq!(malformed.status(), StatusCode::BAD_REQUEST);
    assert_eq!(malformed.route_kind(), Some("transport_denial"));
    assert_eq!(malformed.semantic_runtime_entered(), Some(false));
    assert_eq!(malformed.transport_denial_code(), Some("MalformedJson"));

    let unsupported_content = driver
        .post_bytes(
            "/compat/mutations/product_editor.apply?basis=basis:7",
            &[
                ("x-principal-id", "principal-7"),
                ("x-tenant-id", "tenant-a"),
                ("x-workspace-id", "workspace-42"),
                ("x-branch-id", "branch-9"),
                ("x-product-session-id", "product-session:test"),
            ],
            Some("text/plain"),
            b"rename".to_vec(),
        )
        .await;
    assert_eq!(
        unsupported_content.status(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
    assert_eq!(unsupported_content.route_kind(), Some("transport_denial"));
    assert_eq!(unsupported_content.semantic_runtime_entered(), Some(false));
    assert_eq!(
        unsupported_content.transport_denial_code(),
        Some("UnsupportedContentType")
    );

    let oversized = driver
        .post_bytes(
            "/compat/mutations/product_editor.apply?basis=basis:7",
            &[
                ("x-principal-id", "principal-7"),
                ("x-tenant-id", "tenant-a"),
                ("x-workspace-id", "workspace-42"),
                ("x-branch-id", "branch-9"),
                ("x-product-session-id", "product-session:test"),
            ],
            Some("application/json"),
            format!("{{\"title\":\"{}\"}}", "a".repeat(1024 * 1024)).into_bytes(),
        )
        .await;
    assert_eq!(oversized.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(oversized.route_kind(), Some("transport_denial"));
    assert_eq!(oversized.semantic_runtime_entered(), Some(false));
    assert_eq!(oversized.transport_denial_code(), Some("OversizedBody"));

    let unknown_route = driver
        .get(
            "/compat/reads/not-registered",
            &[
                ("x-principal-id", "principal-7"),
                ("x-tenant-id", "tenant-a"),
                ("x-workspace-id", "workspace-42"),
                ("x-branch-id", "branch-9"),
            ],
        )
        .await;
    assert_eq!(unknown_route.status(), StatusCode::NOT_FOUND);
    assert_eq!(unknown_route.route_kind(), Some("transport_denial"));
    assert_eq!(unknown_route.semantic_runtime_entered(), Some(false));
    assert_eq!(unknown_route.transport_denial_code(), Some("UnknownRoute"));

    let missing_tenant = driver
        .get(
            "/compat/reads/product_editor.render?basis=basis:7",
            &[
                ("x-principal-id", "principal-7"),
                ("x-workspace-id", "workspace-42"),
                ("x-branch-id", "branch-9"),
            ],
        )
        .await;
    assert_eq!(missing_tenant.status(), StatusCode::BAD_REQUEST);
    assert_eq!(missing_tenant.route_kind(), Some("transport_denial"));
    assert_eq!(missing_tenant.semantic_runtime_entered(), Some(false));
    assert_eq!(
        missing_tenant.transport_denial_code(),
        Some("MissingTenantId")
    );
}

#[tokio::test]
async fn operational_routes_do_not_enter_product_operation_runtime() {
    let server = build_server();
    let driver = WorthServerRouteHttpTestDriver::new(&server);

    for (method, path) in [
        ("GET", "/healthz"),
        ("GET", "/metrics"),
        ("GET", "/openapi.json"),
        ("OPTIONS", "/compat/preflight"),
    ] {
        let response = match method {
            "GET" => driver.get(path, &[]).await,
            "OPTIONS" => driver.options(path, &[]).await,
            _ => unreachable!("unexpected method"),
        };
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        assert_eq!(response.route_kind(), Some("operational"), "{path}");
        assert_eq!(response.semantic_runtime_entered(), Some(false), "{path}");
        assert_eq!(response.transport_denial_code(), None, "{path}");
    }
}
