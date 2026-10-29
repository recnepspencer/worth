pub(crate) use worth_query_testkit_host::facade::host::application_runtime_surface::*;

#[test]
fn host_facade_publishes_a_narrow_primary_graph_application_runtime() {
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(8_000_000),
        ),
    );
    let application =
        worth_query_host::facade::application_contribution::with_bootstrap_advancement(
            policy,
            |phase| {
                let declaration = HostIdentitySchema::declaration().unwrap();
                let package = WorthQueryPortableDomainPackage::new(
                    WorthQueryPortableDomainIdentity::new("host_identity_test", 1, 0),
                )
                .application_schema(declaration.clone())
                .validate()
                .unwrap();
                let admitted =
                    WorthQueryInstallationAdmissionProfile::new("host-support", "host-config")
                        .admit(package)
                        .unwrap();
                let installation = WorthQueryExecutionRuntimeInstaller::new()
                    .install(WorthQueryInstallationGeneration::initial(), [admitted])
                    .unwrap();
                let (runtime, authority) = installation.into_parts();
                let schema = runtime
                    .installed_packages()
                    .bind_application_schema(declaration)
                    .unwrap();
                let binding = schema
                    .principal_binding(IdentityBinding::reference())
                    .unwrap();
                let mut graph = authority
                .prepare_primary_graph(
                    &phase,
                    &runtime,
                    &schema,
                    worth_query_execution::facade::integration::product_world_resources_for_test(
                        1_024,
                    ),
                )
                .unwrap();
                graph
                    .bind_principal(
                        &binding,
                        WorthQueryApplicationPrincipalKey::new("host-principal").unwrap(),
                        1_u64,
                        WorthQueryExternalPrincipalIdentity::new(
                            "https://issuer.example",
                            "subject",
                        )
                        .unwrap(),
                        WorthQueryPrincipalMappingStatus::Enabled,
                    )
                    .unwrap();
                graph
                    .bind_entity(
                        WorthQueryApplicationEntitySeed::new(
                            Account::reference(),
                            WorthQueryApplicationEntityKey::new("account-row").unwrap(),
                        )
                        .field(AccountNumber::reference(), "account-001".to_string()),
                    )
                    .unwrap();

                graph
        .publish_application_runtime(
            &phase,
            runtime,
            authority,
            schema,
            worth_query_host::facade::primary_graph::SignalConditionalEvaluationBudget::development(
            ),
        )
        .unwrap()
            },
        )
        .unwrap();

    assert_eq!(application.publication().principal_binding_count(), 1);
    assert_eq!(application.publication().identity_index_count(), 1);
    assert_eq!(
        application.publication().application_equality_index_count(),
        4
    );
    assert_eq!(
        application.installed_schema().schema_name(),
        "HostIdentitySchema"
    );
    let cancellation = WorthQueryCancellationSource::new();
    let request_scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    let account = application
        .with_host_advancement(|phase| {
            application
                .on_branch(application.current_world())
                .select(&phase)
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountNumber::reference(),
                    "account-001".to_string(),
                    &request_scope,
                    WorthQueryPrincipalResolutionMode::Certification,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    assert_eq!(account.examined_candidate_count(), 1);
    assert_eq!(
        account.binding_identity(),
        application.publication().binding_identity()
    );
}
