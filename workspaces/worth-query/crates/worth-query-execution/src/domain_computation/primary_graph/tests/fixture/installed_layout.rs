use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
use worth_relational::facade::schema::RelationalSchemaRegistry;

use super::IdentityExecutionSchema;
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntimeInstaller;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;

/// The primary-graph layout an installed identity schema lowers to.
pub(in crate::domain_computation::primary_graph) fn installed_layout(
) -> WorthQueryPrimaryGraphLayout {
    installed_schema_layout::<IdentityExecutionSchema>().0
}

/// Installed declaration/native contract custody for a schema-layout owner court.
/// This uses the same existing installation root; it creates no graph or read grant.
pub(in crate::domain_computation::primary_graph) fn installed_schema_layout<
    Schema: ApplicationSchema,
>() -> (WorthQueryPrimaryGraphLayout, RelationalSchemaRegistry) {
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
