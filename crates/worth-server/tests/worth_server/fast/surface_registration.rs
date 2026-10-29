use worth_server::{
    facade::{WorthServer, WorthServerBuildError},
    surfaces::WorthNativeSurface,
    WorthServerConfig, WorthServerSurfaceFamily, WorthServerSurfaceRegistryError,
};

#[test]
fn duplicate_surface_family_registration_fails_typed() {
    let build_error = WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .build()
                .expect("bind address should validate"),
        )
        .register_surface(WorthNativeSurface::disabled())
        .register_surface(WorthNativeSurface::disabled())
        .build()
        .expect_err("duplicate family should be rejected");

    assert_eq!(
        build_error,
        WorthServerBuildError::InvalidSurfaceRegistry(
            WorthServerSurfaceRegistryError::DuplicateSurfaceFamily {
                family: WorthServerSurfaceFamily::WorthNative,
            },
        )
    );
}

#[test]
fn duplicate_surface_rejection_fails_at_build_boundary() {
    let result = WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .build()
                .expect("bind address should validate"),
        )
        .register_surface(WorthNativeSurface::disabled())
        .register_surface(WorthNativeSurface::disabled())
        .build();

    assert!(matches!(
        result,
        Err(WorthServerBuildError::InvalidSurfaceRegistry(_))
    ));
}
