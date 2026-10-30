#[test]
fn duplicate_surface_family_registration_fails_typed() {
    let build_error = worth_server::facade::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .build()
                .expect("bind address should validate"),
        )
        .register_surface(worth_server::surfaces::WorthNativeSurface::disabled())
        .register_surface(worth_server::surfaces::WorthNativeSurface::disabled())
        .build()
        .expect_err("duplicate family should be rejected");

    assert_eq!(
        build_error,
        worth_server::facade::WorthServerBuildError::InvalidSurfaceRegistry(
            worth_server::WorthServerSurfaceRegistryError::DuplicateSurfaceFamily {
                family: worth_server::WorthServerSurfaceFamily::WorthNative,
            },
        )
    );
}

#[test]
fn duplicate_surface_rejection_fails_at_build_boundary() {
    let result = worth_server::facade::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .build()
                .expect("bind address should validate"),
        )
        .register_surface(worth_server::surfaces::WorthNativeSurface::disabled())
        .register_surface(worth_server::surfaces::WorthNativeSurface::disabled())
        .build();

    assert!(matches!(
        result,
        Err(worth_server::facade::WorthServerBuildError::InvalidSurfaceRegistry(_))
    ));
}
