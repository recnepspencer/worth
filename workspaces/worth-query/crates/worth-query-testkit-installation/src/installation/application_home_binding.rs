//! Genuine declaration, installation and closed-roster admission for public Home entries.
use worth_foundational::facade::{ApplicationHomeBinding, CanonicalDigestId, HomeProgramBinding};
use worth_query_declaration::facade::application_program::{
    ApplicationFeature, ApplicationFeatureInputLeaf, ApplicationFeatureSpec,
    ApplicationNoOutputGraph, ApplicationProgramAuthoring, ApplicationProgramDefinition,
    ApplicationProgramIdentity, ApplicationProgramOutputs, ApplicationRuleLeaf,
};
use worth_query_declaration::facade::application_schema::{
    fragment::{self, Fragment, NoBindings},
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
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
struct Successor;
struct OtherSuccessor;

impl fragment::Entity for Record {
    const NAME: &'static str = "Record";
    type Aspects = ();
    type Relations = ();
}
impl Fragment for HomeSchema {
    const NAME: &'static str = "HomeSchema";
    type Bindings = NoBindings;
    type Members = (Record,);
}
impl ApplicationSchema for HomeSchema {
    const OWNER: &'static str = "store-home-journey";
    const NAME: &'static str = "HomeSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
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

impl ApplicationProgramDefinition<HomeSchema> for Successor {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.store.home.program.v2");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        Program::feature_specs()
    }
}

impl ApplicationProgramDefinition<HomeSchema> for OtherSuccessor {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.store.home.program.v3");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        Program::feature_specs()
    }
}

pub fn admitted_home_binding() -> worth_proof::AdmittedApplicationHomeBinding<
    worth_query_installation::facade::WorthQueryHomeBindingAdmission,
> {
    admitted_binding::<Program>()
}

/// A second real compiled program, under the same installed schema owner.
pub fn admitted_transition_successor_binding() -> worth_proof::AdmittedApplicationHomeBinding<
    worth_query_installation::facade::WorthQueryHomeBindingAdmission,
> {
    admitted_binding::<Successor>()
}

pub fn admitted_other_successor_binding() -> worth_proof::AdmittedApplicationHomeBinding<
    worth_query_installation::facade::WorthQueryHomeBindingAdmission,
> {
    admitted_binding::<OtherSuccessor>()
}

fn admitted_binding<Selected: ApplicationProgramDefinition<HomeSchema>>(
) -> worth_proof::AdmittedApplicationHomeBinding<
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
    let program = ApplicationProgramAuthoring::<HomeSchema, Selected>::begin()
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
