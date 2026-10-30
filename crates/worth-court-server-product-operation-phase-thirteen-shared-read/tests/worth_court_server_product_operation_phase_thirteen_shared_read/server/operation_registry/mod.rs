#[allow(unused_imports)]
use crate::operation_registry::ordinary_context::*;
use worth_proof::TransitionOutcome;
use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};
use {
    worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface,
    worth_server::WorthServer, worth_server::WorthServerBuildError,
    worth_server::WorthServerCompatHttpRouteFamily,
    worth_server::WorthServerCompatibilityExecutionInput,
    worth_server::WorthServerCompatibilityRequestInput, worth_server::WorthServerConfig,
    worth_server::WorthServerOperationDenial, worth_server::WorthServerOperationFamily,
    worth_server::WorthServerOperationRegistration, worth_server::WorthServerQueryHandoffConfig,
    worth_server::WorthServerQueryWorkspaceBindingError,
    worth_server::WorthServerQueryWorkspaceBindingRequest,
    worth_server::WorthServerQueryWorkspaceProvider, worth_server::WorthServerRequestContextConfig,
};

#[test]
fn operation_family_inventory_is_distinct_from_surface_inventory() {
    let server = build_server(
        crate::operation_registry::ordinary_context::CountingWorkspaceProvider::default(),
        worth_server::WorthServerOperationRegistration::phase_two_defaults(),
    )
    .expect("server should build");

    let surface_inventory = server.surface_inventory();
    let operation_inventory = server.operation_inventory();

    assert_eq!(surface_inventory.registered_families.len(), 2);
    assert!(operation_inventory.registered_families().len() >= 8);
    assert!(operation_inventory
        .registered_families()
        .contains(&worth_server::WorthServerOperationFamily::QueryDirectRead));
    assert!(operation_inventory
        .registered_families()
        .contains(&worth_server::WorthServerOperationFamily::SyncLease));
    assert_eq!(
        server.operation_registry().admit(
            worth_server::WorthServerSurfaceFamily::CompatHttp,
            worth_server::WorthServerOperationFamily::QueryDirectProjection,
        ),
        Err(
            worth_server::WorthServerOperationDenial::SurfaceFamilyNotExposed {
                family: worth_server::WorthServerOperationFamily::QueryDirectProjection,
                surface_family: worth_server::WorthServerSurfaceFamily::CompatHttp,
            }
        )
    );
}

#[test]
fn unregistered_operation_family_denies_before_execution() {
    let provider =
        crate::operation_registry::ordinary_context::CountingWorkspaceProvider::default();
    let binds = provider.bind_count();
    let server = build_server(
        provider,
        vec![worth_server::WorthServerOperationRegistration::enabled(
            worth_server::WorthServerOperationFamily::QueryDirectSubmission,
        )
        .exposed_on([
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerSurfaceFamily::CompatHttp,
        ])],
    )
    .expect("server should build");

    let prepared_request = prepared_read_request(&server);
    let outcome =
        server
            .compat_http()
            .read(worth_server::WorthServerCompatibilityExecutionInput::new(
                prepared_request,
                "users.profile",
            ));

    let denial = match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected operation-family denial, got {other:?}"),
    };
    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::OperationFamilyNotRegistered
    );
    assert_eq!(binds.load(std::sync::atomic::Ordering::Relaxed), 0);
}

#[test]
fn disabled_operation_family_denies_before_execution() {
    let provider =
        crate::operation_registry::ordinary_context::CountingWorkspaceProvider::default();
    let binds = provider.bind_count();
    let server = build_server(
        provider,
        vec![worth_server::WorthServerOperationRegistration::disabled(
            worth_server::WorthServerOperationFamily::QueryDirectRead,
        )
        .exposed_on([
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerSurfaceFamily::CompatHttp,
        ])],
    )
    .expect("server should build");

    let prepared_request = prepared_read_request(&server);
    let outcome =
        server
            .compat_http()
            .read(worth_server::WorthServerCompatibilityExecutionInput::new(
                prepared_request,
                "users.profile",
            ));

    let denial = match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected operation-family denial, got {other:?}"),
    };
    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::OperationFamilyDisabled
    );
    assert_eq!(binds.load(std::sync::atomic::Ordering::Relaxed), 0);
}

#[test]
fn operation_family_surface_exposure_denies_before_execution() {
    let provider =
        crate::operation_registry::ordinary_context::CountingWorkspaceProvider::default();
    let binds = provider.bind_count();
    let server = build_server(
        provider,
        vec![worth_server::WorthServerOperationRegistration::enabled(
            worth_server::WorthServerOperationFamily::QueryDirectRead,
        )
        .exposed_on([worth_server::WorthServerSurfaceFamily::WorthNative])],
    )
    .expect("server should build");

    let prepared_request = prepared_read_request(&server);
    let outcome =
        server
            .compat_http()
            .read(worth_server::WorthServerCompatibilityExecutionInput::new(
                prepared_request,
                "users.profile",
            ));

    let denial = match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected operation-family denial, got {other:?}"),
    };
    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::OperationFamilyNotExposedOnSurface
    );
    assert_eq!(binds.load(std::sync::atomic::Ordering::Relaxed), 0);
}

#[test]
fn duplicate_operation_family_registration_fails_server_build() {
    let build_error = build_server(
        crate::operation_registry::ordinary_context::CountingWorkspaceProvider::default(),
        vec![
            worth_server::WorthServerOperationRegistration::enabled(
                worth_server::WorthServerOperationFamily::QueryDirectRead,
            ),
            worth_server::WorthServerOperationRegistration::disabled(
                worth_server::WorthServerOperationFamily::QueryDirectRead,
            ),
        ],
    )
    .expect_err("duplicate operation registrations must fail build");

    assert_eq!(
        build_error,
        worth_server::WorthServerBuildError::InvalidOperationRegistry(
            worth_server::WorthServerOperationRegistryError::DuplicateOperationFamily {
                family: worth_server::WorthServerOperationFamily::QueryDirectRead,
            },
        )
    );
}
