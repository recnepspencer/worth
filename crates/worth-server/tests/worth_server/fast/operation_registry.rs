use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_proof::TransitionOutcome;
use worth_server::{
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerBuildError, WorthServerCompatHttpRouteFamily,
    WorthServerCompatibilityExecutionInput, WorthServerCompatibilityRequestInput,
    WorthServerConfig, WorthServerOperationDenial, WorthServerOperationFamily,
    WorthServerOperationRegistration, WorthServerQueryHandoffConfig,
    WorthServerQueryWorkspaceBindingError, WorthServerQueryWorkspaceBindingRequest,
    WorthServerQueryWorkspaceProvider, WorthServerRequestContextConfig,
};

#[test]
fn operation_family_inventory_is_distinct_from_surface_inventory() {
    let server = build_server(
        CountingWorkspaceProvider::default(),
        WorthServerOperationRegistration::phase_two_defaults(),
    )
    .expect("server should build");

    let surface_inventory = server.surface_inventory();
    let operation_inventory = server.operation_inventory();

    assert_eq!(surface_inventory.registered_families.len(), 2);
    assert!(operation_inventory.registered_families().len() >= 8);
    assert!(operation_inventory
        .registered_families()
        .contains(&WorthServerOperationFamily::QueryDirectRead));
    assert!(operation_inventory
        .registered_families()
        .contains(&WorthServerOperationFamily::SyncLease));
    assert_eq!(
        server.operation_registry().admit(
            worth_server::WorthServerSurfaceFamily::CompatHttp,
            WorthServerOperationFamily::QueryDirectProjection,
        ),
        Err(WorthServerOperationDenial::SurfaceFamilyNotExposed {
            family: WorthServerOperationFamily::QueryDirectProjection,
            surface_family: worth_server::WorthServerSurfaceFamily::CompatHttp,
        })
    );
}

#[test]
fn unregistered_operation_family_denies_before_execution() {
    let provider = CountingWorkspaceProvider::default();
    let binds = provider.bind_count();
    let server = build_server(
        provider,
        vec![WorthServerOperationRegistration::enabled(
            WorthServerOperationFamily::QueryDirectSubmission,
        )
        .exposed_on([
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerSurfaceFamily::CompatHttp,
        ])],
    )
    .expect("server should build");

    let prepared_request = prepared_read_request(&server);
    let outcome = server
        .compat_http()
        .read(WorthServerCompatibilityExecutionInput::new(
            prepared_request,
            "users.profile",
        ));

    let denial = match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected operation-family denial, got {other:?}"),
    };
    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::OperationFamilyNotRegistered
    );
    assert_eq!(binds.load(Ordering::Relaxed), 0);
}

#[test]
fn disabled_operation_family_denies_before_execution() {
    let provider = CountingWorkspaceProvider::default();
    let binds = provider.bind_count();
    let server = build_server(
        provider,
        vec![WorthServerOperationRegistration::disabled(
            WorthServerOperationFamily::QueryDirectRead,
        )
        .exposed_on([
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerSurfaceFamily::CompatHttp,
        ])],
    )
    .expect("server should build");

    let prepared_request = prepared_read_request(&server);
    let outcome = server
        .compat_http()
        .read(WorthServerCompatibilityExecutionInput::new(
            prepared_request,
            "users.profile",
        ));

    let denial = match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected operation-family denial, got {other:?}"),
    };
    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::OperationFamilyDisabled
    );
    assert_eq!(binds.load(Ordering::Relaxed), 0);
}

#[test]
fn operation_family_surface_exposure_denies_before_execution() {
    let provider = CountingWorkspaceProvider::default();
    let binds = provider.bind_count();
    let server = build_server(
        provider,
        vec![WorthServerOperationRegistration::enabled(
            WorthServerOperationFamily::QueryDirectRead,
        )
        .exposed_on([worth_server::WorthServerSurfaceFamily::WorthNative])],
    )
    .expect("server should build");

    let prepared_request = prepared_read_request(&server);
    let outcome = server
        .compat_http()
        .read(WorthServerCompatibilityExecutionInput::new(
            prepared_request,
            "users.profile",
        ));

    let denial = match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected operation-family denial, got {other:?}"),
    };
    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::OperationFamilyNotExposedOnSurface
    );
    assert_eq!(binds.load(Ordering::Relaxed), 0);
}

#[test]
fn duplicate_operation_family_registration_fails_server_build() {
    let build_error = build_server(
        CountingWorkspaceProvider::default(),
        vec![
            WorthServerOperationRegistration::enabled(WorthServerOperationFamily::QueryDirectRead),
            WorthServerOperationRegistration::disabled(WorthServerOperationFamily::QueryDirectRead),
        ],
    )
    .expect_err("duplicate operation registrations must fail build");

    assert_eq!(
        build_error,
        WorthServerBuildError::InvalidOperationRegistry(
            worth_server::WorthServerOperationRegistryError::DuplicateOperationFamily {
                family: WorthServerOperationFamily::QueryDirectRead,
            },
        )
    );
}

fn build_server(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
    operations: Vec<WorthServerOperationRegistration>,
) -> Result<WorthServer, WorthServerBuildError> {
    WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    WorthServerRequestContextConfig::builder()
                        .with_preview_targeting_enabled(true)
                        .with_default_diagnostics_profile(
                            worth_server::request_context::DiagnosticRichnessProfile::Standard,
                        )
                        .build()
                        .expect("request context config should validate"),
                )
                .with_query_handoff_config(
                    WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(workspace_provider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(operations)
        .register_surface(WorthNativeSurface::enabled())
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .build()
}

fn prepared_read_request(
    server: &WorthServer,
) -> worth_server::WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_route_family(WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/compat/reads/users.profile")
            .with_header("accept", "application/json")
            .build()
            .expect("compat request should validate"),
    ) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared compatibility request, got {other:?}"),
    }
}

#[derive(Clone, Debug, Default)]
struct CountingWorkspaceProvider {
    bind_count: Arc<AtomicUsize>,
}

impl CountingWorkspaceProvider {
    fn bind_count(&self) -> Arc<AtomicUsize> {
        self.bind_count.clone()
    }
}

impl WorthServerQueryWorkspaceProvider for CountingWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "counting-workspace-provider"
    }

    fn bind_workspace(
        &self,
        _request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<
        worth_query::facade::runtime::WorthQueryWorkspace,
        WorthServerQueryWorkspaceBindingError,
    > {
        self.bind_count.fetch_add(1, Ordering::Relaxed);
        Err(WorthServerQueryWorkspaceBindingError::new(
            "bind_workspace",
            "operation-family guard should deny before workspace binding",
        ))
    }
}
