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
    let declaration = IdentityExecutionSchema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        IdentityExecutionSchema::OWNER,
        IdentityExecutionSchema::MAJOR,
        IdentityExecutionSchema::MINOR,
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
    .0
}
