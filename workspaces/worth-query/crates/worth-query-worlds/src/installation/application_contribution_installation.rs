pub use worth_query_declaration::facade::application_schema::{
    ApplicationEntityMarkerIdentity, ApplicationEntityRef, ApplicationSchema,
    ApplicationSchemaContributionAuthoring, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember,
};
pub use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledApplicationSchemaDenialKind,
    WorthQueryInstalledPackageIndex, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

pub struct ContributionSchema;
pub struct Component;

impl ApplicationEntityMarkerIdentity<ContributionSchema> for Component {
    const IDENTIFIER: &'static str = "Component";
}

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
    pub contribution ReassignedContribution in ContributionSchema {
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

pub fn installed_index() -> WorthQueryInstalledPackageIndex {
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
