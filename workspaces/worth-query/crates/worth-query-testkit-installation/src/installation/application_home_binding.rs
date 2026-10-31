//! Genuine declaration, installation and closed-roster admission for public Home entries.
use worth_foundational::facade::{ApplicationHomeBinding, CanonicalDigestId, HomeProgramBinding};
use worth_query_declaration::facade::application_program::{
    ApplicationFeature, ApplicationFeatureInputLeaf, ApplicationFeatureSpec,
    ApplicationNoOutputGraph, ApplicationProgramAuthoring, ApplicationProgramDefinition,
    ApplicationProgramIdentity, ApplicationProgramOutputs, ApplicationRuleLeaf,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEntityMarkerIdentity, ApplicationEntityRef, ApplicationSchema,
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaDeclarationDenial,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
    WorthQueryProgramSupportAdmission,
};

struct HomeSchema;
struct Record;
struct NativeFeature;
struct Program;

impl ApplicationEntityMarkerIdentity<HomeSchema> for Record {
    const IDENTIFIER: &'static str = "Record";
}
impl ApplicationSchema for HomeSchema {
    const OWNER: &'static str = "store-home-journey";
    const NAME: &'static str = "HomeSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .entity(ApplicationEntityRef::<Self, Record>::from_schema_identifier("Record"))
            .build()
    }
}
impl ApplicationFeature<HomeSchema> for NativeFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.store.home.native-feature.v1";
}
impl ApplicationProgramDefinition<HomeSchema> for Program {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.store.home.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<HomeSchema, NativeFeature>().finish()]
    }
}

pub fn admitted_home_binding() -> worth_proof::AdmittedApplicationHomeBinding<
    worth_query_installation::facade::WorthQueryHomeBindingAdmission,
> {
    let declaration = HomeSchema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        HomeSchema::OWNER,
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("home-journey", "native-owner")
        .admit(package)
        .unwrap();
    let installed = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(declaration)
    .unwrap();
    let program = ApplicationProgramAuthoring::<HomeSchema, Program>::begin()
        .validated_program()
        .unwrap();
    let roster = WorthQueryProgramSupportAdmission::for_installed_schema(&installed)
        .support(&program)
        .unwrap()
        .close()
        .unwrap();
    let selected = roster.entry(program.revision()).unwrap();
    let selected = HomeProgramBinding::new(
        selected.identity().as_str().into(),
        CanonicalDigestId::new(*selected.revision().as_bytes()),
    );
    let binding = installed.binding_identity();
    let description = ApplicationHomeBinding::new(
        *binding.package_identity(),
        *binding.schema_identity(),
        selected.clone(),
        vec![selected],
    )
    .unwrap();
    installed
        .admit_home_binding(&roster, program.revision(), description)
        .unwrap()
}
