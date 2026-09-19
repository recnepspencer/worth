use worth_query_declaration::facade::application_schema::{
    ApplicationEntityRef, ApplicationSchema, ApplicationSchemaContributionAuthoring,
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledApplicationSchemaDenialKind,
    WorthQueryInstalledPackageIndex, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

struct ContributionSchema;
struct Component;

worth_query_declaration::worth_query_application_contribution! {
    contribution TopologyContribution in ContributionSchema {
        identity: "worth.tests.topology.v1",
        members: |schema| {
            schema.entity(ApplicationEntityRef::<ContributionSchema, Component>::from_schema_identifier(
                "Component",
            ))
        }
    }
}

worth_query_declaration::worth_query_application_contribution! {
    contribution ReassignedContribution in ContributionSchema {
        identity: "worth.tests.reassigned.v1",
        members: |schema| {
            schema.entity(ApplicationEntityRef::<ContributionSchema, Component>::from_schema_identifier(
                "Component",
            ))
        }
    }
}

impl ApplicationSchema for ContributionSchema {
    const OWNER: &'static str = "worth.tests.contribution-installation";
    const NAME: &'static str = "ContributionSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .contributions()
            .register::<TopologyContribution>()?
            .build()
    }
}

#[test]
fn installed_catalog_resolves_declaration_owned_contribution_closure() {
    let installed = installed_index()
        .bind_application_schema(ContributionSchema::declaration().unwrap())
        .unwrap();
    let catalog = installed.contributions();
    let topology = catalog.get("worth.tests.topology.v1").unwrap();

    assert_eq!(catalog.len(), 1);
    assert_eq!(topology.identity().as_str(), "worth.tests.topology.v1");
    assert_eq!(topology.member_ordinals(), &[0]);
    assert!(matches!(
        topology.members().next(),
        Some(ApplicationSchemaMember::Entity { entity }) if entity == "Component"
    ));
}

#[test]
fn reassigning_same_member_to_another_contribution_changes_installed_meaning() {
    let index = installed_index();
    let reassigned = ApplicationSchemaDeclarationBuilder::<ContributionSchema>::for_schema()
        .contributions()
        .register::<ReassignedContribution>()
        .unwrap()
        .build()
        .unwrap();

    let denial = index.bind_application_schema(reassigned).unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::SchemaMeaningChanged
    );
}

fn installed_index() -> WorthQueryInstalledPackageIndex {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        ContributionSchema::OWNER,
        1,
        0,
    ))
    .application_schema(ContributionSchema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
}
