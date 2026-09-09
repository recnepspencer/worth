use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_execution::facade::{
    primary_graph::WorthQueryApplicationPrincipalKey, runtime::WorthQueryExecutionRuntimeInstaller,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

use super::super::declaration::{PublicationAuthorizationSchema, PublicationIdentityBinding};
use super::{authentication, baseline_graph, scenario};
use super::{CompositionScenario, InstalledWorld};

pub(super) fn install_world(composition_scenario: CompositionScenario) -> InstalledWorld {
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
    let mut graph = authority.prepare_primary_graph(&runtime, &schema).unwrap();

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
            runtime,
            authority,
            schema,
            worth_query_execution::facade::primary_graph::SignalConditionalEvaluationBudget::development(),
        )
        .unwrap();
    InstalledWorld { runtime, binding }
}
