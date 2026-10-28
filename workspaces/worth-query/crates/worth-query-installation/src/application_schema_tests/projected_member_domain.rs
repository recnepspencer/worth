//! Installed projected-domain meaning binds through the real schema index.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationProjectedMemberCaseRef, ApplicationProjectedMemberDomainDefinition,
    ApplicationProjectedMemberDomainRef,
};

struct ProjectedSchema;
struct Uses;
struct Side;

impl ApplicationSchema for ProjectedSchema {
    const OWNER: &'static str = "typed-test";
    const NAME: &'static str = "ProjectedSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        let carrier =
            ApplicationEntityRef::<Self, FixtureEntity<Self>>::from_schema_identifier("TestEntity");
        let domain = ApplicationProjectedMemberDomainRef::<Self, Uses, FixtureEntity<Self>>::new(
            "Uses", carrier,
        );
        let side = ApplicationProjectedMemberCaseRef::<Self, Uses, Side>::new(domain, "Side");
        test_schema_members::<Self>(None)
            .projected_member_domain(
                ApplicationProjectedMemberDomainDefinition::new(domain)
                    .case(side)
                    .unwrap(),
            )
            .build()
    }
}

#[test]
fn projected_member_meaning_binds_through_the_installed_schema_index() {
    let declaration = ProjectedSchema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "typed-test",
        1,
        0,
    ))
    .application_schema(ProjectedSchema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    let index = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap();
    index.bind_application_schema(declaration).unwrap();
    installed_index()
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
}
