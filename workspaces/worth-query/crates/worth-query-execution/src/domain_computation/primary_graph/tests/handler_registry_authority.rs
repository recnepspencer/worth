#[path = "handler_registry_authority/records.rs"]
mod storage_model;
use std::sync::atomic::{AtomicUsize, Ordering};
pub use storage_model::*;

use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntimeInstaller;

static DECLARATION_CALLS: AtomicUsize = AtomicUsize::new(0);

struct InstalledCatalogAuthoritySchema;

impl ApplicationSchema for InstalledCatalogAuthoritySchema {
    const OWNER: &'static str = "installed-catalog-authority-test";
    const NAME: &'static str = "InstalledCatalogAuthoritySchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        assert!(
            DECLARATION_CALLS.fetch_add(1, Ordering::SeqCst) < 2,
            "handler sealing must not become another schema declaration authority"
        );
        ApplicationSchemaDeclarationBuilder::from_fragment(RegistryRecords)
            .expect("owned record model is valid")
            .build()
    }
}

#[test]
fn handler_sealing_consumes_the_retained_installed_catalog() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;

        let declaration = InstalledCatalogAuthoritySchema::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            InstalledCatalogAuthoritySchema::OWNER,
            InstalledCatalogAuthoritySchema::MAJOR,
            InstalledCatalogAuthoritySchema::MINOR,
        ))
        .application_schema(declaration.clone())
        .validate()
        .unwrap();
        let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
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
        let bootstrap = authority
        .prepare_primary_graph(&phase.bootstrap_for_test(),
            &runtime,
            &schema,
            crate::domain_computation::execution_runtime::product_world::test_product_world_resources(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()),
        )
        .unwrap();

        bootstrap
            .mutation_handlers
            .seal(&schema, bootstrap.graph.binding_identity())
            .unwrap();

        assert_eq!(DECLARATION_CALLS.load(Ordering::SeqCst), 2);
    })
}
