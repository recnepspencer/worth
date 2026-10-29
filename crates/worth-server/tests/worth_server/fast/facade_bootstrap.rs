use worth_server::{
    facade::{WorthServer, WorthServerBuildError},
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServerConfig, WorthServerSurfaceFamily,
};

#[test]
fn build_preserves_explicit_surface_inventory_and_counter_snapshot() {
    let server = WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .build()
                .expect("bind address should validate"),
        )
        .register_surface(WorthNativeSurface::disabled())
        .register_surface(CompatHttpSurface::disabled())
        .build()
        .expect("server should build");

    assert_eq!(
        server.surface_inventory().registered_families,
        vec![
            WorthServerSurfaceFamily::WorthNative,
            WorthServerSurfaceFamily::CompatHttp,
        ]
    );
    assert_eq!(server.counters().registered_surface_families, 2);
    assert_eq!(
        server.counters().rejected_duplicate_surface_registrations,
        0
    );
    assert_eq!(server.counters().serve_start_count, 0);
}

#[test]
fn build_requires_validated_config_before_runtime_assembly() {
    let build_error = WorthServer::builder()
        .register_surface(WorthNativeSurface::disabled())
        .build()
        .expect_err("server must reject missing config");

    assert_eq!(build_error, WorthServerBuildError::MissingConfig);
}

#[test]
fn build_canonicalizes_surface_inventory_independent_of_registration_order() {
    let worth_native_first = WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .build()
                .expect("bind address should validate"),
        )
        .register_surface(WorthNativeSurface::disabled())
        .register_surface(CompatHttpSurface::disabled())
        .build()
        .expect("server should build");
    let compat_http_first = WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8081).into())
                .build()
                .expect("bind address should validate"),
        )
        .register_surface(CompatHttpSurface::disabled())
        .register_surface(WorthNativeSurface::disabled())
        .build()
        .expect("server should build");

    assert_eq!(
        worth_native_first.surface_inventory(),
        compat_http_first.surface_inventory()
    );
    assert_eq!(worth_native_first.counters(), compat_http_first.counters());
}
