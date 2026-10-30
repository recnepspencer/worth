pub use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
pub use worth_query_execution::facade::application_contribution::WorthQueryBootstrapAdvancementPhase;
pub use worth_query_execution::facade::primary_graph::WorthQueryApplicationPrincipalKey;
pub use worth_query_execution::facade::runtime::WorthQueryExecutionRuntimeInstaller;
pub use worth_query_installation::facade::WorthQueryInstallationAdmissionProfile;
pub use worth_query_installation::facade::WorthQueryInstallationGeneration;
pub use worth_query_installation::facade::WorthQueryPortableDomainIdentity;
pub use worth_query_installation::facade::WorthQueryPortableDomainPackage;

pub use super::super::declaration::PublicationAuthorizationSchema;
pub use super::super::declaration::PublicationIdentityBinding;
pub use super::authentication;
pub use super::baseline_graph;
pub use super::scenario;
pub use super::CompositionScenario;
pub use super::InstalledWorld;

pub fn install_world(composition_scenario: CompositionScenario) -> InstalledWorld {
    let resources = product_world_resources();
    worth_query_execution::facade::application_contribution::with_bootstrap_advancement(
        resources.execution_policy(),
        |phase| install_in_advancement(&phase, resources, composition_scenario),
    )
    .expect("the declared fixture policy admits its installation")
}

pub fn install_in_advancement(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    resources: worth_query_execution::facade::runtime::WorthQueryProductWorldResources,
    composition_scenario: CompositionScenario,
) -> InstalledWorld {
    let declaration = PublicationAuthorizationSchema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "publication_authorization_proof",
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("proof", "configuration")
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
        .principal_binding(PublicationIdentityBinding::reference())
        .unwrap();
    let mut graph = authority
        .prepare_primary_graph(phase, &runtime, &schema, resources)
        .unwrap();

    graph
        .bind_principal(
            &binding,
            WorthQueryApplicationPrincipalKey::new("principal-1").unwrap(),
            1_u64,
            authentication::external_identity(),
            WorthQueryPrincipalMappingStatus::Enabled,
        )
        .unwrap();
    baseline_graph::bind(&mut graph);
    scenario::bind(&mut graph, composition_scenario);

    let runtime = graph
        .publish_application_runtime(
            phase,
            runtime,
            authority,
            schema,
            worth_query_execution::facade::primary_graph::SignalConditionalEvaluationBudget::development(),
        )
        .unwrap();
    InstalledWorld { runtime, binding }
}

pub fn product_world_resources(
) -> worth_query_execution::facade::runtime::WorthQueryProductWorldResources {
    use worth_query_execution::facade::runtime as query_runtime;

    // Bounds this fixture World's serial request memory and deterministic work.
    let execution_policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(8_000_000),
        ),
    );
    query_runtime::WorthQueryProductWorldResources::install(
        query_runtime::RuntimeWorldBudgetInstallation {
            branches: query_runtime::RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 128,
            },
            history: query_runtime::RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 1_024,
                history_metadata_bytes: 16 * 1024 * 1024,
            },
            observations: query_runtime::RuntimeWorldObservationBudgetInstallation {
                active_observations: 512,
            },
            publication: query_runtime::RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 128,
            },
            recovery: query_runtime::RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 128,
                retained_partial_metadata_bytes: 16 * 1024 * 1024,
            },
            retention: query_runtime::RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 1_024,
                in_flight_pin_acquisition_reservations: 256,
            },
            custody: query_runtime::RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 256,
            },
        },
        query_runtime::WorthQueryProductWorldClock::start(),
        worth_query_execution::facade::runtime::WorthQueryInvalidationResources::new(),
        execution_policy,
    )
    .expect("the publication courtroom Product World resources are valid")
}
