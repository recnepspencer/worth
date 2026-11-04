use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclaration;
use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder;
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
use worth_query_installation::facade::WorthQueryInstallationAdmissionProfile;
use worth_query_installation::facade::WorthQueryInstallationGeneration;
use worth_query_installation::facade::WorthQueryPortableDomainIdentity;
use worth_query_installation::facade::WorthQueryPortableDomainPackage;

use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;

pub static DECLARATION_CALLS: AtomicUsize = AtomicUsize::new(0);

pub struct InstalledCatalogAuthoritySchema;
worth_query_declaration::worth_query_entity!(InstalledEntity for InstalledCatalogAuthoritySchema);
worth_query_declaration::worth_query_aspect!(
    InstalledAspect for InstalledCatalogAuthoritySchema, InstalledEntity;
    identity = AspectIdentity(0x9161_0101),
    revision = AspectContractRevision(1),
);
worth_query_declaration::worth_query_field!(
    InstalledField for InstalledCatalogAuthoritySchema, InstalledEntity, InstalledAspect:
    u64 => U64ApplicationValueBinding, read_only, no_equality
);

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
        ApplicationSchemaDeclarationBuilder::for_schema()
            .entity(InstalledEntity::reference())
            .aspect(InstalledEntity::reference(), InstalledAspect::reference())
            .field(InstalledEntity::reference(), InstalledField::reference())
            .build()
    }
}

#[test]
fn handler_sealing_consumes_the_retained_installed_catalog() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
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
            worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()),
        )
        .unwrap();

        (*bootstrap
            .mutation_handlers_for_test())
            .seal_for_test(&schema, (*bootstrap.graph_for_test()).binding_identity())
            .unwrap();

        assert_eq!(DECLARATION_CALLS.load(Ordering::SeqCst), 2);
    })
}
