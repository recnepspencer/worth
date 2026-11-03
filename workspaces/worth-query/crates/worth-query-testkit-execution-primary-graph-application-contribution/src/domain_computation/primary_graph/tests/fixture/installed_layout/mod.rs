use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstallationAdmissionProfile;
use worth_query_installation::facade::WorthQueryInstallationGeneration;
use worth_query_installation::facade::WorthQueryPortableDomainIdentity;
use worth_query_installation::facade::WorthQueryPortableDomainPackage;
use worth_relational::facade::schema::RelationalSchemaRegistry;

use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;

/// The primary-graph layout an installed identity schema lowers to.
pub fn installed_layout() -> WorthQueryPrimaryGraphLayout {
    installed_schema_layout::<IdentityExecutionSchema>().0
}

/// Installed declaration/native contract custody for a schema-layout owner court.
/// This uses the same existing installation root; it creates no graph or read grant.
pub fn installed_schema_layout<Schema: ApplicationSchema>(
) -> (WorthQueryPrimaryGraphLayout, RelationalSchemaRegistry) {
    let declaration = Schema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        Schema::OWNER,
        Schema::MAJOR,
        Schema::MINOR,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    let (runtime, _) = WorthQueryExecutionRuntimeInstaller::new()
        .install(WorthQueryInstallationGeneration::initial(), [admitted])
        .unwrap()
        .into_parts();
    let installed = runtime
        .installed_packages()
        .bind_application_schema(declaration)
        .unwrap();
    WorthQueryPrimaryGraphLayout::lower(
        installed.installed_declaration(),
        installed.native_contracts(),
        &RelationalSchemaRegistry::new(),
    )
    .unwrap()
}
