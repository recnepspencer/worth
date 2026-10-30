#[test]
fn build_preserves_explicit_surface_inventory_and_counter_snapshot() {
    let server = worth_server::facade::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .build()
                .expect("bind address should validate"),
        )
        .register_surface(worth_server::surfaces::WorthNativeSurface::disabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::disabled())
        .build()
        .expect("server should build");

    assert_eq!(
        server.surface_inventory().registered_families,
        vec![
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerSurfaceFamily::CompatHttp,
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
    let build_error = worth_server::facade::WorthServer::builder()
        .register_surface(worth_server::surfaces::WorthNativeSurface::disabled())
        .build()
        .expect_err("server must reject missing config");

    assert_eq!(
        build_error,
        worth_server::facade::WorthServerBuildError::MissingConfig
    );
}

#[test]
fn build_canonicalizes_surface_inventory_independent_of_registration_order() {
    let worth_native_first = worth_server::facade::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .build()
                .expect("bind address should validate"),
        )
        .register_surface(worth_server::surfaces::WorthNativeSurface::disabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::disabled())
        .build()
        .expect("server should build");
    let compat_http_first = worth_server::facade::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8081).into())
                .build()
                .expect("bind address should validate"),
        )
        .register_surface(worth_server::surfaces::CompatHttpSurface::disabled())
        .register_surface(worth_server::surfaces::WorthNativeSurface::disabled())
        .build()
        .expect("server should build");

    assert_eq!(
        worth_native_first.surface_inventory(),
        compat_http_first.surface_inventory()
    );
    assert_eq!(worth_native_first.counters(), compat_http_first.counters());
}
