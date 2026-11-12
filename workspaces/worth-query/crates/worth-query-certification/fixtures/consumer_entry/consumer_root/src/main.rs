#![cfg_attr(test, allow(clippy::disallowed_methods))]
#![forbid(unsafe_code)]
// Threaded owner tests derive Send/Sync through nested persistent graph storage.
#![recursion_limit = "256"]
// Query's public denials are deliberately rich, so its workspace allows large
// error variants; this consumer returns them unchanged.
#![allow(clippy::result_large_err)]

mod application_invariant_acceptance;
mod application_program;
#[cfg(test)]
mod inline_type_sizes;
mod program_deployment;
pub mod proposal_acceptance;
use program_deployment::ConsumerProgramDeployment;

pub use application_program::{
    ConsumerDiscoveredProgramRoot, ConsumerProgram, ConsumerProgramRoot,
    ConsumerSecondaryProgramRoot, ConsumerTruncatedProgramRoot, ConsumerUndeclaredProgramRoot,
};

use worth_query_consumer_values::PositiveLength;
use worth_query_decl::facade::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding, ApplicationSchema,
};
use worth_query_host::facade::domain::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledApplicationSchema,
    WorthQueryInstalledPackageIndex, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};
use worth_query_parameter_entry::{
    ParameterContribution, ParameterCountBinding, ParameterSchemaBinding,
};
use worth_query_topology_entry::{
    TopologyContribution, TopologyLengthBinding, TopologySchemaBinding,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConsumerSchema;
impl worth_query_decl::facade::application_schema::fragment::Fragment for ConsumerSchema {
    const NAME: &'static str = "ConsumerSchema";
    type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
    type Members = (
        TopologyContribution,
        ParameterContribution,
        ConsumerProgramDeployment,
    );
}
impl worth_query_decl::facade::application_schema::ApplicationSchema for ConsumerSchema {
    const OWNER: &'static str = "worth.query.certification.consumer";
    const NAME: &'static str = "ConsumerSchema";
    const MAJOR: u32 = 2;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclaration<Self>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        worth_query_decl::facade::application_schema::fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        worth_query_decl::facade::application_schema::fragment::visit_correction(
            Self, source, visitor,
        )
    }
}
impl worth_query_decl::facade::application_schema::ApplicationSchemaComposition for ConsumerSchema {
    type Contributions = (
        TopologyContribution,
        ParameterContribution,
        ConsumerProgramDeployment,
    );
}

impl TopologySchemaBinding for ConsumerSchema {
    type PrincipalFragment = program_deployment::ConsumerPrincipalRecords;
    const ESCAPING_PLANAR_OUTPUT: bool = true;
}
impl ParameterSchemaBinding for ConsumerSchema {}

fn main() {
    run();
}

fn installed_schema() -> WorthQueryInstalledApplicationSchema<ConsumerSchema> {
    let declaration = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration()
        .expect("the root-owned contributions form one closed schema declaration");

    assert_eq!(declaration.contributions().len(), 3);
    assert_ne!(
        TopologyLengthBinding::IDENTITY,
        ParameterCountBinding::IDENTITY
    );

    let length = PositiveLength::new(42).expect("the fixture length is positive");
    let encoded = TopologyLengthBinding::encode(&length).expect("the binding accepts the value");
    assert_eq!(
        TopologyLengthBinding::decode(&encoded).expect("the binding decodes its carrier"),
        length
    );

    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        ConsumerSchema::OWNER,
        ConsumerSchema::MAJOR,
        ConsumerSchema::MINOR,
    ))
    .application_schema(declaration.clone())
    .validate()
    .expect("the root declaration forms a portable package");
    let admitted = WorthQueryInstallationAdmissionProfile::new(
        "worth.query.certification.support",
        "worth.query.certification.configuration",
    )
    .admit(package)
    .expect("the public admission path accepts the root package");
    let installed_index = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .expect("the public installation path installs the root package");
    let installed = installed_index
        .bind_application_schema(declaration)
        .expect("the installed authority binds the exact root declaration");
    assert_eq!(installed.contributions().len(), 3);
    assert!(installed
        .contributions()
        .get("worth.query.certification.topology.v2")
        .is_some());
    assert!(installed
        .contributions()
        .get("worth.query.certification.parameter.v1")
        .is_some());
    assert!(installed
        .contributions()
        .get("worth.query.certification.consumer-deployment.v1")
        .is_some());
    installed
}

fn run() {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let document_history = match arguments.as_slice() {
        [] => false,
        [argument] if argument == "--document-history" => true,
        _ => panic!("usage: worth-query-consumer-root [--document-history]"),
    };
    let installed = installed_schema();

    hostile_binding_identity_and_unit_cannot_preserve_schema_identity();
    if document_history {
        application_invariant_acceptance::document_history::run(&installed);
    } else {
        application_invariant_acceptance::run(&installed);
    }
}

#[cfg_attr(test, test)]
fn hostile_binding_identity_and_unit_cannot_preserve_schema_identity() {
    let baseline = <hostile::baseline::Schema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().unwrap();
    let wrong_identity = <hostile::wrong_identity::Schema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().unwrap();
    let wrong_unit = <hostile::wrong_unit::Schema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().unwrap();
    for schema in [
        baseline.erased(),
        wrong_identity.erased(),
        wrong_unit.erased(),
    ] {
        let postures = schema
            .members()
            .iter()
            .filter_map(|member| match member {
                worth_query_decl::facade::application_schema::ApplicationSchemaMember::Field {
                    field,
                    equality_queryable,
                    ..
                } => Some((field.as_str(), *equality_queryable)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(postures, vec![("value", true)]);
    }

    assert_ne!(baseline.identity(), wrong_identity.identity());
    assert_ne!(baseline.identity(), wrong_unit.identity());
}

mod hostile;
