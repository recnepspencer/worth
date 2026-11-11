pub use worth_query_declaration::facade::application_schema::{
    ApplicationEntityMarkerIdentity, ApplicationEntityRef, ApplicationSchema,
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember,
};
pub use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledApplicationSchemaDenialKind,
    WorthQueryInstalledPackageIndex, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

pub struct ContributionSchema;
pub struct Component;

use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    ApplicationSchemaContribution, ApplicationSchemaContributionIdentity,
    ApplicationSchemaDeclarationDenial,
};
impl Entity for Component {
    const NAME: &'static str = "Component";
    type Aspects = ();
    type Relations = ();
}
pub struct ComponentRecords;
impl Fragment for ComponentRecords {
    const NAME: &'static str = "component_records";
    type Bindings = NoBindings;
    type Members = (Component,);
}
pub struct TopologyContribution;
impl ApplicationSchemaContribution<ContributionSchema> for TopologyContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.tests.topology.v1");
    type Members = (ComponentRecords,);
}
pub struct ReassignedContribution;
impl ApplicationSchemaContribution<ContributionSchema> for ReassignedContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.tests.reassigned.v1");
    type Members = (ComponentRecords,);
}
impl Fragment for ContributionSchema {
    const NAME: &'static str = "ContributionSchema";
    type Bindings = NoBindings;
    type Members = (TopologyContribution,);
}
impl ApplicationSchema for ContributionSchema {
    const OWNER: &'static str = "worth.tests.contribution-installation";
    const NAME: &'static str = "ContributionSchema";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
struct ReassignedRoot;
impl Fragment for ReassignedRoot {
    const NAME: &'static str = <ContributionSchema as Fragment>::NAME;
    type Bindings = NoBindings;
    type Members = (ReassignedContribution,);
}
pub fn reassigned_declaration() -> ApplicationSchemaDeclaration<ContributionSchema> {
    ApplicationSchemaDeclarationBuilder::<ContributionSchema>::from_fragment(ReassignedRoot)
        .unwrap()
        .build()
        .unwrap()
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
