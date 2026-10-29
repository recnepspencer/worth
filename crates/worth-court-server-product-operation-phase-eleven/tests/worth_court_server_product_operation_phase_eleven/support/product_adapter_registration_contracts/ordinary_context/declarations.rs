#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

pub(crate) use {std::sync::Arc};

pub(crate) use {axum::body::to_bytes, axum::body::Body};
pub(crate) use {axum::http::Method, axum::http::Request, axum::http::StatusCode};
pub(crate) use {serde_json::json};
pub(crate) use {tower::ServiceExt};
pub(crate) use {worth_query_host::facade::domain, worth_query_host::facade::primary_graph, worth_query_host::facade::runtime};
pub(crate) use {worth_query_host::facade::worth_query_application_schema, worth_query_host::facade::worth_query_aspect, worth_query_host::facade::worth_query_entity, worth_query_host::facade::worth_query_field, worth_query_host::facade::worth_query_principal_binding, worth_query_host::facade::worth_query_relation};
pub(crate) use {worth_server::CompatHttpSurface, worth_server::WorthNativeSurface, worth_server::WorthServer, worth_server::WorthServerBuildError, worth_server::WorthServerProductAdapterCertificationCode, worth_server::WorthServerProductApplicationAdapterRegistration, worth_server::WorthServerProductOperationBasisKind, worth_server::WorthServerProductOperationDeclaration, worth_server::WorthServerProductOperationErrorMaps, worth_server::WorthServerProductOperationSupportSnapshot};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_product_adapter_phase_nine_fixture as fixture};

pub(crate) use {worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::base_config, worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::build_server, worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::result_contract, worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::EditorAdapter};

pub(crate) fn primary_graph_application(
) -> worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<SessionBasisSchema> {
    let declaration = SessionBasisSchema::declaration().expect("test schema must declare");
    let package = worth_query_host::facade::domain::WorthQueryPortableDomainPackage::new(
        worth_query_host::facade::domain::WorthQueryPortableDomainIdentity::new("worth_server_session_basis_test", 1, 0),
    )
    .application_schema(declaration.clone())
    .validate()
    .expect("test package must validate");
    let admitted = worth_query_host::facade::domain::WorthQueryInstallationAdmissionProfile::new("server", "session-basis")
        .admit(package)
        .expect("test package must admit");
    let installation = worth_query_host::facade::runtime::WorthQueryExecutionRuntimeInstaller::new()
        .install(
            worth_query_host::facade::domain::WorthQueryInstallationGeneration::initial(),
            [admitted],
        )
        .expect("test package must install");
    let (runtime, authority) = installation.into_parts();
    let schema = runtime
        .installed_packages()
        .bind_application_schema(declaration)
        .expect("test schema must bind");
    let binding = schema
        .principal_binding(SessionIdentityBinding::reference())
        .expect("test principal binding must install");
    // This fixture policy bounds session-basis graph preparation and evaluation.
    let execution_policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(8_000_000),
        ),
    );
    worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        execution_policy,
        |phase| {
    let mut graph = authority
        .prepare_primary_graph(
            &phase,
            &runtime,
            &schema,
            worth_query_host::facade::runtime::WorthQueryProductWorldResources::install(
                worth_query_host::facade::runtime::RuntimeWorldBudgetInstallation {
                    branches: worth_query_host::facade::runtime::RuntimeWorldBranchBudgetInstallation { live_product_branches: 128 },
                    history: worth_query_host::facade::runtime::RuntimeWorldHistoryBudgetInstallation { retained_composite_commits: 1_024, history_metadata_bytes: 16 * 1024 * 1024 },
                    observations: worth_query_host::facade::runtime::RuntimeWorldObservationBudgetInstallation { active_observations: 512 },
                    publication: worth_query_host::facade::runtime::RuntimeWorldPublicationBudgetInstallation { active_publication_attempts: 128 },
                    recovery: worth_query_host::facade::runtime::RuntimeWorldRecoveryBudgetInstallation { retained_product_unpublished_records: 128, retained_partial_metadata_bytes: 16 * 1024 * 1024 },
                    retention: worth_query_host::facade::runtime::RuntimeWorldRetentionBudgetInstallation { unique_exact_component_pins: 1_024, in_flight_pin_acquisition_reservations: 256 },
                    custody: worth_query_host::facade::runtime::RuntimeWorldCustodyBudgetInstallation { owner_created_component_custody_records: 256 },
                },
                worth_query_host::facade::runtime::WorthQueryProductWorldClock::start(),
                worth_query_host::facade::runtime::WorthQueryInvalidationResources::new(),
                execution_policy,
            )
            .expect("the server test Product World resources are valid"),
        )
        .expect("test graph must prepare");
    graph
        .bind_principal(
            &binding,
            worth_query_host::facade::primary_graph::WorthQueryApplicationPrincipalKey::new("session-principal")
                .expect("principal key must be valid"),
            1_u64,
            worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
                "https://issuer.example",
                "session-principal",
            )
            .expect("external identity must be valid"),
            worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
        )
        .expect("test principal must bind");
    graph
        .publish_application_runtime(
            &phase,
            runtime,
            authority,
            schema,
            worth_query_host::facade::primary_graph::SignalConditionalEvaluationBudget::development(),
        )
        .expect("test application must publish")
        },
    ).expect("the declared server fixture policy admits installation")
}

pub(crate) fn build_broken_registration_server(
    declaration: WorthServerProductOperationDeclaration,
) -> Result<WorthServer, WorthServerBuildError> {
    worth_server::WorthServer::builder()
        .with_config(base_config())
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(worth_server::WorthNativeSurface::enabled())
        .register_surface(worth_server::CompatHttpSurface::phase_one_enabled())
        .register_product_adapter(
            worth_server::WorthServerProductApplicationAdapterRegistration::new(
                "broken-editor",
                std::sync::Arc::new(worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::EditorAdapter::default()),
            )
            .with_operation(declaration),
        )
        .build()
}

pub(crate) fn assert_registration_code(
    result: Result<WorthServer, WorthServerBuildError>,
    expected_code: WorthServerProductAdapterCertificationCode,
) {
    match result {
        Err(worth_server::WorthServerBuildError::InvalidProductAdapterRegistry(error)) => {
            assert_eq!(error.certification_code(), Some(expected_code));
        }
        other => panic!("expected product adapter registration failure, got {other:?}"),
    }
}